/**
 * 轻量 i18n：中文 / English，偏好持久化到 localStorage。
 * 默认语言跟随系统（zh 开头 → 中文，否则英文），可在设置中手动指定。
 */
import { ref } from "vue";

export type Locale = "zh" | "en";

const LOCALE_KEY = "vxm.locale";

function detectLocale(): Locale {
  const saved = localStorage.getItem(LOCALE_KEY);
  if (saved === "zh" || saved === "en") return saved;
  return navigator.language.toLowerCase().startsWith("zh") ? "zh" : "en";
}

export const locale = ref<Locale>(detectLocale());

export function setLocale(l: Locale) {
  locale.value = l;
  localStorage.setItem(LOCALE_KEY, l);
}

const messages = {
  zh: {
    // 导航 / 页头
    navInstances: "实例管理",
    navSettings: "设置",
    instancesTitle: "实例管理",
    instancesSubMac: "创建微信副本并管理多个账号的启动、改名与删除",
    instancesSubWin: "直接多开微信本体，无需创建副本——每次启动新实例即可登录不同账号",
    settingsTitle: "设置",
    settingsSub: "语言、主题、路径与关于信息",

    // 环境检测
    wechatInstalled: "微信已安装",
    wechatMissing: "未检测到微信",
    wechatMissingMacHint: "请确认微信已安装到 /Applications 目录",
    wechatMissingWinHint: "请确认微信已安装（默认安装目录或注册表指定路径）",

    // 实例（macOS）
    copiesLabel: "实例",
    createCopy: "创建副本",
    outdatedWarning: "微信已升级，{n} 个副本版本落后，建议重建（账号数据保留）",
    rebuildAll: "全部重建",
    emptyTitle: "还没有任何实例",
    emptySub: "创建第一个副本，即可用另一个账号同时登录微信",
    main: "主程序",
    wechat: "微信",
    copy: "副本",
    outdatedHint: "微信升级后需重建副本",
    launch: "启动",
    activate: "激活",
    close: "关闭",
    rename: "改名",
    remark: "备注",
    remove: "删除",
    rebuild: "重建",
    startedAt: "{time} 启动",
    launchNew: "启动新实例",
    launching: "启动中…",
    macModel: "macOS · 副本模型",
    winModel: "Windows · 本体多开",

    // Windows 实例行
    winEmptyTitle: "尚未运行任何实例",
    winEmptySub: "点击上方「启动新实例」即可多开一个微信，用不同账号同时登录",
    runningLabel: "运行中的实例",
    instance: "实例",
    instancesRunning: "{n} 个实例运行中",

    // 弹框
    cancel: "取消",
    confirm: "确定",
    createTitle: "创建副本",
    createHint:
      "克隆主程序并重签名，生成独立数据目录的副本，创建后可在启动台 / Dock 直接看到此名字",
    createPlaceholder: "输入名字，如：工作号",
    renameTitle: "重命名「{name}」",
    renameHint: "名字会同步显示在 Finder / 启动台 / Dock，账号数据不受影响",
    aliasTitle: "设置主程序备注",
    aliasHint: "备注名仅在本工具内显示，不会修改微信本体",
    instanceRemarkTitle: "设置实例备注",
    instanceRemarkHint: "备注仅在本工具内显示，关闭该实例后失效",
    saveRemark: "保存备注",
    save: "保存",
    deleteTitle: "删除「{name}」",
    deleteHint:
      "副本应用会移到废纸篓（可恢复），该账号的数据仍保留在独立容器中，不会丢失。",
    rebuildTitle: "重建「{name}」",
    rebuildHint: "将重新克隆当前版本的微信，名字与账号数据全部保留，无需重新登录。",
    deleting: "删除中…",
    creating: "创建中…",
    rebuilding: "重建中…",
    saving: "保存中…",
    defaultCopyName: "副本 {n}",

    // 创建进度弹框
    progressTitle: "正在创建「{name}」",
    progressHint: "克隆并重签名通常需要约 10 秒，期间请勿退出应用。",
    stepClone: "克隆主程序（APFS）",
    stepBundle: "修改 Bundle ID",
    stepSign: "重签名副本",
    stepDone: "完成",

    // 全部重建进度弹框
    rebuildAllTitle: "正在重建落后副本",
    rebuildAllProgress: "正在重建「{label}」（{done}/{total}）",
    rebuildAllFinishing: "收尾中…",

    // Windows 备注弹框
    remarkPlaceholder: "输入备注名，如：主号",

    // 设置页
    groupGeneral: "常规",
    groupAbout: "关于",
    wechatPath: "微信路径",
    copyLocation: "副本位置",
    copyLocationVal: "/Applications（无写入权限时自动回退 ~/Applications）",
    detectMethod: "检测方式",
    detectMethodVal: "注册表（Software\\Tencent\\Weixin），回退默认安装目录",
    dataLocation: "数据位置",
    dataLocationVal:
      "微信本体安装目录，多开实例共享本体、各自独立登录，无副本文件",
    language: "语言",
    theme: "主题",
    themeSystem: "跟随系统",
    themeLight: "亮色",
    themeDark: "暗色",
    version: "版本",
    versionSuffixMac: "（macOS 副本管理）",
    versionSuffixWin: "（Windows 本体多开）",
    github: "GitHub",
    website: "官网",
    statementVal: "开源工具，不修改微信本体、不注入、不碰网络协议；非官方第三方工具，与腾讯无关，请遵守微信用户协议",
    statementWinVal:
      "开源工具，启动时挂起进程并关闭单实例互斥体句柄，不修改微信本体、不注入、不碰网络协议；非官方第三方工具，与腾讯无关，请遵守微信用户协议",
    disclaimer: "声明",
    openProjectPage: "打开项目主页",
    checkUpdate: "检查更新",
    upToDate: "已是最新版本",
    updateAvailable: "发现新版本 v{version}",
    updateDownload: "前往下载",
    checking: "检查中…",
    checkFailed: "检查失败（网络不可用）",

    // 支持作者
    donateTitle: "支持作者",
    donateSub: "如果这个工具帮到了你，欢迎请作者喝杯咖啡～",
    donateWechat: "微信支付",
    donateAlipay: "支付宝",

    // 更新横幅
    updateBanner: "VXMOpen v{version} 已发布，前往下载页获取新版本",

    // Toast / 状态（仅异步长任务完成与错误；可见结果静默——见 design.md）
    copyCreated: "副本「{name}」创建成功",
    rebuilt: "副本已重建为新版本，账号无需重新登录",
    rebuiltAll: "已重建 {n} 个副本",
    opFailed: "操作失败",
    winNoCopy: "Windows 端不支持副本操作",
  },
  en: {
    navInstances: "Instances",
    navSettings: "Settings",
    instancesTitle: "Instance Manager",
    instancesSubMac: "Create WeChat copies and manage launching, renaming and removal of multiple accounts",
    instancesSubWin: "Run multiple WeChat instances directly — no copies needed, launch a new instance for each account",
    settingsTitle: "Settings",
    settingsSub: "Language, theme, paths and about",

    wechatInstalled: "WeChat installed",
    wechatMissing: "WeChat not detected",
    wechatMissingMacHint: "Make sure WeChat is installed in /Applications",
    wechatMissingWinHint: "Make sure WeChat is installed (default directory or registry path)",

    copiesLabel: "Instances",
    createCopy: "Create Copy",
    outdatedWarning: "WeChat has been updated. {n} copies are outdated — rebuild recommended (account data is kept)",
    rebuildAll: "Rebuild All",
    emptyTitle: "No instances yet",
    emptySub: "Create your first copy to sign in to WeChat with another account",
    main: "Main",
    wechat: "WeChat",
    copy: "Copy",
    outdatedHint: "Rebuild required after WeChat update",
    launch: "Launch",
    activate: "Activate",
    close: "Quit",
    rename: "Rename",
    remark: "Remark",
    remove: "Delete",
    rebuild: "Rebuild",
    startedAt: "started at {time}",
    launchNew: "Launch New Instance",
    launching: "Launching…",
    macModel: "macOS · copy model",
    winModel: "Windows · native multi-instance",

    winEmptyTitle: "No instances running",
    winEmptySub: "Click \"Launch New Instance\" above to open another WeChat and sign in with a different account",
    runningLabel: "Running Instances",
    instance: "Instance",
    instancesRunning: "{n} instance(s) running",

    cancel: "Cancel",
    confirm: "Confirm",
    createTitle: "Create Copy",
    createHint:
      "Clones the main app and re-signs it into an isolated copy. The name appears in Launchpad / Dock.",
    createPlaceholder: "Enter a name, e.g. Work",
    renameTitle: "Rename \"{name}\"",
    renameHint: "The name shows in Finder / Launchpad / Dock; account data is not affected",
    aliasTitle: "Set Main App Remark",
    aliasHint: "The remark is shown in this tool only and never modifies WeChat",
    instanceRemarkTitle: "Set Instance Remark",
    instanceRemarkHint: "Shown in this tool only; it disappears when the instance closes",
    saveRemark: "Save Remark",
    save: "Save",
    deleteTitle: "Delete \"{name}\"",
    deleteHint:
      "The copy app moves to Trash (recoverable). Account data stays in its isolated container and is not lost.",
    rebuildTitle: "Rebuild \"{name}\"",
    rebuildHint: "Re-clones the current WeChat version. Name and account data are fully kept — no re-login.",
    deleting: "Deleting…",
    creating: "Creating…",
    rebuilding: "Rebuilding…",
    saving: "Saving…",
    defaultCopyName: "Copy {n}",

    progressTitle: "Creating \"{name}\"",
    progressHint: "Cloning and re-signing usually takes about 10 seconds. Please keep the app open.",
    stepClone: "Clone main app (APFS)",
    stepBundle: "Modify Bundle ID",
    stepSign: "Re-sign copy",
    stepDone: "Done",

    rebuildAllTitle: "Rebuilding outdated copies",
    rebuildAllProgress: "Rebuilding \"{label}\" ({done}/{total})",
    rebuildAllFinishing: "Finishing…",

    remarkPlaceholder: "Enter a remark, e.g. Main",

    groupGeneral: "General",
    groupAbout: "About",
    wechatPath: "WeChat Path",
    copyLocation: "Copy Location",
    copyLocationVal: "/Applications (falls back to ~/Applications when not writable)",
    detectMethod: "Detection",
    detectMethodVal: "Registry (Software\\Tencent\\Weixin), falls back to default install dir",
    dataLocation: "Data Location",
    dataLocationVal:
      "WeChat install directory. Instances share the native app and log in independently, no copy files.",
    language: "Language",
    theme: "Theme",
    themeSystem: "Follow System",
    themeLight: "Light",
    themeDark: "Dark",
    version: "Version",
    versionSuffixMac: " (macOS copy management)",
    versionSuffixWin: " (Windows multi-instance)",
    github: "GitHub",
    website: "Website",
    statementVal:
      "Open-source tool. Never modifies WeChat, no injection, no network protocol access; Unofficial third-party tool, not affiliated with Tencent. Please follow the WeChat user agreement.",
    statementWinVal:
      "Open-source tool. Suspends the process and closes the single-instance mutex handle. Never modifies WeChat, no injection, no network protocol access; Unofficial third-party tool, not affiliated with Tencent. Please follow the WeChat user agreement.",
    disclaimer: "Disclaimer",
    openProjectPage: "Open Project Page",
    checkUpdate: "Check for Updates",
    upToDate: "Up to date",
    updateAvailable: "New version v{version} available",
    updateDownload: "Download",
    checking: "Checking…",
    checkFailed: "Check failed (network unavailable)",

    donateTitle: "Support the Author",
    donateSub: "If this tool has helped you, consider buying the author a coffee~",
    donateWechat: "WeChat Pay",
    donateAlipay: "Alipay",

    updateBanner: "VXMOpen v{version} is available — visit the release page to download",

    copyCreated: "Copy \"{name}\" created",
    rebuilt: "Copy rebuilt to the new version, no re-login needed",
    rebuiltAll: "Rebuilt {n} copies",
    opFailed: "Operation failed",
    winNoCopy: "Copy operations are not supported on Windows",
  },
} as const;

export type MessageKey = keyof (typeof messages)["zh"];

/** 翻译：t("outdatedWarning", { n: 3 })；参数替换 {name} 占位符 */
export function t(key: MessageKey, params?: Record<string, string | number>): string {
  const dict = messages[locale.value] as Record<MessageKey, string>;
  let text = dict[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replaceAll(`{${k}}`, String(v));
    }
  }
  return text;
}
