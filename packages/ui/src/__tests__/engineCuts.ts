// 前端測試共用：以記錄原文加手寫的引擎形狀 `cuts` 組成 `DiscussionDocument`（discussion-cuts-from-engine）。
// 刀由引擎（speclink-core discuss::record_cuts）解析，前端不再判定——測試照引擎會回的值手寫，不在前端重算。
import type { DiscussionCut, DiscussionDocument } from "../adapter";

/** 一把引擎形狀的刀：`line` 為該刀列點去掉 `- ` 的首行原文，其餘欄位缺席時取引擎的空值。 */
export function engineCut(line: string, fields: Partial<Omit<DiscussionCut, "line">> & { cut: string }): DiscussionCut {
  return { title: "", note: "", changes: [], changesInBold: false, unpromoted: [], ...fields, line };
}

/** 記錄原文＋其刀。 */
export function discussionDoc(content: string, cuts: DiscussionCut[] = []): DiscussionDocument {
  return { content, cuts };
}
