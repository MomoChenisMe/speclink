// 技能檔提示確認框（desktop-app spec「指令檔過期提示」；desktop-notice-relocation design D1）：
// 琥珀圖示＋標題＋說明、一張細框卡三列（專案技能檔版本、Speclink 版本、檔案數），按鈕依
// 狀態——過期／缺失三鍵「稍後」「保留現狀」「更新／安裝技能檔」、較新兩鍵「保留現狀」
// 「更新 Speclink」。ConfirmDialog 是兩鍵式，三選項以上各自組 AlertDialog（spec「共用元件
// 唯一來源」）。較新態不提供任何改寫動作：按「更新」把新版檔案改寫回舊內容，正是
// 2026-08-05 事故的形狀——改開設定頁軟體更新卡。「稍後」只關框、「保留現狀」寫入既有略過
// 記憶；更新／安裝不自行關框：成功由提示消失收合，失敗的錯誤留在內容卡下方可重試。
// 規格資料夾遷移由同一個框承載（desktop-spec-dir-migration D2）：按下前寫明會改名或為何
// 暫緩，遷移完成後改顯示要 commit 的路徑與提醒，直到使用者按「知道了」。
import { AlertCircle } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  Card,
  SEMANTIC_TONE,
  cn,
  useI18n,
} from "@speclink/ui";

import type { AssetUpdateResult, MigrationProbe } from "../adapter/workspace";
import type { AssetPromptState } from "../assetPrompt";

export interface AssetNoticeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  prompt: AssetPromptState | null;
  /** 再生進行中：全部動作鍵停用。 */
  busy: boolean;
  /** 上次更新／安裝的失敗訊息（呈現於內容卡下方、可重試）。 */
  error: string | null;
  onApply: () => void;
  onDismiss: () => void;
  /** 較新態的主動作：切到設定頁並捲到軟體更新卡。 */
  onOpenSettingsUpdate: () => void;
  /** 上次更新的遷移結果；有值時顯示完成態（提示已消失也照樣顯示）。 */
  migrated: AssetUpdateResult["migrated"] | null;
  /** 完成態的「知道了」。 */
  onDone: () => void;
}

export function AssetNoticeDialog({
  open,
  onOpenChange,
  prompt,
  busy,
  error,
  onApply,
  onDismiss,
  onOpenSettingsUpdate,
  migrated,
  onDone,
}: AssetNoticeDialogProps) {
  const { t } = useI18n();
  if (migrated) return <MigratedDialog open={open} migrated={migrated} onDone={onDone} />;
  if (!prompt) return null;
  const newer = prompt.kind === "newer";
  const migrateOnly = prompt.kind === "migrate";
  // 前兩列同一套技能檔版號（專案的標記 vs 這版 Speclink 帶的），一眼比得出誰新誰舊。
  const rows: Array<[string, string]> = [
    [t("assets.dialog.projectVersion"), prompt.projectVersion ?? "—"],
    [t("assets.dialog.appVersion"), prompt.version],
    [t(newer ? "assets.dialog.revertCount" : "assets.dialog.fileCount"), String(prompt.fileCount)],
  ];
  return (
    // 更新進行中不收框（含 Esc）：完成態與錯誤都要留在框內。
    <AlertDialog open={open} onOpenChange={(next) => !busy && onOpenChange(next)}>
      <AlertDialogContent data-testid="asset-notice-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2">
            <AlertCircle className={cn("h-5 w-5 shrink-0", SEMANTIC_TONE.warning)} />
            {t(`assets.dialog.${prompt.kind}Title`)}
          </AlertDialogTitle>
          <AlertDialogDescription>{t(`assets.dialog.${prompt.kind}Desc`)}</AlertDialogDescription>
        </AlertDialogHeader>
        {!migrateOnly && (
          <Card size="nested" data-testid="asset-notice-rows" className="divide-y divide-border text-[13px]">
            {rows.map(([label, value]) => (
              <div key={label} className="flex items-center justify-between gap-4 px-3 py-2">
                <span className="text-muted-foreground">{label}</span>
                <span data-testid="asset-notice-value" className="font-medium tabular-nums">
                  {value}
                </span>
              </div>
            ))}
          </Card>
        )}
        {prompt.migration.state !== "none" && <MigrationNote migration={prompt.migration} />}
        {error && (
          <p role="alert" className={cn("m-0 text-[13px]", SEMANTIC_TONE.danger)}>
            {t("assets.errorPrefix")}
            {error}
          </p>
        )}
        <AlertDialogFooter data-testid="asset-notice-actions">
          {!newer && (
            <Button type="button" variant="ghost" disabled={busy} onClick={() => onOpenChange(false)}>
              {t("assets.dialog.later")}
            </Button>
          )}
          <AlertDialogCancel disabled={busy} onClick={onDismiss}>
            {t("assets.dialog.keep")}
          </AlertDialogCancel>
          {prompt.kind === "newer" ? (
            <AlertDialogAction onClick={onOpenSettingsUpdate}>{t("assets.dialog.updateApp")}</AlertDialogAction>
          ) : (
            <Button type="button" disabled={busy} onClick={onApply}>
              {t(APPLY_LABEL[prompt.kind])}
            </Button>
          )}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

/** 主動作的文案：過期→更新、缺失→安裝、只剩遷移→更新規格資料夾（較新態沒有這個鍵）。 */
const APPLY_LABEL: Record<Exclude<AssetPromptState["kind"], "newer">, string> = {
  stale: "assets.dialog.update",
  missing: "assets.dialog.install",
  migrate: "assets.dialog.migrate",
};

/** 按下更新前的遷移說明：會改名什麼，或這次為何不改名——暫緩只出現在技能檔過期或缺失的
 * 提示裡，按更新仍會更新技能檔，要講明資料夾不動。 */
function MigrationNote({ migration }: { migration: Exclude<MigrationProbe, { state: "none" }> }) {
  const { t, tn } = useI18n();
  const lines =
    migration.state === "pending"
      ? [
          ...(migration.folder === "openspec" ? [t("assets.migration.moveFolder")] : []),
          tn("assets.migration.renameFiles", migration.metaFiles),
          t("assets.migration.noCommit"),
        ]
      : [
          t("assets.migration.deferred").replace(
            "{reason}",
            t(`assets.migration.reason.${migration.reason}`)
              .replace("{paths}", migration.worktrees.join("、"))
              .replace("{folder}", migration.folder),
          ),
          t("assets.migration.deferredNote"),
        ];
  return (
    <div data-testid="asset-notice-migration" className="space-y-1 text-[13px]">
      {lines.map((line) => (
        <p key={line} className="m-0">
          {line}
        </p>
      ))}
    </div>
  );
}

/** 遷移完成態：做了什麼、要 commit 的路徑；資料夾改名時再加兩則提醒（與 CLI 相同）。 */
function MigratedDialog({
  open,
  migrated,
  onDone,
}: {
  open: boolean;
  migrated: AssetUpdateResult["migrated"];
  onDone: () => void;
}) {
  const { t, tn } = useI18n();
  const lines = migrated.movedFolder
    ? [
        tn("assets.done.moved", migrated.renamed),
        t("assets.done.commitMove"),
        t("assets.done.teammates"),
        t("assets.done.scripts"),
      ]
    : [tn("assets.done.renamed", migrated.renamed), t("assets.done.commitRenamed")];
  return (
    <AlertDialog open={open} onOpenChange={(next) => !next && onDone()}>
      <AlertDialogContent data-testid="asset-notice-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle>{t("assets.done.title")}</AlertDialogTitle>
          <AlertDialogDescription>{lines[0]}</AlertDialogDescription>
        </AlertDialogHeader>
        <ul className="m-0 list-disc space-y-1 pl-5 text-[13px]">
          {lines.slice(1).map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
        <AlertDialogFooter>
          {/* 按鈕與 Esc 都經 onOpenChange 收框，onDone 只走一次。 */}
          <AlertDialogAction>{t("assets.done.close")}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
