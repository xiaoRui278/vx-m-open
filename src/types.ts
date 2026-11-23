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
  /** 所属应用 profile（微信恒 "wechat"，UI 暂不展示） */
  profileId: string;
}

/** Windows 运行实例的临时备注（按 pid + 启动时间匹配，进程关闭即失效） */
export interface InstanceRemark {
  pid: number;
  startedAt: number;
  remark: string;
}

/** 远程兼容清单中单个 profile 的状态（v1.3，与 manifest 字段对齐） */
export type RemoteProfileStatusLevel = "ok" | "degraded" | "broken" | "disabled";

/** 远程兼容清单 message 字段（中英双语，与后端 LocalizedText 同构） */
export interface RemoteLocalizedMessage {
  zh: string;
  en: string;
}

/** 归一化后的 profile 远程状态；无清单/解析失败/条目不适用 = ok */
export interface RemoteProfileStatus {
  status: RemoteProfileStatusLevel;
  message: RemoteLocalizedMessage | null;
}
