// updater 接線面（design D6）：Tauri 殼對 tauri-plugin-updater／plugin-process 的
// 單行委派，加上主視窗焦點事件的委派（前景重檢的訊號；update-recheck-on-focus
// design D1）。store 只依賴此介面，測試以假 adapter 注入。
import { invoke } from "@tauri-apps/api/core";
import { compareVersions } from "../core/whatsNew";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** app 自身版本（設定頁軟體更新卡的常駐現版號）。 */
export const appVersion = (): Promise<string> => getVersion();

/** check 找到的新版：版本號＋下載與安裝兩段（簽章驗證由 plugin 內建，失敗即 reject）。
 * 分兩段是為了「取消」：使用者在下載中取消後，下載完成也不安裝（desktop-app「取消下載回到閒置」）。 */
export interface PendingUpdate {
  version: string;
  download: () => Promise<void>;
  install: () => Promise<void>;
}
export interface ManualUpdate { version: string; manualUrl: string }
export interface DesktopRelease { version: string; repoUrl: string; releaseUrl: string; publishedAt: string }
export type ReleaseRepository = "fork" | "upstream";
export const desktopRelease = (repo: ReleaseRepository): Promise<DesktopRelease | null> => invoke("desktop_release", {repo});

export interface UpdaterAdapter {
  /** 檢查更新端點；回 null＝已是最新。離線／端點不可達時 reject。 */
  check: () => Promise<PendingUpdate | ManualUpdate | null>;
  /** 套用更新後重啟 app。 */
  relaunch: () => Promise<void>;
  /**
   * 訂閱主視窗焦點變化（前景重檢的訊號；design D1）：handler 收到 true＝取得焦點。
   * 可選——既有假 adapter 與非 Tauri 環境不提供時，app 不訂閱、不重檢。
   * 回傳取消訂閱函式的 Promise。
   */
  onFocusChanged?: (handler: (focused: boolean) => void) => Promise<() => void>;
}

export function tauriUpdaterAdapter(): UpdaterAdapter {
  return {
    async check() {
      let update: {id: string; version: string} | null;
      try { update = await invoke("updater_check"); }
      catch (error) {
        const failure = error as {code?: string; httpStatus?: number};
        if (failure?.httpStatus !== 404 && failure?.code !== "updater-unavailable" && failure?.code !== "parse") throw error;
        const release = await desktopRelease("fork");
        if (!release) throw error;
        const installed = await appVersion();
        if (compareVersions(release.version, installed) <= 0) return null;
        return {version:release.version, manualUrl:release.releaseUrl};
      }
      if (!update) return null;
      return {
        version: update.version,
        download: () => invoke<void>("updater_download", { id: update.id }),
        install: () => invoke<void>("updater_install", { id: update.id }),
      };
    },
    relaunch,
    // async：getCurrentWindow() 在非 Tauri 環境是同步拋錯，包成 async 讓它一律
    // 變成 reject，呼叫端只靠 .catch 即可靜默。
    onFocusChanged: async (handler) =>
      getCurrentWindow().onFocusChanged(({ payload }) => handler(payload)),
  };
}
