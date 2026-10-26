// 自动更新下载通道：Tauri 内走 updater 插件；浏览器预览环境全部静默降级。
// 「有没有新版」的判断不走这里——那由 appStore 的官网/GitHub 版本号比对负责，
// 本模块只在用户点「立即更新」后才连 updater endpoint 下载。
// 通道与签名公钥配置见 src-tauri/tauri.conf.json 的 plugins.updater。
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export const updaterAvailable =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export interface DownloadProgress {
  /** 0-100，未知总大小时为 -1 */
  percent: number;
}

/** 下载并安装更新，完成后自动重启应用。任何异常向上抛出由调用方提示降级 */
export async function installUpdate(onProgress: (p: DownloadProgress) => void): Promise<void> {
  if (!updaterAvailable) throw new Error("updater unavailable");
  const update: Update | null = await check({ timeout: 8000 });
  if (!update) throw new Error("no update available");
  let total = 0;
  let received = 0;
  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        total = event.data.contentLength ?? 0;
        break;
      case "Progress":
        received += event.data.chunkLength;
        onProgress({ percent: total > 0 ? Math.min(99, Math.round((received / total) * 100)) : -1 });
        break;
      case "Finished":
        onProgress({ percent: 100 });
        break;
    }
  });
  await relaunch();
}
