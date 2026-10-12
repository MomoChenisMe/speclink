import { useMemo } from "react";
import { Check, Copy } from "lucide-react";
import remarkBreaks from "remark-breaks";
import {
  defaultRehypePlugins,
  defaultRemarkPlugins,
  Streamdown,
  type StreamdownProps,
} from "streamdown";
import { code } from "@streamdown/code";

import { useI18n } from "../i18n";
import { nameSpans } from "../discussionModel";
import { cn } from "../lib/utils";
import { rehypeSpecKeywords } from "../specSections";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { CALLOUT_BOX_CLS, CALLOUT_TITLE_CLS, CALLOUT_TONE, type CalloutTone } from "./ui/callout";

export interface MarkdownProps {
  content: string | null;
  /** 空狀態文案；省略時用 i18n 預設「（無內容）」。 */
  empty?: string;
  /** 以等寬灰底標示 SHALL／MUST 關鍵字（規格詳情頁的需求文與 Scenario）。 */
  specKeywords?: boolean;
  /** 合進外層 `.markdown` 容器（如列內行內排版的 `[&_p]:my-0`）；與預設衝突的 class 以後者為準。 */
  className?: string;
  /** 文中名稱籤（討論背景）：與名單完全相同的字詞換成 `span.name-chip[data-name-chip="<kind>:<name>"]`；
   * 款式與點擊由呼叫端容器負責。缺席或空陣列時管線不變。 */
  nameChips?: NameChip[];
}

/** 一顆名稱籤的來源：種類（spec／change／archived／discussion 等，呼叫端自訂）與名稱。 */
export interface NameChip {
  kind: string;
  name: string;
}

/** GitHub Alert 四型（desktop-manual-page spec「Markdown 的 GitHub Alert 提示框」）
 * → i18n 標籤鍵；款式與語意色取自 Callout 原語（design D6）。 */
type AlertType = CalloutTone;
const ALERT_LABEL_KEYS: Record<AlertType, string> = {
  note: "markdown.alertNote",
  tip: "markdown.alertTip",
  warning: "markdown.alertWarning",
  caution: "markdown.alertCaution",
};
// 標記須獨占首段第一行（與 GitHub 同：`[!NOTE]` 後只能接換行或段落結束；大小寫不拘）。
const ALERT_MARKER_RE = /^\[!(NOTE|TIP|WARNING|CAUTION)\][ \t]*(?:\r?\n|$)/i;

/** 提示框用到的全部 class token。Streamdown 的 sanitize 預設剝掉 div 的 class，這裡
 * 只放行這份清單——文件裡的 raw HTML 不能借道塞任意 class。 */
const ALERT_CLASS_TOKENS = [
  ...new Set([
    "markdown-alert",
    ...Object.keys(CALLOUT_TONE).map((type) => `markdown-alert-${type}`),
    ...CALLOUT_BOX_CLS.split(" "),
    ...CALLOUT_TITLE_CLS.split(" "),
    ...Object.values(CALLOUT_TONE).flatMap((tone) => [
      ...SEMANTIC_SURFACE[tone].split(" "),
      ...SEMANTIC_TONE[tone].split(" "),
    ]),
  ]),
];

type Pluggable = NonNullable<StreamdownProps["rehypePlugins"]>[number];
type SanitizeSchema = { attributes?: Record<string, unknown[]> };

/** Streamdown 預設 rehype 管線只取 sanitize 一段，schema 多放行提示框的 class（傳自訂
 * rehypePlugins 後 Streamdown 不再自動擴充 schema）。不接 raw：原始 HTML 由 remarkDropHtml
 * 整段丟棄。不接 harden：它在沒有 defaultOrigin 時把 `editor.md` 這類裸相對連結換成 [blocked]，而手冊
 * 的跨頁連結正是這種寫法；協定把關由 sanitize（擋 javascript: 等）承擔。 */
const [sanitize, baseSchema] = defaultRehypePlugins.sanitize as unknown as [Pluggable, SanitizeSchema];
const REHYPE_PLUGINS: Pluggable[] = [
  [
    sanitize,
    {
      ...baseSchema,
      attributes: {
        ...baseSchema.attributes,
        div: [...(baseSchema.attributes?.div ?? []), ["className", ...ALERT_CLASS_TOKENS]],
      },
    },
  ] as Pluggable,
];
// 關鍵字外掛接在 sanitize 之後：它產生的 spec-kw class 不經放行清單。
const SPEC_REHYPE_PLUGINS: Pluggable[] = [...REHYPE_PLUGINS, rehypeSpecKeywords as Pluggable];

/** 最小 hast 節點視圖。 */
interface HastNode {
  type: string;
  value?: string;
  tagName?: string;
  properties?: Record<string, unknown>;
  children?: HastNode[];
}

/** 名稱籤外掛（design D10）：接在 sanitize 之後，產生的 class 與 data 屬性不經放行清單（與 rehypeSpecKeywords
 * 同一路）。名單以選項物件傳入——Streamdown 以「外掛函式名＋選項 JSON」快取 processor，閉包式外掛會讓不同
 * 文件共用同一份。行內 code、程式碼區塊與連結內不處理；命中規則與 matchNames 同一條邊界、長名優先。 */
function rehypeNameChips({ names }: { names: NameChip[] }) {
  const kindOf = new Map<string, string>();
  for (const chip of names) if (!kindOf.has(chip.name)) kindOf.set(chip.name, chip.kind);
  const list = [...kindOf.keys()];
  const split = (value: string): HastNode[] | null => {
    const spans = nameSpans(value, list);
    if (spans.length === 0) return null;
    const out: HastNode[] = [];
    let last = 0;
    for (const { name, start, end } of spans) {
      if (start > last) out.push({ type: "text", value: value.slice(last, start) });
      out.push({
        type: "element",
        tagName: "span",
        properties: { className: ["name-chip"], dataNameChip: `${kindOf.get(name)}:${name}`, role: "button", tabIndex: 0 },
        children: [{ type: "text", value: name }],
      });
      last = end;
    }
    if (last < value.length) out.push({ type: "text", value: value.slice(last) });
    return out;
  };
  const walk = (node: HastNode) => {
    if (!node.children) return;
    if (node.type === "element" && (node.tagName === "code" || node.tagName === "pre" || node.tagName === "a")) return;
    node.children = node.children.flatMap((child) => {
      if (child.type === "text" && child.value) return split(child.value) ?? [child];
      walk(child);
      return [child];
    });
  };
  return walk;
}

const PLUGINS: StreamdownProps["plugins"] = { code };
const CONTROLS: StreamdownProps["controls"] = { code: { copy: true, download: false }, table: false, mermaid: false };
const ICONS: StreamdownProps["icons"] = { CopyIcon: Copy, CheckIcon: Check };
const LINK_SAFETY: StreamdownProps["linkSafety"] = { enabled: false };
const SHIKI_THEME: StreamdownProps["shikiTheme"] = ["github-light", "github-dark"];

/** 最小 mdast 節點視圖——只用到 type／value／children 與 hast 覆寫資料，不引入
 * 型別套件相依。 */
interface MdNode {
  type: string;
  value?: string;
  children?: MdNode[];
  data?: { hName?: string; hProperties?: Record<string, unknown> };
}

/** 原始 HTML（含註解）整段丟棄，與改版前的 skipHtml 一致：文件正文常把 `<summary>`、
 * `<template>` 這類佔位字當字面寫，解析成標籤會把後文藏進收合區塊或整段吞掉。
 * 必須在 mdast 階段拿掉——rehype 管線不含 raw 時，Streamdown 會把 html 節點轉成字面文字。 */
function remarkDropHtml() {
  const walk = (node: MdNode) => {
    if (!node.children) return;
    node.children = node.children.filter((child) => child.type !== "html");
    node.children.forEach(walk);
  };
  return walk;
}

/** 拆成單一 class token：sanitize 逐 token 比對放行清單，含空白的整串會被整個剝掉。 */
const classTokens = (...classes: string[]) => classes.flatMap((cls) => cls.split(" "));

/** 內建 remark 轉換（design：三十行內、不新增相依）：首段以四型標記開頭的
 * blockquote 改渲染為帶類型 class 與類型標籤的提示框、移除標記文字；其餘
 * blockquote 不觸碰。標籤以 unified 外掛選項傳入（`[remarkGithubAlerts, labels]`）：
 * Streamdown 以「外掛函式名＋選項 JSON」快取 processor，閉包式外掛會讓不同語系共用同一份。 */
function remarkGithubAlerts(labels: Record<AlertType, string>) {
  const transform = (quote: MdNode) => {
    const first = quote.children?.[0];
    const text = first?.type === "paragraph" ? first.children?.[0] : undefined;
    if (!first || text?.type !== "text" || typeof text.value !== "string") return;
    const match = ALERT_MARKER_RE.exec(text.value);
    if (!match) return;
    const type = match[1].toLowerCase() as AlertType;
    const tone = CALLOUT_TONE[type];
    text.value = text.value.slice(match[0].length);
    if (!text.value) {
      first.children!.shift();
      // 標記行以硬換行結尾（行尾兩空格）時，剩下的 break 節點也一併拿掉。
      if (first.children![0]?.type === "break") first.children!.shift();
    }
    if (first.children!.length === 0) quote.children!.shift();
    quote.data = {
      hName: "div",
      hProperties: {
        className: classTokens("markdown-alert", `markdown-alert-${type}`, CALLOUT_BOX_CLS, SEMANTIC_SURFACE[tone]),
      },
    };
    quote.children!.unshift({
      type: "paragraph",
      data: {
        hName: "div",
        hProperties: { className: classTokens(CALLOUT_TITLE_CLS, SEMANTIC_TONE[tone]) },
      },
      children: [{ type: "text", value: labels[type] }],
    });
  };
  const walk = (node: MdNode) => {
    for (const child of node.children ?? []) {
      if (child.type === "blockquote") transform(child);
      walk(child);
    }
  };
  return walk;
}

/** 富文本 markdown 渲染（Streamdown static 模式；GFM：表格、checkbox 任務清單、刪除線；
 * 單換行＝換行）。原始 HTML（含註解）整段丟棄、不以原文呈現，code fence 內文字照常。程式碼區塊以 Shiki 上色（淺色 github-light、深色 github-dark）並帶
 * 複製鈕；表格與程式碼區塊不設高度上限（維持整份展開閱讀）。.markdown 為薄覆寫掛鉤。
 * 行寬上限 96ch（≈48 全形字 @16px）——抽屜全螢幕時行寬不隨之增長（spec
 * 「markdown 文件內容行寬有上限」）；寬表格由 Streamdown 的表格容器橫捲。
 * GitHub Alert（`> [!NOTE]` 等四型）恆開、對所有 Markdown 檢視生效。 */
export function Markdown({ content, empty, specKeywords = false, className, nameChips }: MarkdownProps) {
  const { t } = useI18n();
  // Streamdown 的區塊 memo 逐一比對屬性物件是否同一個：固定設定放模組層，隨語系變動的用 useMemo。
  const remarkPlugins = useMemo(
    () => [
      ...Object.values(defaultRemarkPlugins),
      remarkDropHtml,
      [
        remarkGithubAlerts,
        {
          note: t(ALERT_LABEL_KEYS.note),
          tip: t(ALERT_LABEL_KEYS.tip),
          warning: t(ALERT_LABEL_KEYS.warning),
          caution: t(ALERT_LABEL_KEYS.caution),
        },
      ] as Pluggable,
      remarkBreaks,
    ],
    [t],
  );
  const translations = useMemo(() => ({ copyCode: t("markdown.copyCode"), copied: t("markdown.copied") }), [t]);
  // 名單以內容為鍵：呼叫端每次渲染給新陣列時不重建管線。
  const chipKey = JSON.stringify(nameChips ?? []);
  const rehypePlugins = useMemo(() => {
    const base = specKeywords ? SPEC_REHYPE_PLUGINS : REHYPE_PLUGINS;
    const names = JSON.parse(chipKey) as NameChip[];
    return names.length > 0 ? [...base, [rehypeNameChips, { names }] as Pluggable] : base;
  }, [specKeywords, chipKey]);
  if (!content || !content.trim()) {
    return <div className="text-muted-foreground text-sm py-6">{empty ?? t("common.noContent")}</div>;
  }
  return (
    <div className={cn("markdown max-w-[96ch]", className)}>
      <Streamdown
        mode="static"
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        plugins={PLUGINS}
        controls={CONTROLS}
        translations={translations}
        icons={ICONS}
        linkSafety={LINK_SAFETY}
        shikiTheme={SHIKI_THEME}
        tableMaxHeight={0}
        codeBlockMaxHeight={0}
      >
        {content}
      </Streamdown>
    </div>
  );
}
