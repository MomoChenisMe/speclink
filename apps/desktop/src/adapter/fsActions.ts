// 專案層檔案系統動作（design D5）：Tauri command 與剪貼簿的接線面。錯誤字串為
// i18n 鍵（fs.noTerminal／fs.noEditor／fs.dirMissing）或 opener／剪貼簿的原文。
// store 只依賴此介面，測試以假 adapter 注入。
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

import type { WorkspaceLocator } from "../session";

export interface FsActionsAdapter {
  reveal: (path: string) => Promise<void>;
  openTerminal: (path: string) => Promise<void>;
  openEditor: (path: string) => Promise<void>;
  copyPath: (path: string) => Promise<void>;
}

export function tauriFsActionsAdapter(): FsActionsAdapter {
  return {
    reveal: (path) => invoke<void>("reveal_in_folder", { path }),
    openTerminal: (path) => invoke<void>("open_in_terminal", { path }),
    openEditor: (path) => invoke<void>("open_in_editor", { path }),
    copyPath: (path) => writeText(path),
  };
}

/** 檔案系統動作作用的本機目錄：local 為專案根、remote 為 checkout；remote 無 checkout 時為 null。 */
export function projectDirOf(locator: WorkspaceLocator): string | null {
  return locator.kind === "local" ? locator.root : (locator.checkoutRoot ?? null);
}
