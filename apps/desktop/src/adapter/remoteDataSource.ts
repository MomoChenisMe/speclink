import { invoke as tauriInvoke } from "@tauri-apps/api/core";
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
// 所有 HTTP、token、重試邏輯在 Rust。不支援方法（決策 1 (c)：server 無端點）
// 回拒絕錯誤、不打任何 invoke——server 缺什麼就停用什麼，不在 client 偽造。

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
    listArchived(): Promise<ArchivedItem[]> {
      return unsupported("封存瀏覽");
    },
    async status(change: string): Promise<StatusReport> {
      return await invoke<StatusReport>("remote_status", { ...locator, change });
    },
    async getDocument(change: string, artifact: string): Promise<string | null> {
      return await invoke<string | null>("remote_document", { ...locator, change, artifact });
    },
    getSpecDocument(): Promise<string | null> {
      return unsupported("正典 spec 內文");
    },
    searchWorkspace(): Promise<SearchHit[]> {
      return unsupported("全文搜尋");
    },
    changeCapabilities(): Promise<string[]> {
      return unsupported("capability 清單");
    },
    changeMeta(): Promise<import("@speclink/ui").ChangeMetaInfo | null> {
      return unsupported("change 詮釋資料");
    },
    deleteChange(): Promise<void> {
      return unsupported("刪除變更");
    },
    async setTaskDone(change: string, task: string, done: boolean): Promise<void> {
      await invoke("remote_set_task_done", { ...locator, change, task, done });
    },
    async setAllTasks(change: string, done: boolean): Promise<void> {
      await invoke("remote_set_all_tasks", { ...locator, change, done });
    },
    moveTask(): Promise<void> {
      return unsupported("任務排序");
    },
    async runVerb(verb: Verb, change: string): Promise<unknown> {
      if (verb !== "archive") {
        return unsupported(verb === "validate" ? "validate 動詞" : "analyze 動詞");
      }
      return await invoke("remote_archive", { ...locator, change });
    },
    getArchivedDocument(): Promise<string | null> {
      return unsupported("封存文件");
    },
    archivedCapabilities(): Promise<string[]> {
      return unsupported("封存 capability 清單");
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
    reorderCard(_kind: CardKind): Promise<void> {
      return unsupported("看板排序");
    },
  };
}
