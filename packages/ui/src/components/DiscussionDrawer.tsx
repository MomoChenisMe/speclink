import { useEffect, useRef, useState } from "react";
import { ArrowUpRight, FileText, Flag, MessagesSquare, Rocket } from "lucide-react";

import type { ArchivedItem, ChangeItem, DiscussionItem } from "../adapter";
import { useI18n } from "../i18n";
import { Button } from "./ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "./ui/sheet";
import { Markdown } from "./Markdown";
import { discussionChipStage } from "./DiscussionColumn";

/** 討論記錄的三個標準區段。 */
export interface DiscussionSections {
  context: string;
  rounds: string;
  conclusion: string;
}

/**
 * 把討論記錄全文按 `## Context`／`## Rounds`／`## Conclusion` 切分（design D5）。
 * 任一區段缺失（手寫、pre-scaffold 格式）回 `null`——呼叫端整篇以單一檢視退回。
 */
export function splitDiscussionSections(text: string): DiscussionSections | null {
  const body = (name: string): string | null => {
    const lines = text.split(/\r?\n/);
    const start = lines.findIndex((l) => l.trimEnd() === `## ${name}`);
    if (start < 0) return null;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i++) {
      if (lines[i].startsWith("## ") && !lines[i].startsWith("###")) {
        end = i;
        break;
      }
    }
    return lines.slice(start + 1, end).join("\n");
  };
  const context = body("Context");
  const rounds = body("Rounds");
  const conclusion = body("Conclusion");
  if (context === null || rounds === null || conclusion === null) return null;
  return { context, rounds, conclusion };
}

export interface DiscussionDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  discussion: DiscussionItem | null;
  /** 刷新世代——遞增即重載討論記錄內容（未傳＝0，行為等同僅開啟時載入）。 */
  refreshGen?: number;
  /** 記錄全文載入（slug 定址）。 */
  loadDocument: (slug: string) => Promise<string | null>;
  /** active change 清單（衍生變更分頁現況派生）。 */
  changes: ChangeItem[];
  /** 已封存 change 清單（衍生變更分頁已封存態派生）。 */
  archivedChanges: ArchivedItem[];
  /** 轉為變更／再轉出一個變更請求（app 端接確認對話框）。 */
  onPromote?: (slug: string) => void;
  /** 衍生變更分頁「開啟卡片」跳轉。 */
  onOpenChangeCard?: (name: string) => void;
  /** 轉為變更失敗的單行錯誤（app 端注入；null＝無錯誤）。 */
  error?: string | null;
}

/**
 * 討論抽屜：結論／討論過程／背景／衍生變更四分頁（結論非空時預設開結論），
 * 標題下方生命週期階梯標示現站（design D3）；衍生變更分頁列各子變更現況與
 * 跳轉，底部轉為變更（concluded）／再轉出一個變更（promoted）動詞。
 * 記錄格式非預期時整篇單一檢視退回。GUI 不提供 conclude 等寫入。
 */
export function DiscussionDrawer({
  open,
  onOpenChange,
  discussion,
  refreshGen,
  loadDocument,
  changes,
  archivedChanges,
  onPromote,
  onOpenChangeCard,
  error,
}: DiscussionDrawerProps) {
  const { t } = useI18n();
  const [doc, setDoc] = useState<string | null | undefined>();
  const slug = discussion?.slug ?? null;
  const gen = refreshGen ?? 0;
  // latest-wins：回應帶發起序號，落後即丟棄（涵蓋世代與換討論的交錯）。
  const requestSeq = useRef(0);
  const loadedGen = useRef(-1);

  const loadRecord = (g: number, target: string, clear: boolean) => {
    const seq = ++requestSeq.current;
    loadedGen.current = g;
    if (clear) setDoc(undefined);
    void loadDocument(target).then((v) => {
      if (requestSeq.current === seq) setDoc(v);
    });
  };

  // 開啟／換討論：清空後載入。
  useEffect(() => {
    if (!open || !slug) return;
    loadRecord(gen, slug, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, slug]);

  // 外部世代重載（add-round／conclude／轉出後）：不清空、回應到達後就地替換，分頁選擇不重置。
  useEffect(() => {
    if (!open || !slug) return;
    if (gen <= loadedGen.current) return;
    loadRecord(gen, slug, false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, slug, gen]);

  if (!discussion) return null;

  const sections = doc ? splitDiscussionSections(doc) : null;
  const canPromote = discussion.status === "concluded" || discussion.status === "promoted";
  const promoteLabel =
    discussion.promotedTo.length > 0 ? t("ddrawer.promoteAgain") : t("discussion.promote");

  const promotePane = (
    <div className="flex h-full flex-col gap-2">
      {discussion.promotedTo.length === 0 ? (
        <p className="text-sm text-muted-foreground py-4">{t("ddrawer.notPromoted")}</p>
      ) : (
        <div className="flex flex-col gap-1.5">
          {discussion.promotedTo.map((name) => {
            const stage = t(discussionChipStage(name, changes, archivedChanges));
            const alive = changes.some((c) => c.name === name);
            return (
              <div
                key={name}
                data-promoted-row={name}
                className="flex items-center gap-2 rounded-md border border-border/60 px-2.5 py-1.5"
              >
                <span className="text-sm font-medium min-w-0 flex-1 truncate">{name}</span>
                <span className="shrink-0 rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
                  {stage}
                </span>
                {alive && (
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-6 gap-1 px-2 text-xs shrink-0"
                    onClick={() => onOpenChangeCard?.(name)}
                  >
                    <ArrowUpRight className="h-3 w-3" /> {t("ddrawer.openCard")}
                  </Button>
                )}
              </div>
            );
          })}
        </div>
      )}
      {canPromote && (
        <div className="pt-2">
          <Button
            variant="outline"
            size="sm"
            className="h-7 gap-1"
            onClick={() => onPromote?.(discussion.slug)}
          >
            <Rocket className="h-3.5 w-3.5" /> {promoteLabel}
          </Button>
        </div>
      )}
    </div>
  );

  // 預設分頁＝讀者第一想看的：結論非空（去除鷹架註解後有內容）開結論，否則背景。
  const conclusionHasContent =
    !!sections && sections.conclusion.replace(/<!--[\s\S]*?-->/g, "").trim().length > 0;
  const defaultTab = conclusionHasContent ? "conclusion" : "context";

  // 生命週期階梯：現站由 status 決定，封存不入階梯（封存後只在已封存頁）。
  const stationIndex = discussion.status === "promoted" ? 2 : discussion.status === "concluded" ? 1 : 0;
  const stations = [
    t("discussion.statusOpen"),
    t("discussion.statusConcluded"),
    t("ddrawer.stationPromoted"),
  ];

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-[max(720px,42vw)] max-w-[95vw]">
        <SheetHeader>
          <div className="flex items-center gap-2 pr-14">
            <SheetTitle className="truncate">{discussion.topic}</SheetTitle>
          </div>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span className="tabular-nums">{t("common.rounds").replace("{n}", String(discussion.rounds))}</span>
            {discussion.created && <span>{discussion.created}</span>}
          </div>
          {/* 生命週期階梯：狀態模型可見，免使用者腦補（design D3）。 */}
          <div className="flex items-center gap-1.5 text-xs">
            {stations.map((label, i) => (
              <span key={label} className="flex items-center gap-1.5">
                {i > 0 && <span className="text-muted-foreground/50">→</span>}
                <span
                  aria-current={i === stationIndex ? "step" : undefined}
                  className={
                    i === stationIndex
                      ? "inline-flex items-center gap-1 rounded-full bg-primary/12 px-2 py-0.5 font-semibold text-primary"
                      : i < stationIndex
                        ? "inline-flex items-center gap-1 text-muted-foreground"
                        : "inline-flex items-center gap-1 text-muted-foreground/50"
                  }
                >
                  <span aria-hidden="true">{i < stationIndex ? "✓" : i === stationIndex ? "●" : "○"}</span>
                  <span>{label}</span>
                </span>
              </span>
            ))}
          </div>
          {error && (
            <p role="alert" className="text-xs text-destructive">
              {error}
            </p>
          )}
        </SheetHeader>

        {sections ? (
          // key 區分分頁組與預設分頁——否則自載入中 fallback 切換過來時，Radix
          // Tabs 的未受控 state 停留在不存在的分頁值，內容全數隱藏。
          <Tabs
            key={`sections-${defaultTab}`}
            defaultValue={defaultTab}
            className="flex-1 min-h-0 flex flex-col"
          >
            <TabsList>
              <TabsTrigger value="conclusion">
                <Flag className="h-3.5 w-3.5" /> {t("ddrawer.tabConclusion")}
              </TabsTrigger>
              <TabsTrigger value="rounds">
                <MessagesSquare className="h-3.5 w-3.5" /> {t("ddrawer.tabRounds")} {discussion.rounds}
              </TabsTrigger>
              <TabsTrigger value="context">
                <FileText className="h-3.5 w-3.5" /> {t("ddrawer.tabContext")}
              </TabsTrigger>
              <TabsTrigger value="promote">
                <Rocket className="h-3.5 w-3.5" /> {t("ddrawer.tabPromote")}
              </TabsTrigger>
            </TabsList>
            <div className="flex-1 overflow-y-auto pt-3">
              <TabsContent value="conclusion"><Markdown content={sections.conclusion} empty={t("ddrawer.noConclusion")} /></TabsContent>
              <TabsContent value="rounds"><Markdown content={sections.rounds} empty={t("ddrawer.noRounds")} /></TabsContent>
              <TabsContent value="context"><Markdown content={sections.context} empty={t("ddrawer.noContext")} /></TabsContent>
              <TabsContent value="promote">{promotePane}</TabsContent>
            </div>
          </Tabs>
        ) : (
          <Tabs key="fallback" defaultValue="record" className="flex-1 min-h-0 flex flex-col">
            <TabsList>
              <TabsTrigger value="record">
                <FileText className="h-3.5 w-3.5" /> {t("ddrawer.tabRecord")}
              </TabsTrigger>
              <TabsTrigger value="promote">
                <Rocket className="h-3.5 w-3.5" /> {t("ddrawer.tabPromote")}
              </TabsTrigger>
            </TabsList>
            <div className="flex-1 overflow-y-auto pt-3">
              {/* 區段缺失或格式非預期：整篇以單一檢視退回（不報錯）。 */}
              <TabsContent value="record"><Markdown content={doc ?? null} empty={t("common.loading")} /></TabsContent>
              <TabsContent value="promote">{promotePane}</TabsContent>
            </div>
          </Tabs>
        )}
      </SheetContent>
    </Sheet>
  );
}
