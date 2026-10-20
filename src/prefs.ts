/** 主题偏好：亮色 / 暗色 / 跟随系统，持久化到 localStorage 并应用到 <html data-theme> */
import { ref } from "vue";

export type Theme = "system" | "light" | "dark";

const THEME_KEY = "vxm.theme";

export const theme = ref<Theme>(
  (["system", "light", "dark"] as const).includes(
    localStorage.getItem(THEME_KEY) as Theme,
  )
    ? (localStorage.getItem(THEME_KEY) as Theme)
    : "system",
);

const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

function applyTheme() {
  const resolved =
    theme.value === "system" ? (systemDark.matches ? "dark" : "light") : theme.value;
  document.documentElement.dataset.theme = resolved;
}

export function setTheme(v: Theme) {
  theme.value = v;
  localStorage.setItem(THEME_KEY, v);
  applyTheme();
}

let bound = false;

/** 在应用挂载时调用：立即应用主题并监听系统外观变化 */
export function initTheme() {
  applyTheme();
  if (!bound) {
    bound = true;
    systemDark.addEventListener("change", applyTheme);
  }
}
