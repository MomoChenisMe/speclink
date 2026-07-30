// 更新通知列（desktop-app spec「桌面自動更新」，design D6）：available 徵詢
// （顯示版本、同意／稍後）、downloading 進度提示、restartPending 重啟提示、
// error 呈現錯誤。其餘狀態不佔畫面——檢查結果的行內呈現歸設定頁軟體更新卡。
import { AlertTriangle, ArrowUpCircle } from "lucide-react";
import { Button, useI18n } from "@speclink/ui";

import type { UpdaterState } from "../core/updater";

export interface UpdateBannerProps {
  state: UpdaterState;
  onAccept: () => void;
  onDismiss: () => void;
  onRelaunch: () => void;
}

export function UpdateBanner({ state, onAccept, onDismiss, onRelaunch }: UpdateBannerProps) {
  const { t } = useI18n();
  if (
    state.phase !== "available" &&
    state.phase !== "downloading" &&
    state.phase !== "restartPending" &&
    state.phase !== "error"
  ) {
    return null;
  }
  const isError = state.phase === "error";
  return (
    <div
      data-testid="update-banner"
      role="status"
      className={`flex items-center gap-2.5 px-4 py-1.5 text-sm border-b border-border shrink-0 ${
        isError ? "bg-amber-500/10 text-amber-700 dark:text-amber-400" : "bg-primary/8"
      }`}
    >
      {isError ? (
        <AlertTriangle className="h-4 w-4 shrink-0" />
      ) : (
        <ArrowUpCircle className="h-4 w-4 shrink-0 text-primary" />
      )}
      {state.phase === "available" && (
        <>
          <span>
            {t("updater.available")} {state.version}
          </span>
          <span className="ml-auto flex gap-1.5">
            <Button type="button" size="sm" className="h-7" onClick={onAccept}>
              {t("updater.accept")}
            </Button>
            <Button type="button" size="sm" variant="ghost" className="h-7" onClick={onDismiss}>
              {t("updater.later")}
            </Button>
          </span>
        </>
      )}
      {state.phase === "downloading" && (
        <span>
          {t("updater.downloading")} {state.version}…
        </span>
      )}
      {state.phase === "restartPending" && (
        <>
          <span>{t("updater.restartPending")}</span>
          <Button type="button" size="sm" className="h-7 ml-auto" onClick={onRelaunch}>
            {t("updater.restartNow")}
          </Button>
        </>
      )}
      {isError && (
        <>
          <span>
            {t("updater.errorPrefix")}
            {state.message}
          </span>
          <Button type="button" size="sm" variant="ghost" className="h-7 ml-auto" onClick={onDismiss}>
            {t("updater.close")}
          </Button>
        </>
      )}
    </div>
  );
}
