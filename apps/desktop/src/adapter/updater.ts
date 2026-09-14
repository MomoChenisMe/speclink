// updater 接線面（design D6）：Tauri 殼對 tauri-plugin-updater／plugin-process 的
// 單行委派，加上主視窗焦點事件的委派（前景重檢的訊號；update-recheck-on-focus
// design D1）。store 只依賴此介面，測試以假 adapter 注入。
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** app 自身版本（設定頁軟體更新卡的常駐現版號）。 */
export const appVersion = (): Promise<string> => getVersion();

/** check 找到的新版：版本號＋下載並套用（簽章驗證由 plugin 內建，失敗即 reject）。 */
export interface PendingUpdate {
  version: string;
  downloadAndInstall: () => Promise<void>;
}

export interface UpdaterAdapter {
  /** 檢查更新端點；回 null＝已是最新。離線／端點不可達時 reject。 */
  check: () => Promise<PendingUpdate | null>;
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
      const update = await check();
      if (!update) return null;
      return {
        version: update.version,
        downloadAndInstall: () => update.downloadAndInstall(),
      };
    },
    relaunch,
    // async：getCurrentWindow() 在非 Tauri 環境是同步拋錯，包成 async 讓它一律
    // 變成 reject，呼叫端只靠 .catch 即可靜默。
    onFocusChanged: async (handler) =>
      getCurrentWindow().onFocusChanged(({ payload }) => handler(payload)),
  };
}
