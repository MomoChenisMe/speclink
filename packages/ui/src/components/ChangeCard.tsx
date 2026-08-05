import { AlertTriangle, Archive, FileText, GitBranch, MessageSquareText, RefreshCw, Undo2 } from "lucide-react";

import type { ChangeItem, SearchHit } from "../adapter";
import { useI18n } from "../i18n";
import { changeStage } from "../stage";
import { SEMANTIC_TONE } from "../tone";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader } from "./ui/card";
import { CardNameRow } from "./CardNameRow";
import { HighlightText } from "./HighlightText";
import { REVIEW_ICON, REVIEW_LABEL_KEY, REVIEW_TONE, type ReviewBadgeStatus } from "./reviewStyle";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

export interface ChangeCardProps {
  change: ChangeItem;
  onOpen?: (name: string) => void;
  /** 封存請求（僅「已就緒」階段的卡片顯示封存鈕）。 */
  onArchive?: (name: string) => void;
  /** 退回提案中請求（僅派生「進行中」的卡片顯示退回鈕；未提供時不渲染）。 */
  onRevert?: (name: string) => void;
  /** 進度條填色 class（看板依階段配色）。 */
  barClass?: string;
  /** 搜尋字串（design D7）：名稱子字串命中時高亮命中原文。 */
  highlight?: string;
  /** 全文命中（design D6）：卡身呈 snippet 行（artifact 名＋前後文）。 */
  hit?: SearchHit;
}

/** 看板卡片（極簡）：名稱＋複製鈕＋進度。點卡片開詳情；動作歸詳情抽屜，僅 ready 卡有封存。 */
export function ChangeCard({
  change,
  onOpen,
  onArchive,
  onRevert,
  barClass = "bg-primary",
  highlight,
  hit,
}: ChangeCardProps) {
  const { t } = useI18n();
  const pct =
    change.totalTasks > 0 ? Math.round((change.completedTasks / change.totalTasks) * 100) : 0;
  const stage = changeStage(change);
  return (
    <TooltipProvider>
    <Card
      data-change={change.name}
      className="group cursor-pointer transition-[border-color,box-shadow] hover:border-primary/60 hover:shadow-md"
      onClick={() => onOpen?.(change.name)}
    >
      <CardHeader className="p-3 flex-row items-start gap-1.5">
        <CardNameRow text={change.name} copyLabel={t("common.copyName")} highlight={highlight} />
        {change.createdBy && (
          <Tooltip>
            <TooltipTrigger asChild>
              <span
                aria-label={change.createdBy}
                className="inline-flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-muted text-[9px] font-bold text-muted-foreground"
              >
                {change.createdBy.charAt(0).toUpperCase()}
              </span>
            </TooltipTrigger>
            <TooltipContent>{change.createdBy}</TooltipContent>
          </Tooltip>
        )}
        {/* 審查章（spec「看板卡片的審查標示」）：行內小章＋tooltip 狀態詞，
            不加文字列維持卡片極簡；none 無任何審查元素。 */}
        {change.reviewStatus && change.reviewStatus !== "none" && (() => {
          const status: ReviewBadgeStatus = change.reviewStatus;
          const labelKey = REVIEW_LABEL_KEY[status];
          return (
            <Tooltip>
              <TooltipTrigger asChild>
                <span aria-label={t(labelKey)} className={`shrink-0 ${REVIEW_TONE[status]}`}>
                  {REVIEW_ICON[status]}
                </span>
              </TooltipTrigger>
              <TooltipContent>{t(labelKey)}</TooltipContent>
            </Tooltip>
          );
        })()}
        {(change.fromDiscussions ?? []).length > 0 && (
          <Tooltip>
            <TooltipTrigger asChild>
              <span aria-label={t("card.fromDiscussion")} className="shrink-0 text-primary/60">
                <MessageSquareText className="h-3.5 w-3.5" />
              </span>
            </TooltipTrigger>
            <TooltipContent>
              {t("card.fromDiscussionTitle").replace("{name}", (change.fromDiscussions ?? []).join(", "))}
            </TooltipContent>
          </Tooltip>
        )}
        {change.worktree && (
          <Tooltip>
            <TooltipTrigger asChild>
              {/* worktree 掛著＝工作正於副本進行中，屬狀態非靜態 metadata。 */}
              <span
                aria-label={t("card.worktree")}
                className={`shrink-0 ${SEMANTIC_TONE.inProgress}`}
              >
                <GitBranch className="h-3.5 w-3.5" />
              </span>
            </TooltipTrigger>
            <TooltipContent>
              {t("card.worktreeTitle").replace("{branch}", change.worktree.branch)}
            </TooltipContent>
          </Tooltip>
        )}
        {(change.restaleFrom ?? []).length > 0 && (
          <Tooltip>
            <TooltipTrigger asChild>
              <span aria-label={t("card.restale")} className={`shrink-0 ${SEMANTIC_TONE.warning}`}>
                <RefreshCw className="h-3.5 w-3.5" />
              </span>
            </TooltipTrigger>
            <TooltipContent>
              {t("card.restaleTitle").replace("{name}", (change.restaleFrom ?? []).join(", "))}
            </TooltipContent>
          </Tooltip>
        )}
        {/* 最小 invalid 標記（design 決策四）：metadata 損壞卡照常顯示，
            操作由引擎錯誤拒絕；tooltip 載解析原因供修復。 */}
        {change.metaError && (
          <Tooltip>
            <TooltipTrigger asChild>
              <span aria-label={t("card.invalidMeta")} className="shrink-0 text-destructive">
                <AlertTriangle className="h-3.5 w-3.5" />
              </span>
            </TooltipTrigger>
            <TooltipContent>
              {t("card.invalidMetaTitle").replace("{reason}", change.metaError)}
            </TooltipContent>
          </Tooltip>
        )}
      </CardHeader>
      <CardContent className="p-3 pt-0 gap-2">
        {/* 描述列（anatomy 三列骨架）：proposal Why 首句一行截斷；null 時整列缺席。 */}
        {change.whyExcerpt && (
          <div data-desc className="truncate text-[11px] text-muted-foreground">
            {change.whyExcerpt}
          </div>
        )}
        <div className="flex items-center gap-2">
          <div className="flex-1 h-1.5 rounded-full bg-muted overflow-hidden">
            <div className={`h-full rounded-full transition-all ${barClass}`} style={{ width: `${pct}%` }} />
          </div>
          <span className="text-xs text-muted-foreground tabular-nums">
            {change.completedTasks}/{change.totalTasks}
          </span>
        </div>
        {/* stopPropagation 掛在按鈕自身而非整列：鈕旁空白仍冒泡開卡片詳情。 */}
        {stage === "ready" && (
          <div>
            <Button
              variant="outline"
              size="sm"
              className="h-6 gap-1 px-2 text-xs"
              onClick={(e) => {
                e.stopPropagation();
                onArchive?.(change.name);
              }}
            >
              <Archive className="h-3 w-3" /> {t("common.archive")}
            </Button>
          </div>
        )}
        {/* 退回提案中（樣式沿討論卡封存鈕）：僅派生進行中呈現；點擊交宿主先確認,
            UI 不預判守門——引擎是唯一裁決點。 */}
        {stage === "in-progress" && onRevert && (
          <div>
            <Button
              variant="outline"
              size="sm"
              className="h-6 gap-1 px-2 text-xs"
              onClick={(e) => {
                e.stopPropagation();
                onRevert(change.name);
              }}
            >
              <Undo2 className="h-3 w-3" /> {t("common.revert")}
            </Button>
          </div>
        )}
        {/* 全文命中 snippet 行（design D6/D7）：哪個 artifact 命中＋前後文＋命中高亮。 */}
        {hit && (
          <div
            data-snippet
            className="flex items-start gap-1 border-t border-border/40 pt-1.5 text-[11px] leading-snug text-muted-foreground"
          >
            <FileText className="mt-0.5 h-3 w-3 shrink-0" />
            <span className="min-w-0">
              <span className="font-mono">{hit.artifact}</span>{" "}
              <HighlightText text={hit.snippet} query={highlight} />
            </span>
          </div>
        )}
      </CardContent>
    </Card>
    </TooltipProvider>
  );
}
