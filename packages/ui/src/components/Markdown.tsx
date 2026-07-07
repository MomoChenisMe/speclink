import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { useI18n } from "../i18n";

export interface MarkdownProps {
  content: string | null;
  /** 空狀態文案；省略時用 i18n 預設「（無內容）」。 */
  empty?: string;
}

/** 富文本 markdown 渲染（GFM：表格、checkbox 任務清單、刪除線）。樣式見 .markdown。 */
export function Markdown({ content, empty }: MarkdownProps) {
  const { t } = useI18n();
  if (!content || !content.trim()) {
    return <div className="text-muted-foreground text-sm py-6">{empty ?? t("common.noContent")}</div>;
  }
  return (
    <div className="markdown">
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
    </div>
  );
}
