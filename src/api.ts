import { invoke } from "@tauri-apps/api/core";
import type { LaunchTarget, WeChatInfo } from "./types";

/* —— 浏览器预览夹具（pnpm dev 用普通浏览器直接查看 UI；Tauri 内零影响）—— */
const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const MOCK_NOW = Math.floor(Date.now() / 1000);

const MOCK_TARGETS: LaunchTarget[] = [
  {
    path: "/Applications/WeChat.app",
    appName: "WeChat",
    label: "WeChat",
    version: "4.1.5.16",
    isMain: true,
    running: true,
    pid: 4821,
    startedAt: MOCK_NOW - 5400,
    outdated: false,
    instanceCount: 2,
  },
  {
    path: "/Applications/WeChat 工作号.app",
    appName: "WeChat",
    label: "工作号",
    version: "4.1.5.16",
    isMain: false,
    running: true,
    pid: 5107,
    startedAt: MOCK_NOW - 1200,
    outdated: false,
    instanceCount: 0,
  },
  {
    path: "/Applications/WeChat 测试号（这是一个非常长的备注名称用来验证换行）.app",
    appName: "WeChat",
    label: "测试号（这是一个非常长的备注名称用来验证换行）",
    version: "4.0.3.99",
    isMain: false,
    running: false,
    pid: null,
    startedAt: null,
    outdated: true,
    instanceCount: 0,
  },
];

const MOCK_WECHAT: WeChatInfo = {
  installed: true,
  path: "/Applications/WeChat.app",
  version: "4.1.5.16",
  bundleId: "com.tencent.xinWeChat",
};

async function previewInvoke<T>(cmd: string): Promise<T> {
  await new Promise((r) => setTimeout(r, 150));
  switch (cmd) {
    case "detect_wechat":
      return MOCK_WECHAT as T;
    case "list_launch_targets":
      return MOCK_TARGETS as T;
    case "rebuild_all":
      return 2 as T;
    default:
      return undefined as T;
  }
}

const call = inTauri
  ? invoke
  : (((cmd: string) => previewInvoke(cmd)) as unknown as typeof invoke);

export const detectWechat = () => call<WeChatInfo>("detect_wechat");

export const listLaunchTargets = () => call<LaunchTarget[]>("list_launch_targets");

export const launchInstance = (path: string) =>
  call<void>("launch_instance", { path });

export const activateInstance = (path: string, pid: number | null) =>
  call<void>("activate_instance", { path, pid: pid ?? undefined });

export const terminateInstance = (pid: number) =>
  call<void>("terminate_instance", { pid });

export const generateCopy = (name: string) =>
  call<void>("generate_copy", { name });

export const renameCopy = (path: string, newName: string) =>
  call<void>("rename_copy", { path, newName });

export const deleteCopy = (path: string) => call<void>("delete_copy", { path });

export const rebuildCopy = (path: string) => call<void>("rebuild_copy", { path });

export const setMainAlias = (name: string) =>
  call<void>("set_main_alias", { name });

export const rebuildAll = () => call<number>("rebuild_all");

export const openUrl = (url: string) => call<void>("open_url", { url });
