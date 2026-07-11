import { useState } from "react";
import { Archive, Check, Copy, FileText, MessageSquareText, RefreshCw } from "lucide-react";

import type { ChangeItem, SearchHit } from "../adapter";
import { useI18n } from "../i18n";
import { changeStage } from "../stage";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader } from "./ui/card";
import { HighlightText } from "./HighlightText";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

export interface ChangeCardProps {
  change: ChangeItem;
  onOpen?: (name: string) => void;
  /** 封存請求（僅「已就緒」階段的卡片顯示封存鈕）。 */
  onArchive?: (name: string) => void;
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
  barClass = "bg-primary",
  highlight,
  hit,
}: ChangeCardProps) {
  const { t } = useI18n();
  const pct =
    change.totalTasks > 0 ? Math.round((change.completedTasks / change.totalTasks) * 100) : 0;
  const stage = changeStage(change);
  const [copied, setCopied] = useState(false);
  const copyName = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard?.writeText(change.name);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
  return (
    <TooltipProvider>
    <Card
      data-change={change.name}
      className="group cursor-pointer transition-[border-color,box-shadow] hover:border-primary/60 hover:shadow-md"
      onClick={() => onOpen?.(change.name)}
    >
      <CardHeader className="p-3 flex-row items-start gap-1.5">
        {/* 複製鈕行內尾隨（design D4 複製鈕位置規則）：緊跟名稱最後一個字元後
            流動，不以 flex 推至卡片右緣。 */}
        <span className="font-semibold text-sm leading-tight min-w-0 flex-1">
          <HighlightText text={change.name} query={highlight} />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            aria-label={t("common.copyName")}
            className={`ml-1 inline-flex h-4 w-4 align-text-bottom text-muted-foreground hover:text-foreground transition-opacity ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
            onClick={copyName}
          >
            {copied ? <Check className="h-3 w-3 text-primary" /> : <Copy className="h-3 w-3" />}
          </Button>
        </span>
        {change.createdBy && (
          <Tooltip>
            <TooltipTrigger asChild>
              <span
                aria-label={change.createdBy}
                className="inline-flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary text-[9px] font-bold text-primary-foreground"
              >
                {change.createdBy.charAt(0).toUpperCase()}
              </span>
            </TooltipTrigger>
            <TooltipContent>{change.createdBy}</TooltipContent>
          </Tooltip>
        )}
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
        {(change.restaleFrom ?? []).length > 0 && (
          <Tooltip>
            <TooltipTrigger asChild>
              <span aria-label={t("card.restale")} className="shrink-0 text-amber-500">
                <RefreshCw className="h-3.5 w-3.5" />
              </span>
            </TooltipTrigger>
            <TooltipContent>
              {t("card.restaleTitle").replace("{name}", (change.restaleFrom ?? []).join(", "))}
            </TooltipContent>
          </Tooltip>
        )}
      </CardHeader>
      <CardContent className="p-3 pt-0 gap-2">
        <div className="flex items-center gap-2">
          <div className="flex-1 h-1.5 rounded-full bg-muted overflow-hidden">
            <div className={`h-full rounded-full transition-all ${barClass}`} style={{ width: `${pct}%` }} />
          </div>
          <span className="text-xs text-muted-foreground tabular-nums">
            {change.completedTasks}/{change.totalTasks}
          </span>
        </div>
        {stage === "ready" && (
          <div onClick={(e) => e.stopPropagation()}>
            <Button
              variant="outline"
              size="sm"
              className="h-6 gap-1 px-2 text-xs"
              onClick={() => onArchive?.(change.name)}
            >
              <Archive className="h-3 w-3" /> {t("common.archive")}
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
