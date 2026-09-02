import { useMemo } from "react";
import ReactMarkdown from "react-markdown";
import remarkBreaks from "remark-breaks";
import remarkGfm from "remark-gfm";

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

/** 最小 mdast 節點視圖——只用到 type／value／children 與 hast 覆寫資料，不引入
 * 型別套件相依。 */
interface MdNode {
  type: string;
  value?: string;
  children?: MdNode[];
  data?: { hName?: string; hProperties?: Record<string, unknown> };
}

/** 內建 remark 轉換（design：三十行內、不新增相依）：首段以四型標記開頭的
 * blockquote 改渲染為帶類型 class 與類型標籤的提示框、移除標記文字；其餘
 * blockquote 不觸碰，輸出與變更前逐位元一致。 */
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
        className: [
          "markdown-alert",
          `markdown-alert-${type}`,
          "my-4 rounded-r-md border-l-4 px-4 py-2 [&>p]:my-1 [&>ul]:my-1 [&>ol]:my-1",
          SEMANTIC_SURFACE[tone],
        ],
      },
    };
    quote.children!.unshift({
      type: "paragraph",
      data: {
        hName: "div",
        hProperties: { className: ["markdown-alert-title mb-1 text-sm font-semibold", SEMANTIC_TONE[tone]] },
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
  return () => walk;
}

/** 富文本 markdown 渲染（GFM：表格、checkbox 任務清單、刪除線；單換行＝換行）。
 * 排版由 typography 的 prose 接管，.markdown 為薄覆寫掛鉤；skipHtml 丟棄 raw HTML
 * （討論 scaffold 的註解不進畫面，code fence 內文字不受影響）。
 * 行寬上限 96ch（≈48 全形字 @16px）——抽屜全螢幕時行寬不隨之增長（spec
 * 「markdown 文件內容行寬有上限」）；寬表格由 .markdown table 於容器內橫捲。
 * GitHub Alert（`> [!NOTE]` 等四型）恆開、對所有 Markdown 檢視生效。 */
export function Markdown({ content, empty }: MarkdownProps) {
  const { t } = useI18n();
  const alerts = useMemo(
    () =>
      remarkGithubAlerts({
        note: t(ALERT_STYLE.note.labelKey),
        tip: t(ALERT_STYLE.tip.labelKey),
        warning: t(ALERT_STYLE.warning.labelKey),
        caution: t(ALERT_STYLE.caution.labelKey),
      }),
    [t],
  );
  if (!content || !content.trim()) {
    return <div className="text-muted-foreground text-sm py-6">{empty ?? t("common.noContent")}</div>;
  }
  return (
    <div className="markdown prose max-w-[96ch]">
      <ReactMarkdown remarkPlugins={[remarkGfm, alerts, remarkBreaks]} skipHtml>
        {content}
      </ReactMarkdown>
    </div>
  );
}
