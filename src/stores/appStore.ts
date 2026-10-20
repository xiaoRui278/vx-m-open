import { defineStore } from "pinia";
import * as api from "../api";
import type { LaunchTarget, WeChatInfo } from "../types";

let pollTimer: ReturnType<typeof setInterval> | null = null;

export const useAppStore = defineStore("app", {
  state: () => ({
    info: null as WeChatInfo | null,
    targets: [] as LaunchTarget[],
    busy: false,
    busyText: "",
    message: "",
    messageType: "info" as "info" | "error",
    messageTimer: 0 as ReturnType<typeof setTimeout> | 0,
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
      await this.runLong("正在启动…", async () => {
        await api.launchInstance(path);
        this.showMessage("已发起启动，窗口即将弹出", "info");
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
      await this.runLong("正在关闭…", async () => {
        await api.terminateInstance(pid);
        this.showMessage(`实例 ${pid} 已关闭`, "info");
      });
    },
    async generateCopy(name: string) {
      await this.runLong("正在创建副本（克隆 + 重签名，约 10 秒）…", async () => {
        await api.generateCopy(name);
        this.showMessage(`副本「${name}」创建成功`, "info");
      });
    },
    async renameCopy(path: string, newName: string) {
      await this.runLong("正在改名（重签名，约 10 秒）…", async () => {
        await api.renameCopy(path, newName);
        this.showMessage(`已改名为「${newName}」`, "info");
      });
    },
    async deleteCopy(path: string) {
      await this.runLong("正在移到废纸篓…", async () => {
        await api.deleteCopy(path);
        this.showMessage("副本已移到废纸篓（账号数据保留）", "info");
      });
    },
    async rebuildCopy(path: string) {
      await this.runLong(
        "正在重建副本（保留名字与账号数据，约 10 秒）…",
        async () => {
          await api.rebuildCopy(path);
          this.showMessage("副本已重建为新版本，账号无需重新登录", "info");
        },
      );
    },
    async setMainAlias(name: string) {
      await this.runLong("保存中…", async () => {
        await api.setMainAlias(name);
        this.showMessage("主程序备注已保存", "info");
      });
    },
    init() {
      void this.refreshAll();
      if (pollTimer) clearInterval(pollTimer);
      pollTimer = setInterval(() => {
        if (!this.busy) void this.refreshAll();
      }, 3000);
    },
  },
});
