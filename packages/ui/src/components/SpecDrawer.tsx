import { useEffect, useRef, useState } from "react";
import { Check, Copy, Maximize2, Minimize2 } from "lucide-react";

import type { ArchivedItem } from "../adapter";
import { useI18n } from "../i18n";
import { useLingering } from "../lib/useLingering";
import { parseTraceSources } from "../trace";
import { Button } from "./ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { Markdown, READING_COLUMN_CLS } from "./Markdown";
import { DocSkeleton } from "./skeletons";
import { SourceChipRow, type SourceLinkItem } from "./SourceDiscussionChip";
import { useCopied } from "./useCopied";

export interface SpecDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** 目標 capability id；null＝未選定（不渲染內容）。 */
  capability: string | null;
  /** 刷新世代——遞增即就地重載（不清空、latest-wins；未傳＝0）。 */
  refreshGen?: number;
  /** 讀取正典 spec.md 全文（capability 定址）；缺件回 null。 */
  loadDocument: (capability: string) => Promise<string | null>;
  /** 封存變更清單（host 自 store 帶入，開工作區即全量載入）：把 @trace 來源變更名對應到
   * 封存目錄名與封存日期（drawer-provenance-links design D2）；缺席／空＝全部籤不可點。 */
  archivedChanges?: ArchivedItem[];
  /** 點可點的溯源籤：宿主開啟該封存變更的唯讀抽屜（帶 datedName）。 */
  onOpenArchivedChange?: (datedName: string) => void;
}

type Doc = string | null | undefined;

/** 溯源籤項：可點者帶 datedName 供跳轉。 */
type TraceItem = SourceLinkItem & { datedName?: string };

/**
 * 把正典全文的 @trace 來源變更名解析成出身列籤項（design D2）：
 * 封存清單命中者依封存日期升冪（同日依文件首次出現序）、副標為封存日期；
 * 未命中者不可點、副標「無封存記錄」、排在所有可點籤之後。首籤即最早封存的變更＝出身。
 */
function resolveTraceItems(
  doc: Doc,
  archived: ArchivedItem[],
  noRecordLabel: string,
): TraceItem[] {
  const names = parseTraceSources(doc);
  if (names.length === 0) return [];
  const byName = new Map<string, ArchivedItem>();
  for (const item of archived) {
    if (!byName.has(item.name)) byName.set(item.name, item);
  }
  const hits: Array<{ name: string; index: number; item: ArchivedItem }> = [];
  const misses: string[] = [];
  names.forEach((name, index) => {
    const item = byName.get(name);
    if (item) hits.push({ name, index, item });
    else misses.push(name);
  });
  hits.sort((a, b) => a.item.date.localeCompare(b.item.date) || a.index - b.index);
  return [
    ...hits.map((h) => ({ slug: h.name, topic: h.item.date, datedName: h.item.datedName })),
    ...misses.map((name) => ({ slug: name, topic: noRecordLabel, disabled: true })),
  ];
}

/** 唯讀規格抽屜（spec「桌面 app 呈現 change 與 spec 的清單與內容」）：標頭為標題列
 * （capability 名＋複製名稱鈕）與出身列（「來自」＋溯源變更籤），內文為正典 spec.md 全文。
 * 寬度與全螢幕切換與變更詳情抽屜同款；開啟／換目標清空全量載入、世代重載不清空
 * 且 latest-wins 防交錯（design D3，與 RichDetailDrawer 的 loadAll 模式同款）。 */
export function SpecDrawer({
  open,
  onOpenChange,
  capability: capabilityProp,
  refreshGen,
  loadDocument,
  archivedChanges,
  onOpenArchivedChange,
}: SpecDrawerProps) {
  const capability = useLingering(capabilityProp);
  const { t } = useI18n();
  const [doc, setDoc] = useState<Doc>();
  const [full, setFull] = useState(false);
  const [copied, markCopied] = useCopied();

  const gen = refreshGen ?? 0;
  // latest-wins：每次載入取遞增序號，回應到達時序號已過期即丟棄（涵蓋世代與換目標的交錯）。
  const requestSeq = useRef(0);
  // 已發起載入的世代——判斷外部世代是否落後需補載。
  const loadedGen = useRef(-1);

  const load = (g: number, target: string, clear: boolean) => {
    const seq = ++requestSeq.current;
    loadedGen.current = g;
    if (clear) setDoc(undefined);
    void loadDocument(target)
      .then((v) => {
        if (requestSeq.current === seq) setDoc(v);
      })
      // 失敗收斂：還沒有東西可顯示才落終態空文案（undefined 停著＝永久骨架）；
      // 已有內容維持前值——重載的短暫失敗不得抹成假空態。
      .catch(() => {
        if (requestSeq.current === seq) setDoc((prev) => (prev === undefined ? null : prev));
      });
  };

  // 開啟／換目標：清空後全量載入（載入中狀態屬新內容的正確呈現）。
  useEffect(() => {
    if (!open || !capability) return;
    load(gen, capability, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, capability]);

  // 外部世代重載：不清空、回應到達後單次替換（不重置捲動）。
  useEffect(() => {
    if (!open || !capability) return;
    if (gen <= loadedGen.current) return;
    load(gen, capability, false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, capability, gen]);

  if (!capability) return null;

  const traceItems = resolveTraceItems(doc, archivedChanges ?? [], t("rdrawer.noArchiveRecord"));
  const openTrace = (name: string) => {
    const datedName = traceItems.find((it) => it.slug === name)?.datedName;
    if (datedName) onOpenArchivedChange?.(datedName);
  };
  const copyName = () => {
    void navigator.clipboard?.writeText(capability);
    markCopied();
  };

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        data-spec-drawer
        className={full ? "w-[96vw] max-w-none" : "w-[max(720px,42vw)] max-w-[95vw]"}
      >
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label={full ? t("rdrawer.restore") : t("rdrawer.fullScreen")}
          className="absolute right-11 top-3 h-7 w-7 text-muted-foreground"
          onClick={() => setFull((f) => !f)}
        >
          {full ? <Minimize2 className="h-4 w-4" /> : <Maximize2 className="h-4 w-4" />}
        </Button>
        <SheetHeader>
          {/* 標題列：capability 名＋複製名稱鈕（與規格卡、已封存抽屜的複製鈕同款）。 */}
          <div className="flex items-center gap-2 pr-14">
            <SheetTitle className="truncate">{capability}</SheetTitle>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              aria-label={copied ? t("specs.copied") : t("common.copyName")}
              className="h-6 w-6 shrink-0 text-muted-foreground hover:text-foreground"
              onClick={copyName}
            >
              {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
            </Button>
          </div>
          {/* 出身列（design D1）：「來自」＋溯源變更籤，首籤直出、其餘收 +N——與變更詳情
              抽屜、已封存抽屜的出身列同一元件；無 @trace 來源時整列缺席。 */}
          {traceItems.length > 0 && (
            <div
              data-provenance-row
              className="flex min-w-0 items-center gap-1.5 whitespace-nowrap overflow-hidden text-xs text-muted-foreground"
            >
              <SourceChipRow label={t("sdrawer.fromChanges")} items={traceItems} onOpen={openTrace} />
            </div>
          )}
        </SheetHeader>
        <div className="flex-1 min-h-0 overflow-y-auto">
          {/* 共用置中容器——正典內文同欄（design D4）。 */}
          <div data-reading-column className={READING_COLUMN_CLS}>
            {/* 載入中畫骨架；空態文案只在載入完成後出（spec「抽屜文件載入以 skeleton 呈現」）。 */}
            {doc === undefined ? (
              <DocSkeleton />
            ) : (
              <Markdown content={doc} empty={t("common.noContent")} />
            )}
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}
