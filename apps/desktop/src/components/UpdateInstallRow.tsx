// 更新頁「下載與安裝」卡的列（desktop-app spec「桌面自動更新」；desktop-settings-mode design D6）：
// 標題「Speclink <目標版本>」，說明與控制依狀態——待同意＝「安裝後會重新啟動」＋「下載」、
// 下載中＝不確定進度列＋「取消」、待重啟＝「更新已就緒」＋「安裝並重新啟動」、失敗＝錯誤＋
// 「重試」（重新檢查）；其他狀態不出現。取消＝放棄本次同意回閒置：外掛沒有真正的下載取消，
// 其後到達的下載完成在閒置態被狀態機忽略。進度列為不確定狀態：外掛的下載進度事件未接。
import { Button, SEMANTIC_TONE, SettingsRow, useI18n } from "@speclink/ui";

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
  // 失敗態不帶目標版本（簽章驗證失敗後待套用項已清掉）。
  const label = state.phase === "error" ? "Speclink" : `Speclink ${state.version}`;
  const button = (text: string, onClick: () => void, variant?: "ghost" | "outline") => (
    <Button type="button" size="sm" variant={variant} onClick={onClick}>
      {text}
    </Button>
  );
  return (
    <div data-testid="update-install-row" data-phase={state.phase}>
      {state.phase === "available" && (
        <SettingsRow label={label} description={t("updater.page.installHint")} control={button(t("updater.row.download"), onAccept)} />
      )}
      {state.phase === "downloading" && (
        <SettingsRow
          label={label}
          description={
            <span className="flex flex-col gap-1.5">
              <span>{t("updater.downloading")}…</span>
              {/* 不確定進度：無 aria-valuenow，整段脈動。 */}
              <span role="progressbar" aria-label={t("updater.downloading")} className="block h-1.5 w-full overflow-hidden rounded-full bg-primary/15">
                <span className="block h-full w-1/2 animate-pulse rounded-full bg-primary motion-reduce:animate-none" />
              </span>
            </span>
          }
          control={button(t("updater.row.cancel"), onCancel, "ghost")}
        />
      )}
      {state.phase === "restartPending" && (
        <SettingsRow
          label={label}
          description={t("updater.row.ready")}
          control={button(t("updater.row.installRestart"), onRelaunch)}
        />
      )}
      {state.phase === "error" && (
        <SettingsRow
          label={label}
          description={
            <span className={`break-words ${SEMANTIC_TONE.danger}`}>
              {t("updater.errorPrefix")}
              {state.message}
            </span>
          }
          control={button(t("updater.row.retry"), onRetry, "outline")}
        />
      )}
    </div>
  );
}
