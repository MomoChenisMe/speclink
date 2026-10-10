// 專案層檔案系統動作（design D5）：Tauri command 與剪貼簿的接線面。錯誤字串為
// i18n 鍵（fs.noTerminal／fs.noEditor／fs.dirMissing）或 opener／剪貼簿的原文。
// store 只依賴此介面，測試以假 adapter 注入。
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import type { DocumentRef } from "@speclink/ui";

import type { WorkspaceLocator } from "../session";

/** 文件在磁碟上的位置（core 回報）：絕對路徑，與相對所在 checkout 根的路徑（正斜線；
 * 「開啟 ▾」首列與失敗 toast 的主詞）。 */
export interface DocumentLocation {
  path: string;
  relative: string;
}

export interface FsActionsAdapter {
  reveal: (path: string) => Promise<void>;
  /** 以系統預設 App 開啟文件（只收 .md，其餘回 fs.notDocument）。 */
  openDefault: (path: string) => Promise<void>;
  openTerminal: (path: string) => Promise<void>;
  openEditor: (path: string) => Promise<void>;
  /** 寫入剪貼簿（路徑或文件原文）。 */
  copyText: (text: string) => Promise<void>;
  /** 詳情頁文件的磁碟位置：core 以讀文件同一套解析（worktree 副本、spec 目錄名、已封存
   * 討論的日期檔名、封存目錄）；找不到為 null。 */
  locate: (root: string, doc: DocumentRef) => Promise<DocumentLocation | null>;
}

export function tauriFsActionsAdapter(): FsActionsAdapter {
  return {
    reveal: (path) => invoke<void>("reveal_in_folder", { path }),
    openDefault: (path) => invoke<void>("open_path", { path }),
    openTerminal: (path) => invoke<void>("open_in_terminal", { path }),
    openEditor: (path) => invoke<void>("open_in_editor", { path }),
    copyText: (text) => writeText(text),
    locate: (root, doc) => {
      if ("discussion" in doc) return invoke<DocumentLocation | null>("discussion_location", { root, slug: doc.discussion });
      if ("spec" in doc) return invoke<DocumentLocation | null>("spec_location", { root, capability: doc.spec });
      if ("archived" in doc) {
        return invoke<DocumentLocation | null>("archived_document_location", {
          root,
          datedName: doc.archived,
          artifact: doc.artifact,
        });
      }
      return invoke<DocumentLocation | null>("document_location", { root, change: doc.change, artifact: doc.artifact });
    },
  };
}

/** 檔案系統動作作用的本機目錄：local 為專案根、remote 為 checkout；remote 無 checkout 時為 null。 */
export function projectDirOf(locator: WorkspaceLocator): string | null {
  return locator.kind === "local" ? locator.root : (locator.checkoutRoot ?? null);
}
