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
// 所有 HTTP、token、重試邏輯在 Rust。全讀取面直達 server：change 詮釋資料與
// capability 清單以既有 remote_status payload 映射（remote-read-parity design
// D2，不開新 command、不另發請求），舊 server 不送的欄位以缺席呈現、不偽造。

/** server 的討論 payload：欄位鏡射 protocol DiscussionInfo（camelCase）。 */
interface RemoteDiscussionInfo {
  slug: string;
  topic: string;
  status: string;
  rounds: number;
  created: string;
  createdBy?: string | null;
  /** 討論型別（目前唯一值 "improve"）——一般討論缺席。 */
  kind?: string | null;
  /** 已轉出變更名清單（wire promotedTo）——空清單時 server 省略鍵。 */
  promotedTo?: string[];
  /** 結論已寫入與否（wire concluded）——舊 server 不送時缺席＝未知，不補值。 */
  concluded?: boolean;
}

function toDiscussionItem(info: RemoteDiscussionInfo): DiscussionItem {
  return { ...info, promotedTo: info.promotedTo ?? [] };
}

/** remote_status 的整包 payload：StatusReport 疊加 show 組合欄位（wire
 * ChangeStatus 的選填 meta 欄位，舊 server 缺席）。 */
interface RemoteStatusPayload extends StatusReport {
  created?: string | null;
  createdBy?: string | null;
  createdWith?: string | null;
  startedAt?: string | null;
  startedBy?: string | null;
  fromDiscussions?: string[];
  deltaCapabilities?: string[];
}

export function createRemoteDataSource(
  connectionId: string,
  project: string,
  repo: string,
  invoke: InvokeFn = tauriInvoke as InvokeFn,
): SpeclinkDataSource {
  const locator = { connectionId, project, repo };
  // changeCapabilities 與 changeMeta 映射同一份 status payload（spec：不另開
  // 請求）——併發載入共用同一個 in-flight 請求，落定即清、不留 stale 快取。
  const statusInFlight = new Map<string, Promise<RemoteStatusPayload>>();
  function fetchStatus(change: string): Promise<RemoteStatusPayload> {
    const pending = statusInFlight.get(change);
    if (pending) return pending;
    const p = invoke<RemoteStatusPayload>("remote_status", { ...locator, change }).finally(() => {
      statusInFlight.delete(change);
    });
    statusInFlight.set(change, p);
    return p;
  }
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
    async changeCapabilities(change: string): Promise<string[]> {
      const s = await fetchStatus(change);
      return s.deltaCapabilities ?? [];
    },
    async changeMeta(change: string): Promise<import("@speclink/ui").ChangeMetaInfo | null> {
      const s = await fetchStatus(change);
      return {
        schema: s.schemaName,
        created: s.created ?? null,
        createdBy: s.createdBy ?? null,
        createdWith: s.createdWith ?? null,
        fromDiscussions: s.fromDiscussions ?? [],
        startedAt: s.startedAt ?? null,
        startedBy: s.startedBy ?? null,
      };
    },
    async claim(change: string): Promise<void> {
      // 已被他人持有時 Rust 側原樣帶回 server 的 409 訊息（含持有人與建議
      // 動作）——沿既有失敗 toast 呈現，這裡不加工。
      await invoke("remote_claim", { ...locator, change });
    },
    async deleteChange(change: string): Promise<void> {
      // archive-readiness-gating D2 翻案：force=false 與本地 discard 守門語意對齊
      // ——已開工 change 由 server 拒絕（需要強制），拒絕錯誤沿既有 deleteFailed
      // toast 路徑呈現；server 端執行 discard 全語意（守門＋unlink＋原子刪除）。
      await invoke("remote_delete_change", { ...locator, change, force: false });
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
