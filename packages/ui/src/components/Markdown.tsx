import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export interface MarkdownProps {
  content: string | null;
  empty?: string;
}

/** 富文本 markdown 渲染（GFM：表格、checkbox 任務清單、刪除線）。樣式見 .markdown。 */
export function Markdown({ content, empty = "（無內容）" }: MarkdownProps) {
  if (!content || !content.trim()) {
    return <div className="text-muted-foreground text-sm py-6">{empty}</div>;
  }
  return (
    <div className="markdown">
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
    </div>
  );
}
