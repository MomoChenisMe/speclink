export interface DocumentViewerProps {
  content: string | null;
}

/** 呈現文件內容。v1 以原始 markdown 純文字呈現；content 為 null 時顯示空狀態。 */
export function DocumentViewer({ content }: DocumentViewerProps) {
  if (content === null) {
    return (
      <div className="text-muted-foreground p-6 text-center">
        選擇左側的 change 或 spec 以檢視內容
      </div>
    );
  }
  return (
    <pre className="rounded-lg border border-border bg-card text-card-foreground p-4.5 m-0 overflow-x-auto whitespace-pre-wrap break-words font-mono text-[12.5px] leading-relaxed">
      {content}
    </pre>
  );
}
