// 「有新版本」圖示鈕（desktop-app spec「桌面自動更新」；desktop-notice-relocation design D2）：
// 圖示列底部、齒輪上方的狀態鈕——待同意＝下載箭頭＋琥珀點、下載中＝轉圈、待重啟＝重啟箭頭＋
// 琥珀點、失敗＝紅色警示；閒置、檢查中、已最新、檢查失敗不出現（手動檢查的結果在設定卡內
// 行內呈現）。提示是全域的事，住在圖示列而非視窗頂端橫幅；點擊進設定頁並捲到軟體更新卡。
import { AlertTriangle, Download, LoaderCircle, RotateCw } from "lucide-react";
import { SEMANTIC_TONE, Tooltip, TooltipContent, TooltipTrigger, cn, useI18n } from "@speclink/ui";

import type { UpdaterState } from "../core/updater";

export interface UpdateRailButtonProps {
  state: UpdaterState;
  onClick: () => void;
}

export function UpdateRailButton({ state, onClick }: UpdateRailButtonProps) {
  const { t } = useI18n();
  if (
    state.phase !== "available" &&
    state.phase !== "downloading" &&
    state.phase !== "restartPending" &&
    state.phase !== "error"
  ) {
    return null;
  }
  const label =
    state.phase === "error"
      ? t("updater.rail.error")
      : t(`updater.rail.${state.phase}`).replace("{version}", state.version);
  // 點只標「有事等你做」的兩態：待同意與待重啟；下載中與失敗由圖示本身表達。
  const dot = state.phase === "available" || state.phase === "restartPending";
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          data-testid="update-rail-button"
          data-phase={state.phase}
          aria-label={label}
          className="relative flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-transparent text-muted-foreground transition-colors hover:bg-foreground/5 hover:text-foreground"
          onClick={onClick}
        >
          {state.phase === "available" && <Download className="h-4 w-4" />}
          {state.phase === "downloading" && (
            <LoaderCircle className="h-4 w-4 animate-spin motion-reduce:animate-none" />
          )}
          {state.phase === "restartPending" && <RotateCw className="h-4 w-4" />}
          {state.phase === "error" && <AlertTriangle className={cn("h-4 w-4", SEMANTIC_TONE.danger)} />}
          {dot && (
            <span
              data-update-dot
              className="absolute right-0.5 top-0.5 h-2 w-2 rounded-full border-2 border-sidebar bg-status-warning"
            />
          )}
        </button>
      </TooltipTrigger>
      <TooltipContent side="right">{label}</TooltipContent>
    </Tooltip>
  );
}
