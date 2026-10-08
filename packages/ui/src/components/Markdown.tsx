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
import { SEMANTIC_SURFACE, SEMANTIC_TONE, type SemanticTone } from "../tone";

/** 共用閱讀欄置中容器（design D4，spec「markdown 文件內容行寬有上限」）：
 * 寬度撐滿、max-width 與 Markdown 行寬上限同值、水平 margin auto——容器寬於
 * 行寬上限時（含全螢幕）留白均分兩側。套在各抽屜分頁內容的捲動容器內側，
 * 包住整個分頁內容（區段標籤、輪卡片、任務清單與內文同欄對齊置中）。 */
export const READING_COLUMN_CLS = "w-full max-w-[96ch] mx-auto";

export interface MarkdownProps {
  content: string | null;
  /** 空狀態文案；省略時用 i18n 預設「（無內容）」。 */
  empty?: string;
}

/** GitHub Alert 四型（desktop-manual-page spec「Markdown 的 GitHub Alert 提示框」）
 * → 介面狀態語意色（資訊、成功、警告、危險；不佔主色）與 i18n 標籤鍵。 */
type AlertType = "note" | "tip" | "warning" | "caution";
const ALERT_STYLE: Record<AlertType, { tone: SemanticTone; labelKey: string }> = {
  note: { tone: "inProgress", labelKey: "markdown.alertNote" },
  tip: { tone: "success", labelKey: "markdown.alertTip" },
  warning: { tone: "warning", labelKey: "markdown.alertWarning" },
  caution: { tone: "danger", labelKey: "markdown.alertCaution" },
};
// 標記須獨占首段第一行（與 GitHub 同：`[!NOTE]` 後只能接換行或段落結束；大小寫不拘）。
const ALERT_MARKER_RE = /^\[!(NOTE|TIP|WARNING|CAUTION)\][ \t]*(?:\r?\n|$)/i;
const ALERT_BOX_CLS = "my-4 rounded-r-md border-l-4 px-4 py-2 [&>p]:my-1 [&>ul]:my-1 [&>ol]:my-1";
const ALERT_TITLE_CLS = "markdown-alert-title mb-1 text-sm font-semibold";

/** 提示框用到的全部 class token。Streamdown 的 sanitize 預設剝掉 div 的 class，這裡
 * 只放行這份清單——文件裡的 raw HTML 不能借道塞任意 class。 */
const ALERT_CLASS_TOKENS = [
  ...new Set([
    "markdown-alert",
    ...Object.keys(ALERT_STYLE).map((type) => `markdown-alert-${type}`),
    ...ALERT_BOX_CLS.split(" "),
    ...ALERT_TITLE_CLS.split(" "),
    ...Object.values(ALERT_STYLE).flatMap(({ tone }) => [
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
    const { tone } = ALERT_STYLE[type];
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
        className: classTokens("markdown-alert", `markdown-alert-${type}`, ALERT_BOX_CLS, SEMANTIC_SURFACE[tone]),
      },
    };
    quote.children!.unshift({
      type: "paragraph",
      data: {
        hName: "div",
        hProperties: { className: classTokens(ALERT_TITLE_CLS, SEMANTIC_TONE[tone]) },
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
export function Markdown({ content, empty }: MarkdownProps) {
  const { t } = useI18n();
  // Streamdown 的區塊 memo 逐一比對屬性物件是否同一個：固定設定放模組層，隨語系變動的用 useMemo。
  const remarkPlugins = useMemo(
    () => [
      ...Object.values(defaultRemarkPlugins),
      remarkDropHtml,
      [
        remarkGithubAlerts,
        {
          note: t(ALERT_STYLE.note.labelKey),
          tip: t(ALERT_STYLE.tip.labelKey),
          warning: t(ALERT_STYLE.warning.labelKey),
          caution: t(ALERT_STYLE.caution.labelKey),
        },
      ] as Pluggable,
      remarkBreaks,
    ],
    [t],
  );
  const translations = useMemo(() => ({ copyCode: t("markdown.copyCode"), copied: t("markdown.copied") }), [t]);
  if (!content || !content.trim()) {
    return <div className="text-muted-foreground text-sm py-6">{empty ?? t("common.noContent")}</div>;
  }
  return (
    <div className="markdown max-w-[96ch]">
      <Streamdown
        mode="static"
        remarkPlugins={remarkPlugins}
        rehypePlugins={REHYPE_PLUGINS}
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
