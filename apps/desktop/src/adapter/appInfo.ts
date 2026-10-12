// 關於頁的版本資訊（desktop-about-page design D1）：對 Tauri 命令 `app_info` 的單行委派。
// 引擎版號只有 Rust 端知道，前端不自行拼湊。
import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  appVersion: string;
  /** 引擎版號（ASSET_VERSION，與 CLI `--version` 的 engine 同一個值）。 */
  engineVersion: string;
  /** 顯示文字：macOS／Windows／Linux。 */
  platform: string;
  /** 顯示文字：arm64／x64。 */
  arch: string;
}

export const loadAppInfo = (): Promise<AppInfo> => invoke<AppInfo>("app_info");
