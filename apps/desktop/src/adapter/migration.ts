import { invoke as tauriInvoke } from "@tauri-apps/api/core";

import type { InvokeFn } from "../session";

export interface MigrationResult {
  report: {
    projectRevision: number;
    documents: unknown[];
  };
  backupPath: string;
  checkoutRoot: string;
}

export interface RemoteAdoptionResult {
  backupPath: string;
  checkoutRoot: string;
}

export interface MigrationAdapter {
  migrate(
    root: string,
    connectionId: string,
    project: string,
    repo: string,
  ): Promise<MigrationResult>;
  /** 並存資料夾採用 server：只保留本機備份，不組 Bundle、不呼叫 import。 */
  adoptRemote(root: string): Promise<RemoteAdoptionResult>;
}

/** 桌面遷移命令的薄 adapter；Bundle、import 與本機轉換順序皆由 Rust 保證。 */
export function createMigrationAdapter(
  invoke: InvokeFn = tauriInvoke as InvokeFn,
): MigrationAdapter {
  return {
    migrate: (root, connectionId, project, repo) =>
      invoke("migrate_workspace", { root, connectionId, project, repo }),
    adoptRemote: (root) => invoke("adopt_remote_workspace", { root }),
  };
}
