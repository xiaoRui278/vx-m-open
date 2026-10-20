export interface WeChatInfo {
  installed: boolean;
  path: string | null;
  version: string | null;
  bundleId: string | null;
}

/** 主程序或副本（含运行状态） */
export interface LaunchTarget {
  path: string;
  appName: string;
  label: string;
  version: string | null;
  isMain: boolean;
  running: boolean;
  pid: number | null;
  /** Unix 秒 */
  startedAt: number | null;
  /** 副本版本落后于主程序（微信升级后） */
  outdated: boolean;
  /** 运行实例数（Windows 主程序多开时 > 1；macOS 恒为 0 或 1） */
  instanceCount: number;
}
