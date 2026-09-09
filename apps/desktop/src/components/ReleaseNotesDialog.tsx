// 更新日誌對話框（desktop-app spec「更新日誌彈窗」）：同一個對話框兩種模式——
// whatsNew（版號變更後首次啟動自動彈出，標題「X.Y.Z 更新內容」、主鈕「知道了」）與
// browse（設定頁按鈕開啟，標題「更新日誌」、列出全部條目、主鈕「關閉」）。看過即止的
// 記錄由 App 層在 onOpenChange 決定，這裡只呈現（MigrationDialog 先例）。
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  useI18n,
} from "@speclink/ui";

import type { ReleaseNotesEntry } from "../release-notes/release-notes";

export interface ReleaseNotesDialogProps {
  open: boolean;
  mode: "whatsNew" | "browse";
  entries: ReleaseNotesEntry[];
  onOpenChange: (open: boolean) => void;
}

export function ReleaseNotesDialog({ open, mode, entries, onOpenChange }: ReleaseNotesDialogProps) {
  const { t } = useI18n();
  const title =
    mode === "whatsNew"
      ? t("releaseNotes.whatsNewTitle").replace("{version}", entries[0].version)
      : t("releaseNotes.browseTitle");

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      {/* 寬度沿 markdown 行寬上限（Markdown.tsx 的 96ch）；內容區可捲動。 */}
      <AlertDialogContent
        className="max-w-[96ch] gap-4"
        aria-describedby={undefined}
        data-testid="release-notes-dialog"
      >
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
        </AlertDialogHeader>

        <div className="flex max-h-[60vh] flex-col gap-5 overflow-y-auto pr-1">
          {entries.length === 0 ? (
            <p className="m-0 rounded-md border border-dashed border-border p-3 text-sm text-muted-foreground">
              {t("releaseNotes.empty")}
            </p>
          ) : (
            entries.map((entry) => (
              <section key={entry.version} className="flex flex-col gap-3">
                <h3 className="m-0 text-sm font-semibold">
                  {entry.version}（{entry.date}）
                </h3>
                {entry.sections.map((section) => (
                  <div key={section.title} className="flex flex-col gap-1">
                    <h4 className="m-0 text-xs font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                      {section.title}
                    </h4>
                    <ul className="m-0 list-disc pl-5 text-sm leading-6">
                      {section.items.map((item, index) => (
                        <li key={index}>{item}</li>
                      ))}
                    </ul>
                  </div>
                ))}
              </section>
            ))
          )}
        </div>

        <div className="flex items-center justify-end border-t border-border pt-3">
          <Button type="button" size="sm" onClick={() => onOpenChange(false)}>
            {mode === "whatsNew" ? t("releaseNotes.gotIt") : t("releaseNotes.close")}
          </Button>
        </div>
      </AlertDialogContent>
    </AlertDialog>
  );
}
