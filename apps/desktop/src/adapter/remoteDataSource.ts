import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { toRevertError } from "@speclink/ui";
import type {
  SpeclinkDataSource,
  CardKind,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  DiscussionItem,
  DiscussionLists,
  SearchHit,
  StatusReport,
  Verb,
} from "@speclink/ui";

import type { InvokeFn } from "../session";

// remote adapter（remote-data-source 決策 7）：SpeclinkDataSource 對 remote_*
// command 的薄 invoke 包裝——每擊帶 locator 識別（connectionId＋project＋repo），
// 所有 HTTP、token、重試邏輯在 Rust。server 純讀取面直達；其餘不支援方法
// （決策 1 (c)：server 無端點）回拒絕錯誤、不打任何 invoke——server 缺什麼
// 就停用什麼，不在 client 偽造。

/** (c) 類操作的拒絕（與 Rust remote::unsupported 同語意、同繁中措辭）。 */
function unsupported(operation: string): Promise<never> {
  return Promise.reject(new Error(`此 server 尚未提供「${operation}」——功能已停用`));
}

/** server 的討論 payload：欄位鏡射 protocol DiscussionInfo（camelCase）。 */
interface RemoteDiscussionInfo {
  slug: string;
  topic: string;
  status: string;
  rounds: number;
  created: string;
  createdBy?: string | null;
}

/** server 不外露 promotedTo——以空清單補齊 UI 必填欄位（資料缺口，非偽造 affordance）。 */
function toDiscussionItem(info: RemoteDiscussionInfo): DiscussionItem {
  return { ...info, promotedTo: [] };
}

export function createRemoteDataSource(
  connectionId: string,
  project: string,
  repo: string,
  invoke: InvokeFn = tauriInvoke as InvokeFn,
): SpeclinkDataSource {
  const locator = { connectionId, project, repo };
  return {
    async listChanges(): Promise<ChangeItem[]> {
      const r = await invoke<{ changes: ChangeItem[] }>("remote_list_changes", { ...locator });
      return r.changes;
    },
    async listSpecs(): Promise<SpecItem[]> {
      const r = await invoke<{ specs: SpecItem[] }>("remote_list_specs", { ...locator });
      return r.specs;
    },
    async listArchived(): Promise<ArchivedItem[]> {
      const r = await invoke<{ archived: ArchivedItem[] }>("remote_list_archived", {
        ...locator,
      });
      return r.archived;
    },
    async status(change: string): Promise<StatusReport> {
      return await invoke<StatusReport>("remote_status", { ...locator, change });
    },
    async getDocument(change: string, artifact: string): Promise<string | null> {
      return await invoke<string | null>("remote_document", { ...locator, change, artifact });
    },
    async getSpecDocument(capability: string): Promise<string | null> {
      return await invoke<string | null>("remote_spec_document", {
        ...locator,
        capability,
      });
    },
    async searchWorkspace(query: string): Promise<SearchHit[]> {
      const r = await invoke<{ hits: SearchHit[] }>("remote_search_workspace", {
        ...locator,
        query,
      });
      return r.hits;
    },
    changeCapabilities(): Promise<string[]> {
      return unsupported("capability 清單");
    },
    changeMeta(): Promise<import("@speclink/ui").ChangeMetaInfo | null> {
      return unsupported("change 詮釋資料");
    },
    async deleteChange(change: string): Promise<void> {
      // 決策 3：桌面 remote 刪除固定帶 force=true（與本地無 guard 直刪同模式，
      // 確認對話框在 UI 層）；server 端仍執行 discard 全語意（unlink＋原子刪除）。
      await invoke("remote_delete_change", { ...locator, change, force: true });
    },
    async revertChangeToProposed(change: string): Promise<void> {
      // 守門 409 的證據由 Rust bridge 轉為與本地同形狀的 JSON 錯誤字串——
      // 同一 toRevertError 轉譯,App 的守門對話框單一消費入口。
      try {
        await invoke("remote_revert_change_to_proposed", { ...locator, change });
      } catch (e) {
        throw toRevertError(e);
      }
    },
    async setTaskDone(change: string, task: string, done: boolean): Promise<void> {
      await invoke("remote_set_task_done", { ...locator, change, task, done });
    },
    async setAllTasks(change: string, done: boolean): Promise<void> {
      await invoke("remote_set_all_tasks", { ...locator, change, done });
    },
    async moveTask(change: string, from: number, to: number, before?: boolean): Promise<void> {
      await invoke("remote_move_task", { ...locator, change, from, to, before: before ?? null });
    },
    async runVerb(verb: Verb, change: string): Promise<unknown> {
      // 動詞 command 對 remote_* 同名映射（validate/analyze/archive 皆直達）。
      if (verb === "validate") return await invoke("remote_validate", { ...locator, change });
      if (verb === "analyze") return await invoke("remote_analyze", { ...locator, change });
      return await invoke("remote_archive", { ...locator, change });
    },
    async getArchivedDocument(datedName: string, artifact: string): Promise<string | null> {
      return await invoke<string | null>("remote_archived_document", {
        ...locator,
        datedName,
        artifact,
      });
    },
    async archivedCapabilities(datedName: string): Promise<string[]> {
      return await invoke<string[]>("remote_archived_capabilities", {
        ...locator,
        datedName,
      });
    },
    async listDiscussions(): Promise<DiscussionLists> {
      const r = await invoke<{ active: RemoteDiscussionInfo[]; archived: RemoteDiscussionInfo[] }>(
        "remote_list_discussions",
        { ...locator },
      );
      return { active: r.active.map(toDiscussionItem), archived: r.archived.map(toDiscussionItem) };
    },
    async getDiscussionDocument(slug: string): Promise<string | null> {
      return await invoke<string | null>("remote_discussion_document", { ...locator, slug });
    },
    async promoteDiscussion(slug: string, name?: string): Promise<{ change: string }> {
      return await invoke<{ change: string }>("remote_promote_discussion", {
        ...locator,
        slug,
        name: name ?? null,
      });
    },
    async archiveDiscussion(slug: string): Promise<void> {
      await invoke("remote_archive_discussion", { ...locator, slug });
    },
    async reorderCard(
      kind: CardKind,
      id: string,
      prevId: string | null,
      nextId: string | null,
    ): Promise<void> {
      await invoke("remote_reorder_card", { ...locator, kind, id, prevId, nextId });
    },
  };
}
