import { useI18n } from "../i18n";

export interface DocumentViewerProps {
  content: string | null;
}

/** 呈現文件內容。v1 以原始 markdown 純文字呈現；content 為 null 時顯示空狀態。 */
export function DocumentViewer({ content }: DocumentViewerProps) {
  const { t } = useI18n();
  if (content === null) {
    return (
      <div className="text-muted-foreground p-6 text-center">{t("viewer.empty")}</div>
    );
  }
  return (
    <pre className="rounded-lg border border-border bg-card text-card-foreground p-4.5 m-0 overflow-x-auto whitespace-pre-wrap break-words font-mono text-[12.5px] leading-relaxed">
      {content}
    </pre>
  );
}
