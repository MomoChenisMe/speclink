// 軟體更新卡的「下載與安裝」列（desktop-app spec「桌面自動更新」；desktop-notice-relocation
// design D3）：承接原頂端橫幅的全部動作——待同意＝目標版本＋「下載」、下載中＝不確定進度列＋
// 「取消」、待重啟＝「更新已就緒」＋「安裝並重新啟動」、失敗＝錯誤＋「重試」（重新檢查）；
// 其他狀態不出現（檢查結果由卡內行內文字承載）。取消＝放棄本次同意回閒置：外掛沒有真正的
// 下載取消，其後到達的下載完成在閒置態被狀態機忽略。進度列為不確定狀態：外掛的下載進度
// 事件未接（design Non-Goals）。
import { Button, SEMANTIC_TONE, cn, useI18n } from "@speclink/ui";

import { updateNeedsAttention, type UpdaterState } from "../core/updater";

export interface UpdateInstallRowProps {
  state: UpdaterState;
  onAccept: () => void;
  onCancel: () => void;
  onRelaunch: () => void;
  /** 失敗後重試＝重新手動檢查。 */
  onRetry: () => void;
}

export function UpdateInstallRow({ state, onAccept, onCancel, onRelaunch, onRetry }: UpdateInstallRowProps) {
  const { t } = useI18n();
  if (!updateNeedsAttention(state)) return null;
  return (
    <div
      data-testid="update-install-row"
      data-phase={state.phase}
      className="flex items-center gap-3 rounded-lg border border-border px-3 py-2 text-sm"
    >
      {state.phase === "available" && (
        <>
          <span className={cn("flex-1", SEMANTIC_TONE.inProgress)}>
            {t("updater.available")} {state.version}
          </span>
          <Button type="button" size="sm" className="h-7" onClick={onAccept}>
            {t("updater.row.download")}
          </Button>
        </>
      )}
      {state.phase === "downloading" && (
        <>
          <span className="flex min-w-0 flex-1 flex-col gap-1.5">
            <span className="text-muted-foreground">
              {t("updater.downloading")} {state.version}…
            </span>
            {/* 不確定進度：無 aria-valuenow，整段脈動。 */}
            <span role="progressbar" aria-label={t("updater.downloading")} className="block h-1.5 w-full overflow-hidden rounded-full bg-primary/15">
              <span className="block h-full w-1/2 animate-pulse rounded-full bg-primary motion-reduce:animate-none" />
            </span>
          </span>
          <Button type="button" size="sm" variant="ghost" className="h-7" onClick={onCancel}>
            {t("updater.row.cancel")}
          </Button>
        </>
      )}
      {state.phase === "restartPending" && (
        <>
          <span className="flex-1 text-primary">{t("updater.row.ready")}</span>
          <Button type="button" size="sm" className="h-7" onClick={onRelaunch}>
            {t("updater.row.installRestart")}
          </Button>
        </>
      )}
      {state.phase === "error" && (
        <>
          <span className={cn("min-w-0 flex-1 break-words", SEMANTIC_TONE.danger)}>
            {t("updater.errorPrefix")}
            {state.message}
          </span>
          <Button type="button" size="sm" variant="outline" className="h-7" onClick={onRetry}>
            {t("updater.row.retry")}
          </Button>
        </>
      )}
    </div>
  );
}
