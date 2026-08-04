import { useState } from "react";

import { useI18n } from "../i18n";
import { Button } from "./ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

/** 來源連結項：slug 為識別錨點（openspec/LANGUAGE.md 受控例外），topic 為描述副標（同源變更等無副標項缺席）。 */
export interface SourceLinkItem {
  slug: string;
  topic?: string;
}

/**
 * 來源連結籤：變更詳情抽屜與已封存抽屜共用的單一實作。
 *
 * 籤面以 slug 直出（等寬字型）——slug 短而穩定，籤形一致；topic 是自由文字整句話，
 * 降為主題化提示（slug＋topic 兩行），不再上籤面（spec「抽屜標頭標記受寬度約束且
 * 抽屜不產生水平捲軸」）。寬度上限 max-w-[140px] 截斷兜底超長 slug；截斷必須落在
 * 內層區塊子項（text-overflow 不作用於 flex 容器本身）。
 */
export function SourceDiscussionChip({
  slug,
  topic,
  onClick,
}: SourceLinkItem & {
  onClick?: () => void;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto min-w-0 max-w-[140px] rounded-full bg-primary/10 px-2 py-0.5 font-mono font-medium text-primary hover:bg-primary/20 hover:text-primary"
          onClick={onClick}
        >
          <span data-source-discussion-label className="truncate">
            {slug}
          </span>
        </Button>
      </TooltipTrigger>
      <TooltipContent className="max-w-72">
        <div className="truncate font-mono">{slug}</div>
        {topic && <div className="line-clamp-3">{topic}</div>}
      </TooltipContent>
    </Tooltip>
  );
}

/**
 * 來源連結列：前綴標籤＋首籤（清單第一份＝出身）＋「+N」溢出浮層。
 *
 * 出身列恆定單行的固定顆數切點（design D3）：固定顯示 1 顆，其餘收進 +N 數字籤，
 * 點擊以 Popover 浮層列出（slug 主行＋topic 副行），點浮層項跳轉並關閉浮層——
 * 與看板卡片「單一討論徽章以出身討論為代表」對稱。「來自」與「同源」兩組共用此列。
 */
export function SourceChipRow({
  label,
  items,
  onOpen,
}: {
  label: string;
  items: SourceLinkItem[];
  onOpen?: (slug: string) => void;
}) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  if (items.length === 0) return null;
  const [first, ...rest] = items;
  return (
    <TooltipProvider delayDuration={0}>
      <span className="inline-flex min-w-0 items-center gap-1">
        <span className="shrink-0">{label}</span>
        <SourceDiscussionChip
          slug={first.slug}
          topic={first.topic}
          onClick={() => onOpen?.(first.slug)}
        />
        {rest.length > 0 && (
          <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                data-source-overflow
                aria-label={t("rdrawer.moreSources").replace("{n}", String(rest.length))}
                className="h-auto shrink-0 rounded-full bg-muted px-2 py-0.5 font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
              >
                +{rest.length}
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-72">
              <div data-source-overflow-list className="flex flex-col">
                {rest.map((it) => (
                  <Button
                    key={it.slug}
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="h-auto justify-start px-2 py-1.5 text-left"
                    onClick={() => {
                      setOpen(false);
                      onOpen?.(it.slug);
                    }}
                  >
                    <span className="flex min-w-0 flex-col items-start">
                      <span className="max-w-full truncate font-mono text-xs font-medium">
                        {it.slug}
                      </span>
                      {it.topic && (
                        <span className="max-w-full truncate text-xs font-normal text-muted-foreground">
                          {it.topic}
                        </span>
                      )}
                    </span>
                  </Button>
                ))}
              </div>
            </PopoverContent>
          </Popover>
        )}
      </span>
    </TooltipProvider>
  );
}
