import { useEffect, useState } from "react";
import { Check, ChevronDown, ChevronRight, Copy } from "lucide-react";

import type { SpecItem } from "../adapter";
import { useI18n } from "../i18n";
import { matchesQuery } from "../search";
import { relativeDays } from "../time";
import { parseTraceSources } from "../trace";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Markdown } from "./Markdown";

type Doc = string | null | undefined;

/** 規格卡：名稱＋相對修改時間（mtime 缺席即不顯示）＋複製名稱；點標題展開
 * 懶載入正典 spec.md 全文（design D4：首次展開才讀、同 session 重展不重載）。 */
function SpecRow({
  item,
  loadDocument,
  refreshGen,
}: {
  item: SpecItem;
  loadDocument: (capability: string) => Promise<string | null>;
  refreshGen: number;
}) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [doc, setDoc] = useState<Doc>();
  // 已載入內容對應的世代；null＝從未載入（或快取已被世代清空）。
  const [loadedGen, setLoadedGen] = useState<number | null>(null);

  const copy = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard?.writeText(item.id);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  const toggle = () => {
    const next = !expanded;
    setExpanded(next);
    if (next && loadedGen == null) {
      setLoadedGen(refreshGen);
      void loadDocument(item.id).then(setDoc);
    }
  };

  // 世代遞增清空快取（design D4）：展開中就地重載（保留舊文避免閃爍）、
  // 縮合則丟棄內容，下次展開重讀磁碟現況。
  useEffect(() => {
    if (loadedGen == null || refreshGen <= loadedGen) return;
    if (expanded) {
      setLoadedGen(refreshGen);
      void loadDocument(item.id).then(setDoc);
    } else {
      setLoadedGen(null);
      setDoc(undefined);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshGen]);

  const rel = relativeDays(item.modifiedAt, t);
  // 溯源 footer（design D2）：聚合全文所有 @trace 的 source，去重保序；空即不渲染。
  const traceSources = parseTraceSources(doc);

  return (
    <div data-spec={item.id} className="rounded-lg border border-border bg-card">
      <Button
        type="button"
        variant="ghost"
        className="group h-auto w-full justify-start gap-2.5 whitespace-normal rounded-lg p-3 text-left font-normal hover:bg-transparent"
        aria-expanded={expanded}
        onClick={toggle}
      >
        {expanded ? (
          <ChevronDown className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        )}
        <span className="font-medium text-sm truncate flex-1">{item.id}</span>
        {rel && <span className="text-xs text-muted-foreground tabular-nums shrink-0">{rel}</span>}
        <span
          role="button"
          aria-label={copied ? t("specs.copied") : t("common.copyName")}
          className={`shrink-0 text-muted-foreground hover:text-foreground transition-opacity ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
          onClick={copy}
        >
          {copied ? <Check className="h-3.5 w-3.5 text-primary" /> : <Copy className="h-3.5 w-3.5" />}
        </span>
      </Button>
      {expanded && (
        <div className="px-3 pb-3 border-t border-border pt-3 max-h-[50vh] overflow-y-auto">
          <Markdown
            content={doc ?? null}
            empty={doc === undefined ? t("common.loading") : t("common.noContent")}
          />
          {traceSources.length > 0 && (
            <div className="mt-3 pt-2 border-t border-border/60 text-xs text-muted-foreground">
              {t("specs.sourceChanges") + traceSources.join(t("specs.sourceSep"))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export interface SpecListProps {
  specs: SpecItem[];
  /** 展開懶載入正典 spec.md 全文（capability 定址）。 */
  loadDocument: (capability: string) => Promise<string | null>;
  /** 刷新世代——遞增時清空已載入內容快取（外部變更反映）；未傳＝0。 */
  refreshGen?: number;
}

/** 規格頁（design D1）：正典 spec 卡片清單＋名稱搜尋（design D3：大小寫不敏感
 * 子字串、純前端即打即濾）＋展開全文；純唯讀，無任何規格寫入動詞。 */
export function SpecList({ specs, loadDocument, refreshGen = 0 }: SpecListProps) {
  const { t } = useI18n();
  // 搜尋字串留元件內——規格頁無跨視圖保留需求（比對規則共用 matchesQuery）。
  const [query, setQuery] = useState("");
  const filtered = specs.filter((s) => matchesQuery(query, s.id));
  return (
    <div className="flex flex-col gap-3 max-w-3xl mx-auto w-full">
      <Input
        placeholder={t("specs.searchPlaceholder")}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold">{t("specs.heading")}</h2>
        <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums">
          {filtered.length}
        </span>
      </div>
      <div className="flex flex-col gap-2.5">
        {specs.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">{t("specs.empty")}</div>
        ) : filtered.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">{t("specs.noResults")}</div>
        ) : (
          filtered.map((s) => (
            <SpecRow key={s.id} item={s} loadDocument={loadDocument} refreshGen={refreshGen} />
          ))
        )}
      </div>
    </div>
  );
}
