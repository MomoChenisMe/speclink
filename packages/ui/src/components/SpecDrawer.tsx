import { useEffect, useRef, useState } from "react";
import { Maximize2, Minimize2 } from "lucide-react";

import { useI18n } from "../i18n";
import { parseTraceSources } from "../trace";
import { Button } from "./ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { Markdown, READING_COLUMN_CLS } from "./Markdown";
import { DocSkeleton } from "./skeletons";

export interface SpecDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** 目標 capability id；null＝未選定（不渲染內容）。 */
  capability: string | null;
  /** 刷新世代——遞增即就地重載（不清空、latest-wins；未傳＝0）。 */
  refreshGen?: number;
  /** 讀取正典 spec.md 全文（capability 定址）；缺件回 null。 */
  loadDocument: (capability: string) => Promise<string | null>;
}

type Doc = string | null | undefined;

/** 唯讀規格抽屜（spec-archive-drawer design D1）：正典 spec.md 全文＋溯源 footer。
 * 寬度與全螢幕切換與變更詳情抽屜同款；開啟／換目標清空全量載入、世代重載不清空
 * 且 latest-wins 防交錯（design D3，與 RichDetailDrawer 的 loadAll 模式同款）。 */
export function SpecDrawer({ open, onOpenChange, capability, refreshGen, loadDocument }: SpecDrawerProps) {
  const { t } = useI18n();
  const [doc, setDoc] = useState<Doc>();
  const [full, setFull] = useState(false);

  const gen = refreshGen ?? 0;
  // latest-wins：每次載入取遞增序號，回應到達時序號已過期即丟棄（涵蓋世代與換目標的交錯）。
  const requestSeq = useRef(0);
  // 已發起載入的世代——判斷外部世代是否落後需補載。
  const loadedGen = useRef(-1);

  const load = (g: number, target: string, clear: boolean) => {
    const seq = ++requestSeq.current;
    loadedGen.current = g;
    if (clear) setDoc(undefined);
    void loadDocument(target).then((v) => {
      if (requestSeq.current === seq) setDoc(v);
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

  // 溯源 footer：聚合全文所有 @trace 的 source，去重保序；空即不渲染。
  const traceSources = parseTraceSources(doc);

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
          <SheetTitle className="truncate pr-14">{capability}</SheetTitle>
        </SheetHeader>
        <div className="flex-1 min-h-0 overflow-y-auto">
          {/* 共用置中容器——正典內文與溯源 footer 同欄（design D4）。 */}
          <div data-reading-column className={READING_COLUMN_CLS}>
            {/* 載入中畫骨架；空態文案只在載入完成後出（spec「抽屜文件載入以 skeleton 呈現」）。 */}
            {doc === undefined ? (
              <DocSkeleton />
            ) : (
              <Markdown content={doc} empty={t("common.noContent")} />
            )}
            {traceSources.length > 0 && (
              <div className="mt-4 pt-3 border-t border-border/60 text-xs text-muted-foreground">
                {t("specs.sourceChanges") + traceSources.join(t("specs.sourceSep"))}
              </div>
            )}
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}
