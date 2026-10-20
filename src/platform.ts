// 平台判定——单一事实来源，App / InstancesView / SettingsView 统一引用。
// 浏览器预览可加 ?win 参数查看 Windows 布局；生产环境无影响。
export const isWindows =
  navigator.userAgent.includes("Windows") ||
  new URLSearchParams(location.search).has("win");
