import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { toRevertError } from "@speclink/ui";
import type {
  SpeclinkDataSource,
  CardKind,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  DiscussionLists,
  SearchHit,
  StatusReport,
  Verb,
} from "@speclink/ui";

import type { InvokeFn } from "../session";

// core-backed adapter：以 Tauri invoke 呼叫 speclink-desktop 的 #[command]，
// 實作 @speclink/ui 的 SpeclinkDataSource 介面。command 名與參數對應 src-tauri/src/lib.rs。
// root 綁入閉包（workspace-session 決策 3/4）：每支 command 顯式帶 root，直通
// desktop-core 的帶路徑函式——Rust 側無 current-root 全域。invoke 可注入以利測試。
export function createTauriDataSource(
  root: string,
  invoke: InvokeFn = tauriInvoke as InvokeFn,
): SpeclinkDataSource {
  return {
    async listChanges(): Promise<ChangeItem[]> {
      const r = await invoke<{ changes: ChangeItem[] }>("list_changes", { root });
      return r.changes;
    },
    async listSpecs(): Promise<SpecItem[]> {
      const r = await invoke<{ specs: SpecItem[] }>("list_specs", { root });
      return r.specs;
    },
    async listArchived(): Promise<ArchivedItem[]> {
      const r = await invoke<{ archived: ArchivedItem[] }>("archived_changes", { root });
      return r.archived;
    },
    async status(change: string): Promise<StatusReport> {
      return await invoke<StatusReport>("status", { root, change });
    },
    async getDocument(change: string, artifact: string): Promise<string | null> {
      return await invoke<string | null>("document", { root, change, artifact });
    },
    async getSpecDocument(capability: string): Promise<string | null> {
      return await invoke<string | null>("spec_document", { root, capability });
    },
    async searchWorkspace(query: string): Promise<SearchHit[]> {
      const r = await invoke<{ hits: SearchHit[] }>("search_workspace", { root, query });
      return r.hits;
    },
    async changeCapabilities(change: string): Promise<string[]> {
      return await invoke<string[]>("change_capabilities", { root, change });
    },
    async changeMeta(change: string) {
      return await invoke<import("@speclink/ui").ChangeMetaInfo | null>("change_meta", {
        root,
        change,
      });
    },
    async revertChangeToProposed(change: string): Promise<void> {
      // 守門拒絕由 desktop core 以 JSON 字串回證據——經共用 toRevertError
      // 轉為 RevertBlockedError,App 的守門對話框單一消費入口。
      try {
        await invoke("revert_change_to_proposed", { root, change });
      } catch (e) {
        throw toRevertError(e);
      }
    },
    async deleteChange(change: string): Promise<void> {
      await invoke("delete_change", { root, change });
    },
    async setTaskDone(change: string, task: string, done: boolean): Promise<void> {
      await invoke("set_task_done", { root, change, task, done });
    },
    async setAllTasks(change: string, done: boolean): Promise<void> {
      await invoke("set_all_tasks", { root, change, done });
    },
    async moveTask(change: string, from: number, to: number, before?: boolean): Promise<void> {
      await invoke("move_task", { root, change, from, to, before: before ?? null });
    },
    async runVerb(verb: Verb, change: string): Promise<unknown> {
      // 動詞 command 與動詞同名（validate/analyze/archive）。
      return await invoke(verb, { root, change });
    },
    async getArchivedDocument(datedName: string, artifact: string): Promise<string | null> {
      return await invoke<string | null>("archived_document", { root, datedName, artifact });
    },
    async archivedCapabilities(datedName: string): Promise<string[]> {
      return await invoke<string[]>("archived_capabilities", { root, datedName });
    },
    async listDiscussions(): Promise<DiscussionLists> {
      return await invoke<DiscussionLists>("list_discussions", { root });
    },
    async getDiscussionDocument(slug: string): Promise<string | null> {
      return await invoke<string | null>("discussion_document", { root, slug });
    },
    async promoteDiscussion(slug: string, name?: string): Promise<{ change: string }> {
      return await invoke<{ change: string }>("promote_discussion", {
        root,
        slug,
        name: name ?? null,
      });
    },
    async archiveDiscussion(slug: string): Promise<void> {
      await invoke("archive_discussion", { root, slug });
    },
    async reorderCard(
      kind: CardKind,
      id: string,
      prevId: string | null,
      nextId: string | null,
    ): Promise<void> {
      await invoke("reorder_card", { root, kind, id, prevId, nextId });
    },
  };
}
