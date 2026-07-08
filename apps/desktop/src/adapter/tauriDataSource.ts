import { invoke } from "@tauri-apps/api/core";
import type {
  SpeclinkDataSource,
  CardKind,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  DiscussionLists,
  StatusReport,
  Verb,
} from "@speclink/ui";

// core-backed adapter：以 Tauri invoke 呼叫 speclink-desktop 的 #[command]，
// 實作 @speclink/ui 的 SpeclinkDataSource 介面。command 名與參數對應 src-tauri/src/lib.rs。
export function createTauriDataSource(): SpeclinkDataSource {
  return {
    async listChanges(): Promise<ChangeItem[]> {
      const r = await invoke<{ changes: ChangeItem[] }>("list_changes");
      return r.changes;
    },
    async listSpecs(): Promise<SpecItem[]> {
      const r = await invoke<{ specs: SpecItem[] }>("list_specs");
      return r.specs;
    },
    async listArchived(): Promise<ArchivedItem[]> {
      const r = await invoke<{ archived: ArchivedItem[] }>("archived_changes");
      return r.archived;
    },
    async status(change: string): Promise<StatusReport> {
      return await invoke<StatusReport>("status", { change });
    },
    async getDocument(change: string, artifact: string): Promise<string | null> {
      return await invoke<string | null>("document", { change, artifact });
    },
    async getSpecDocument(capability: string): Promise<string | null> {
      return await invoke<string | null>("spec_document", { capability });
    },
    async changeCapabilities(change: string): Promise<string[]> {
      return await invoke<string[]>("change_capabilities", { change });
    },
    async changeMeta(change: string) {
      return await invoke<import("@speclink/ui").ChangeMetaInfo | null>("change_meta", { change });
    },
    async deleteChange(change: string): Promise<void> {
      await invoke("delete_change", { change });
    },
    async setTaskDone(change: string, ordinal: number, done: boolean): Promise<void> {
      await invoke("set_task_done", { change, ordinal, done });
    },
    async moveTask(change: string, from: number, to: number, before?: boolean): Promise<void> {
      await invoke("move_task", { change, from, to, before: before ?? null });
    },
    async runVerb(verb: Verb, change: string): Promise<unknown> {
      // 動詞 command 與動詞同名（validate/analyze/archive）。
      return await invoke(verb, { change });
    },
    async getArchivedDocument(datedName: string, artifact: string): Promise<string | null> {
      return await invoke<string | null>("archived_document", { datedName, artifact });
    },
    async archivedCapabilities(datedName: string): Promise<string[]> {
      return await invoke<string[]>("archived_capabilities", { datedName });
    },
    async listDiscussions(): Promise<DiscussionLists> {
      return await invoke<DiscussionLists>("list_discussions");
    },
    async getDiscussionDocument(slug: string): Promise<string | null> {
      return await invoke<string | null>("discussion_document", { slug });
    },
    async promoteDiscussion(slug: string, name?: string): Promise<{ change: string }> {
      return await invoke<{ change: string }>("promote_discussion", { slug, name: name ?? null });
    },
    async archiveDiscussion(slug: string): Promise<void> {
      await invoke("archive_discussion", { slug });
    },
    async reorderCard(
      kind: CardKind,
      id: string,
      prevId: string | null,
      nextId: string | null,
    ): Promise<void> {
      await invoke("reorder_card", { kind, id, prevId, nextId });
    },
  };
}
