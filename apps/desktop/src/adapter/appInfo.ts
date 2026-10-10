// 關於頁的版本資訊（desktop-about-page design D1）：對 Tauri 命令 `app_info` 的單行委派。
// 引擎版號與技能檔版本只有 Rust 端知道，前端不自行拼湊。
import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  appVersion: string;
  /** speclink-core 的 crate 版本。 */
  engineVersion: string;
  /** 顯示文字：macOS／Windows／Linux。 */
  platform: string;
  /** 顯示文字：arm64／x64。 */
  arch: string;
  /** 技能檔版本（ASSET_VERSION）。 */
  assetsVersion: string;
}

export const loadAppInfo = (): Promise<AppInfo> => invoke<AppInfo>("app_info");
