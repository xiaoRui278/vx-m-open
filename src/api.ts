import { invoke } from "@tauri-apps/api/core";
import type { LaunchTarget, WeChatInfo } from "./types";

export const detectWechat = () => invoke<WeChatInfo>("detect_wechat");

export const listLaunchTargets = () =>
  invoke<LaunchTarget[]>("list_launch_targets");

export const launchInstance = (path: string) =>
  invoke<void>("launch_instance", { path });

export const activateInstance = (path: string, pid: number | null) =>
  invoke<void>("activate_instance", { path, pid: pid ?? undefined });

export const terminateInstance = (pid: number) =>
  invoke<void>("terminate_instance", { pid });

export const generateCopy = (name: string) =>
  invoke<void>("generate_copy", { name });

export const renameCopy = (path: string, newName: string) =>
  invoke<void>("rename_copy", { path, newName });

export const deleteCopy = (path: string) =>
  invoke<void>("delete_copy", { path });

export const rebuildCopy = (path: string) =>
  invoke<void>("rebuild_copy", { path });

export const setMainAlias = (name: string) =>
  invoke<void>("set_main_alias", { name });
