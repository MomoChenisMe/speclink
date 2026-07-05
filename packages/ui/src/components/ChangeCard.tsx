import { useState } from "react";
import { Archive, Check, Copy } from "lucide-react";

import type { ChangeItem } from "../adapter";
import { changeStage } from "../stage";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader } from "./ui/card";

export interface ChangeCardProps {
  change: ChangeItem;
  onOpen?: (name: string) => void;
  /** 封存請求（僅「已就緒」階段的卡片顯示封存鈕）。 */
  onArchive?: (name: string) => void;
  /** 進度條填色 class（看板依階段配色）。 */
  barClass?: string;
}

/** 看板卡片（極簡）：名稱＋複製鈕＋進度。點卡片開詳情；動作歸詳情抽屜，僅 ready 卡有封存。 */
export function ChangeCard({ change, onOpen, onArchive, barClass = "bg-primary" }: ChangeCardProps) {
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
    <Card
      data-change={change.name}
      className="group cursor-pointer transition-[border-color,box-shadow] hover:border-primary/60 hover:shadow-md"
      onClick={() => onOpen?.(change.name)}
    >
      <CardHeader className="p-3 flex-row items-start gap-1.5">
        <span className="font-semibold text-sm leading-tight min-w-0 flex-1">{change.name}</span>
        <button
          type="button"
          aria-label="複製名稱"
          className={`shrink-0 text-muted-foreground hover:text-foreground transition-opacity ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
          onClick={copyName}
        >
          {copied ? <Check className="h-3.5 w-3.5 text-primary" /> : <Copy className="h-3.5 w-3.5" />}
        </button>
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
              <Archive className="h-3 w-3" /> 封存
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
