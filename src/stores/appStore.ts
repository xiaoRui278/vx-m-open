import { defineStore } from "pinia";
import { getVersion } from "@tauri-apps/api/app";
import * as api from "../api";
import { installUpdate, updaterAvailable } from "../updater";
import { t } from "../i18n";
import type { InstanceRemark, LaunchTarget, WeChatInfo } from "../types";

let pollTimer: ReturnType<typeof setInterval> | null = null;
let visBound = false;

export const GITHUB_URL = "https://github.com/xiaoRui278/vx-m-open";
export const SITE_URL = "https://vxmo.cting.vip";
const CHECK_TIMEOUT_MS = 5000;

export const useAppStore = defineStore("app", {
  state: () => ({
    info: null as WeChatInfo | null,
    targets: [] as LaunchTarget[],
    instanceRemarks: [] as InstanceRemark[],
    busy: false,
    busyText: "",
    message: "",
    messageType: "info" as "info" | "error",
    messageTimer: 0 as ReturnType<typeof setTimeout> | 0,
    appVersion: "",
    updateAvailable: false,
    updateVersion: "",
    updateNotes: "",
    updateViaUpdater: false,
  }),
  actions: {
    async refreshAll() {
      try {
        const [info, targets, remarks] = await Promise.all([
          api.detectWechat(),
          api.listLaunchTargets(),
          api.listInstanceRemarks(),
        ]);
        this.info = info;
        this.targets = targets;
        this.instanceRemarks = remarks;
      } catch (e) {
        this.showMessage(String(e), "error");
      }
    },
    showMessage(msg: string, type: "info" | "error") {
      this.message = msg;
      this.messageType = type;
      if (this.messageTimer) clearTimeout(this.messageTimer);
      this.messageTimer = setTimeout(() => (this.message = ""), 6000);
    },
    clearMessage() {
      this.message = "";
    },
    /** 长操作通用包装：busy 提示 + 完成后刷新 */
    async runLong(text: string, fn: () => Promise<void>) {
      this.busy = true;
      this.busyText = text;
      try {
        await fn();
        await this.refreshAll();
      } catch (e) {
        this.showMessage(String(e), "error");
      } finally {
        this.busy = false;
        this.busyText = "";
      }
    },
    async launch(path: string) {
      await this.runLong(t("launching"), async () => {
        await api.launchInstance(path);
      });
    },
    async activate(path: string, pid: number | null) {
      try {
        await api.activateInstance(path, pid);
      } catch (e) {
        this.showMessage(String(e), "error");
      }
    },
    async terminate(pid: number) {
      await this.runLong(t("close"), async () => {
        await api.terminateInstance(pid);
      });
    },
    async renameCopy(path: string, newName: string) {
      await this.runLong(t("saving"), async () => {
        await api.renameCopy(path, newName);
      });
    },
    async deleteCopy(path: string) {
      await this.runLong(t("deleting"), async () => {
        await api.deleteCopy(path);
      });
    },
    async rebuildCopy(path: string) {
      await this.runLong(t("rebuilding"), async () => {
        await api.rebuildCopy(path);
        this.showMessage(t("rebuilt"), "info");
      });
    },
    async setMainAlias(name: string) {
      await this.runLong(t("saving"), async () => {
        await api.setMainAlias(name);
      });
    },
    /** 设置 Windows 实例备注（跟随进程，进程关闭即失效） */
    async setInstanceRemark(pid: number, startedAt: number, remark: string) {
      await this.runLong(t("saving"), async () => {
        await api.setInstanceRemark(pid, startedAt, remark);
      });
    },
    /** 启动时读取应用版本并检查官网最新版本 */
    async initVersion() {
      try {
        this.appVersion = await getVersion();
      } catch {
        this.appVersion = "";
      }
      await this.checkUpdate();
    },
    /** 返回 "ok"（已是最新）/ "new"（有新版）/ "fail"（网络检查失败）。
     *  判断层保持现状：官网 latest.json 优先、GitHub 兜底（顺带匿名统计）。
     *  下载层（updater 通道）只在用户点「立即更新」时才连，见 updateNow。 */
    async checkUpdate(): Promise<"ok" | "new" | "fail"> {
      const { version, failed } = await fetchLatestVersion(this.appVersion);
      if (failed) return "fail";
      if (version && isNewer(version, this.appVersion)) {
        this.updateVersion = version;
        this.updateNotes = "";
        this.updateViaUpdater = updaterAvailable;
        this.updateAvailable = true;
        return "new";
      }
      this.updateAvailable = false;
      return "ok";
    },
    /** 一键更新：连 updater 通道下载安装并重启。失败返回 false（横幅降级为"前往官网"）。
     *  不走 runLong：它内部吞异常且会触发 refreshAll，更新失败需要自己接住并降级 */
    async updateNow(): Promise<boolean> {
      this.busy = true;
      this.busyText = t("updatePreparing");
      try {
        await installUpdate((p) => {
          this.busyText = p.percent >= 0 ? t("updateDownloading", { percent: p.percent }) : t("updateDownloadingUnknown");
        });
        return true; // relaunch 成功则进程已重启，走不到这行
      } catch (e) {
        this.updateViaUpdater = false;
        this.showMessage(String(e), "error");
        return false;
      } finally {
        this.busy = false;
        this.busyText = "";
      }
    },
    init() {
      void this.refreshAll();
      void this.initVersion();
      if (pollTimer) clearInterval(pollTimer);
      pollTimer = setInterval(() => {
        // 窗口不可见时暂停轮询（Tauri 最小化/完全遮挡时 webview 不可见；部分环境可能始终 visible，仅优化失效无副作用）
        if (document.visibilityState !== "visible") return;
        if (!this.busy) void this.refreshAll();
      }, 3000);
      if (!visBound) {
        visBound = true;
        document.addEventListener("visibilitychange", () => {
          if (document.visibilityState === "visible") void this.refreshAll();
        });
      }
    },
  },
});

/** fetch JSON + 超时中断（含响应体读取；AbortSignal.timeout 在 macOS 11 的 WKWebView 不可用，手动实现） */
async function fetchJsonWithTimeout(url: string, init: RequestInit, ms: number): Promise<unknown> {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), ms);
  try {
    const res = await fetch(url, { ...init, signal: ctrl.signal });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return await res.json();
  } finally {
    clearTimeout(timer);
  }
}

/** 匿名安装标识：首启随机生成、localStorage 持久化。仅用于访问日志计数，无任何行为数据 */
function installUuid(): string {
  try {
    let id = localStorage.getItem("vxmo_install_id");
    if (!id) {
      id = crypto.randomUUID();
      localStorage.setItem("vxmo_install_id", id);
    }
    return id;
  } catch {
    return "";
  }
}

/** 平台标记：供 OSS 访问日志分平台统计 */
function platformTag(): string {
  const ua = navigator.userAgent;
  if (ua.includes("Mac")) return "macos";
  if (ua.includes("Win")) return "windows";
  return "other";
}

/** 取最新版本号：官网 latest.json 优先（顺带匿名上报安装标识/平台/版本，OSS 忽略 query 但日志留痕），GitHub Release 兜底；全部失败返回 failed: true */
async function fetchLatestVersion(appVersion: string): Promise<{ version: string; failed: boolean }> {
  try {
    const qs = new URLSearchParams();
    const uuid = installUuid();
    if (uuid) qs.set("u", uuid);
    qs.set("p", platformTag());
    if (appVersion) qs.set("v", appVersion);
    const data = (await fetchJsonWithTimeout(
      `${SITE_URL}/latest.json?${qs.toString()}`,
      { cache: "no-store" },
      CHECK_TIMEOUT_MS,
    )) as { version?: string };
    const version = (data.version ?? "").trim();
    if (version) return { version, failed: false };
  } catch {
    /* 官网未放开跨域、超时或响应异常，静默转兜底 */
  }
  try {
    const data = (await fetchJsonWithTimeout(
      `${GITHUB_URL.replace("github.com", "api.github.com/repos")}/releases/latest`,
      { headers: { Accept: "application/vnd.github+json" } },
      CHECK_TIMEOUT_MS,
    )) as { tag_name?: string };
    const version = (data.tag_name ?? "").replace(/^v/, "").trim();
    if (!version) return { version: "", failed: true };
    return { version, failed: false };
  } catch {
    return { version: "", failed: true };
  }
}

/** 语义化版本比较：a > b 返回 true */
function isNewer(a: string, b: string): boolean {
  const pa = a.split(".").map((n) => parseInt(n, 10) || 0);
  const pb = b.split(".").map((n) => parseInt(n, 10) || 0);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const x = pa[i] ?? 0;
    const y = pb[i] ?? 0;
    if (x !== y) return x > y;
  }
  return false;
}
