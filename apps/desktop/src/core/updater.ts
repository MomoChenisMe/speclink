// 更新狀態機（desktop-app spec「桌面自動更新」，design D6）：純 reducer、不依賴
// Tauri。手動／自動檢查的失敗與已最新分流在此決定：自動一律靜默回閒置，手動
// 才浮出「無法檢查」與「已是最新」；簽章驗證等安裝失敗轉錯誤態，永不進待重啟。
export type UpdaterState =
  | { phase: "idle" }
  | { phase: "checking"; manual: boolean }
  | { phase: "available"; version: string }
  | { phase: "downloading"; version: string }
  | { phase: "restartPending"; version: string }
  | { phase: "upToDate" }
  | { phase: "checkFailed" }
  | { phase: "error"; message: string };

export type UpdaterEvent =
  | { type: "checkStarted"; manual: boolean }
  | { type: "updateFound"; version: string }
  | { type: "noUpdate" }
  | { type: "checkFailed" }
  | { type: "accepted" }
  | { type: "dismissed" }
  | { type: "downloaded" }
  | { type: "installFailed"; message: string };

export const initialUpdaterState: UpdaterState = { phase: "idle" };

/** 前景重檢的節流間隔：主視窗回到前景時，距上次檢查滿此值才再查。 */
export const FOCUS_RECHECK_INTERVAL_MS = 60 * 60 * 1000;

/**
 * 主視窗回到前景時是否該重檢（desktop-app「回到前景未滿一小時不重檢」）：
 * 從未檢查（null）視為逾時；否則距上次檢查滿 FOCUS_RECHECK_INTERVAL_MS 才重檢。
 */
export function focusRecheckDue(lastCheckedAt: number | null, now: number): boolean {
  return lastCheckedAt === null || now - lastCheckedAt >= FOCUS_RECHECK_INTERVAL_MS;
}

/**
 * 目前狀態是否容許前景重檢（desktop-app「回到前景未滿一小時不重檢」）：只有
 * 沒有東西等使用者處置的狀態才重檢。待同意的提示（available）與安裝失敗的錯誤
 * （error）要留到使用者處置，不能被自動重檢清掉；檢查中、下載中、待重啟不重入。
 */
export function focusRecheckAllowed(state: UpdaterState): boolean {
  return state.phase === "idle" || state.phase === "upToDate" || state.phase === "checkFailed";
}

export function reduceUpdater(state: UpdaterState, event: UpdaterEvent): UpdaterState {
  switch (event.type) {
    case "checkStarted":
      // 下載中與待重啟不可被重檢打斷；其餘狀態（含錯誤、已最新）都可再檢查。
      if (state.phase === "downloading" || state.phase === "restartPending") return state;
      return { phase: "checking", manual: event.manual };
    case "updateFound":
      return state.phase === "checking" ? { phase: "available", version: event.version } : state;
    case "noUpdate":
      if (state.phase !== "checking") return state;
      return state.manual ? { phase: "upToDate" } : { phase: "idle" };
    case "checkFailed":
      if (state.phase !== "checking") return state;
      return state.manual ? { phase: "checkFailed" } : { phase: "idle" };
    case "accepted":
      return state.phase === "available"
        ? { phase: "downloading", version: state.version }
        : state;
    case "dismissed":
      // available＝稍後、error＝關閉錯誤提示；皆回閒置。
      return state.phase === "available" || state.phase === "error" ? { phase: "idle" } : state;
    case "downloaded":
      return state.phase === "downloading"
        ? { phase: "restartPending", version: state.version }
        : state;
    case "installFailed":
      return state.phase === "downloading" ? { phase: "error", message: event.message } : state;
  }
}
