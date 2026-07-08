import ReactMarkdown from "react-markdown";
import remarkBreaks from "remark-breaks";
import remarkGfm from "remark-gfm";

import { useI18n } from "../i18n";

export interface MarkdownProps {
  content: string | null;
  /** 空狀態文案；省略時用 i18n 預設「（無內容）」。 */
  empty?: string;
}

/** 富文本 markdown 渲染（GFM：表格、checkbox 任務清單、刪除線；單換行＝換行）。
 * 排版由 typography 的 prose 接管，.markdown 為薄覆寫掛鉤；skipHtml 丟棄 raw HTML
 * （討論 scaffold 的註解不進畫面，code fence 內文字不受影響）。
 * 行寬上限 72ch（≈36 全形字 @16px）——抽屜全螢幕時行寬不隨之增長（spec
 * 「markdown 文件內容行寬有上限」）；寬表格由 .markdown table 於容器內橫捲。 */
export function Markdown({ content, empty }: MarkdownProps) {
  const { t } = useI18n();
  if (!content || !content.trim()) {
    return <div className="text-muted-foreground text-sm py-6">{empty ?? t("common.noContent")}</div>;
  }
  return (
    <div className="markdown prose max-w-[72ch]">
      <ReactMarkdown remarkPlugins={[remarkGfm, remarkBreaks]} skipHtml>
        {content}
      </ReactMarkdown>
    </div>
  );
}
