import { defineStore } from "pinia";
import { getVersion } from "@tauri-apps/api/app";
import * as api from "../api";
import { t } from "../i18n";
import type { LaunchTarget, WeChatInfo } from "../types";

let pollTimer: ReturnType<typeof setInterval> | null = null;

export const GITHUB_URL = "https://github.com/xiaoRui278/vx-m-open";

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
    /** 启动时读取应用版本并检查 GitHub 最新 Release */
    async initVersion() {
      try {
        this.appVersion = await getVersion();
      } catch {
        this.appVersion = "";
      }
      await this.checkUpdate();
    },
    async checkUpdate(): Promise<boolean> {
      try {
        const res = await fetch(
          `${GITHUB_URL.replace("github.com", "api.github.com/repos")}/releases/latest`,
          { headers: { Accept: "application/vnd.github+json" } },
        );
        if (!res.ok) return false;
        const data = (await res.json()) as { tag_name?: string };
        const latest = (data.tag_name ?? "").replace(/^v/, "");
        if (latest && isNewer(latest, this.appVersion)) {
          this.updateVersion = latest;
          this.updateAvailable = true;
          return true;
        }
        this.updateAvailable = false;
        return false;
      } catch {
        return false;
      }
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
