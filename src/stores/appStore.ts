import { defineStore } from "pinia";
import { getVersion } from "@tauri-apps/api/app";
import * as api from "../api";
import { t } from "../i18n";
import type { LaunchTarget, WeChatInfo } from "../types";

let pollTimer: ReturnType<typeof setInterval> | null = null;

export const GITHUB_URL = "https://github.com/xiaoRui278/vx-m-open";
export const SITE_URL = "https://vxmo.cting.vip";
const CHECK_TIMEOUT_MS = 5000;

export const useAppStore = defineStore("app", {
  state: () => ({
    info: null as WeChatInfo | null,
    targets: [] as LaunchTarget[],
    busy: false,
    busyText: "",
    message: "",
    messageType: "info" as "info" | "error",
    messageTimer: 0 as ReturnType<typeof setTimeout> | 0,
    appVersion: "",
    updateAvailable: false,
    updateVersion: "",
  }),
  actions: {
    async refreshAll() {
      try {
        const [info, targets] = await Promise.all([
          api.detectWechat(),
          api.listLaunchTargets(),
        ]);
        this.info = info;
        this.targets = targets;
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
    /** 启动时读取应用版本并检查官网最新版本 */
    async initVersion() {
      try {
        this.appVersion = await getVersion();
      } catch {
        this.appVersion = "";
      }
      await this.checkUpdate();
    },
    /** 返回 "ok"（已是最新）/ "new"（有新版）/ "fail"（网络检查失败） */
    async checkUpdate(): Promise<"ok" | "new" | "fail"> {
      const { version, failed } = await fetchLatestVersion();
      if (failed) return "fail";
      if (version && isNewer(version, this.appVersion)) {
        this.updateVersion = version;
        this.updateAvailable = true;
        return "new";
      }
      this.updateAvailable = false;
      return "ok";
    },
    init() {
      void this.refreshAll();
      void this.initVersion();
      if (pollTimer) clearInterval(pollTimer);
      pollTimer = setInterval(() => {
        if (!this.busy) void this.refreshAll();
      }, 3000);
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

/** 取最新版本号：官网 latest.json 优先，GitHub Release 兜底；全部失败返回 failed: true */
async function fetchLatestVersion(): Promise<{ version: string; failed: boolean }> {
  try {
    const data = (await fetchJsonWithTimeout(
      `${SITE_URL}/latest.json`,
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
