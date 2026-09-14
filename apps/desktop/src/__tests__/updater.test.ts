// 更新狀態機（desktop-app spec「桌面自動更新」，design D6）：純 reducer、不依賴
// Tauri。自動檢查失敗靜默回閒置、手動檢查失敗才呈現無法檢查；簽章驗證失敗轉
// 錯誤態（不進待重啟＝既有安裝不受影響）；同意前不下載。
import { describe, it, expect, vi } from "vitest";

import {
  FOCUS_RECHECK_INTERVAL_MS,
  focusRecheckAllowed,
  focusRecheckDue,
  initialUpdaterState,
  reduceUpdater,
  type UpdaterState,
} from "../core/updater";
import type { PendingUpdate, UpdaterAdapter } from "../adapter/updater";
import { createAppStore } from "../store";

// 前景重檢測試的共用時間軸（規格 Example「節流邊界」以 09:00 為上次檢查）。
const T0 = Date.UTC(2026, 0, 1, 9, 0, 0);
const MINUTES = 60 * 1000;

/// 依序餵事件，回傳最終狀態。
function replay(events: Parameters<typeof reduceUpdater>[1][]): UpdaterState {
  return events.reduce(reduceUpdater, initialUpdaterState);
}

describe("更新狀態機（core/updater）", () => {
  it("閒置→檢查中→發現新版（含目標版本）→同意→下載→待重啟", () => {
    let state: UpdaterState = initialUpdaterState;
    expect(state).toEqual({ phase: "idle" });

    state = reduceUpdater(state, { type: "checkStarted", manual: false });
    expect(state).toEqual({ phase: "checking", manual: false });

    state = reduceUpdater(state, { type: "updateFound", version: "0.2.0" });
    expect(state).toEqual({ phase: "available", version: "0.2.0" });

    state = reduceUpdater(state, { type: "accepted" });
    expect(state).toEqual({ phase: "downloading", version: "0.2.0" });

    state = reduceUpdater(state, { type: "downloaded" });
    expect(state).toEqual({ phase: "restartPending", version: "0.2.0" });
  });

  it("自動檢查失敗（離線）靜默回閒置", () => {
    const state = replay([
      { type: "checkStarted", manual: false },
      { type: "checkFailed" },
    ]);
    expect(state).toEqual({ phase: "idle" });
  });

  it("手動檢查失敗才呈現無法檢查更新", () => {
    const state = replay([
      { type: "checkStarted", manual: true },
      { type: "checkFailed" },
    ]);
    expect(state).toEqual({ phase: "checkFailed" });
  });

  it("手動檢查且已是最新時回報已最新", () => {
    const state = replay([
      { type: "checkStarted", manual: true },
      { type: "noUpdate" },
    ]);
    expect(state).toEqual({ phase: "upToDate" });
  });

  it("自動檢查已最新則靜默回閒置（不打擾）", () => {
    const state = replay([
      { type: "checkStarted", manual: false },
      { type: "noUpdate" },
    ]);
    expect(state).toEqual({ phase: "idle" });
  });

  it("發現新版但使用者稍後：回閒置、不下載", () => {
    const state = replay([
      { type: "checkStarted", manual: false },
      { type: "updateFound", version: "0.2.0" },
      { type: "dismissed" },
    ]);
    expect(state).toEqual({ phase: "idle" });
  });

  it("下載中簽章驗證失敗：轉錯誤態、不進待重啟（既有安裝不受影響）", () => {
    const state = replay([
      { type: "checkStarted", manual: false },
      { type: "updateFound", version: "0.2.0" },
      { type: "accepted" },
      { type: "installFailed", message: "signature verification failed" },
    ]);
    expect(state).toEqual({
      phase: "error",
      message: "signature verification failed",
    });
  });

  it("非法事件不改變狀態（閒置時收到 downloaded 仍為閒置）", () => {
    const state = reduceUpdater(initialUpdaterState, { type: "downloaded" });
    expect(state).toEqual({ phase: "idle" });
  });

  it("錯誤態可關閉回閒置", () => {
    const state = reduceUpdater(
      { phase: "error", message: "invalid signature" },
      { type: "dismissed" },
    );
    expect(state).toEqual({ phase: "idle" });
  });
});

// --- 前景重檢節流（desktop-app「回到前景未滿一小時不重檢」Example「節流邊界」） ---

describe("前景重檢節流判定（core/updater focusRecheckDue）", () => {
  it("節流間隔為 1 小時", () => {
    expect(FOCUS_RECHECK_INTERVAL_MS).toBe(60 * 60 * 1000);
  });

  it("從未檢查（無時間戳）視為逾時", () => {
    expect(focusRecheckDue(null, T0)).toBe(true);
  });

  it.each([
    ["09:20 未滿 1 小時 → 不重檢", 20 * MINUTES, false],
    ["09:59:59.999 差 1 毫秒 → 不重檢", FOCUS_RECHECK_INTERVAL_MS - 1, false],
    ["10:00:00 剛好滿 1 小時 → 重檢", FOCUS_RECHECK_INTERVAL_MS, true],
    ["12:00 超過 1 小時 → 重檢", 180 * MINUTES, true],
  ])("上次 09:00、切回 %s", (_label, elapsed, due) => {
    expect(focusRecheckDue(T0, T0 + elapsed)).toBe(due);
  });
});

// 哪些狀態容許前景重檢（desktop-app「回到前景未滿一小時不重檢」Example 的
// 待同意／錯誤／下載中／待重啟四列）：只有沒有東西等使用者處置的狀態才重檢。
describe("前景重檢的狀態守門（core/updater focusRecheckAllowed）", () => {
  it.each<[UpdaterState, boolean]>([
    [{ phase: "idle" }, true],
    [{ phase: "upToDate" }, true],
    [{ phase: "checkFailed" }, true],
    [{ phase: "checking", manual: false }, false],
    [{ phase: "available", version: "0.5.1" }, false],
    [{ phase: "downloading", version: "0.5.1" }, false],
    [{ phase: "restartPending", version: "0.5.1" }, false],
    [{ phase: "error", message: "invalid signature" }, false],
  ])("%o → %s", (state, allowed) => {
    expect(focusRecheckAllowed(state)).toBe(allowed);
  });
});

// --- store 接線（design D6：plugin 事件經 adapter 注入，store 只驅動 reducer） ---

function storeWith(adapter?: UpdaterAdapter) {
  return createAppStore({
    createSession: vi.fn() as never,
    ...(adapter ? { updater: adapter } : {}),
  });
}

describe("更新 store 接線", () => {
  it("checkForUpdates 找到新版：狀態轉 available 並帶版本", async () => {
    const pending: PendingUpdate = {
      version: "0.2.0",
      downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    };
    const store = storeWith({ check: vi.fn().mockResolvedValue(pending), relaunch: vi.fn() });

    await store.getState().checkForUpdates(false);
    expect(store.getState().updater).toEqual({ phase: "available", version: "0.2.0" });
  });

  it("同意後下載套用成功：downloadAndInstall 恰被呼叫一次、轉待重啟", async () => {
    const pending: PendingUpdate = {
      version: "0.2.0",
      downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    };
    const store = storeWith({ check: vi.fn().mockResolvedValue(pending), relaunch: vi.fn() });

    await store.getState().checkForUpdates(false);
    await store.getState().acceptUpdate();
    expect(pending.downloadAndInstall).toHaveBeenCalledTimes(1);
    expect(store.getState().updater).toEqual({ phase: "restartPending", version: "0.2.0" });
  });

  it("下載套用被拒（簽章驗證失敗）：轉錯誤態並帶訊息", async () => {
    const pending: PendingUpdate = {
      version: "0.2.0",
      downloadAndInstall: vi.fn().mockRejectedValue(new Error("invalid signature")),
    };
    const store = storeWith({ check: vi.fn().mockResolvedValue(pending), relaunch: vi.fn() });

    await store.getState().checkForUpdates(false);
    await store.getState().acceptUpdate();
    expect(store.getState().updater).toEqual({ phase: "error", message: "invalid signature" });
  });

  it("check reject：自動檢查靜默回閒置、手動檢查浮出無法檢查", async () => {
    const adapter: UpdaterAdapter = {
      check: vi.fn().mockRejectedValue(new Error("offline")),
      relaunch: vi.fn(),
    };
    const auto = storeWith(adapter);
    await auto.getState().checkForUpdates(false);
    expect(auto.getState().updater).toEqual({ phase: "idle" });

    const manual = storeWith(adapter);
    await manual.getState().checkForUpdates(true);
    expect(manual.getState().updater).toEqual({ phase: "checkFailed" });
  });

  it("未注入 adapter 時 checkForUpdates 為 no-op", async () => {
    const store = storeWith();
    await store.getState().checkForUpdates(true);
    expect(store.getState().updater).toEqual({ phase: "idle" });
  });
});

// --- 前景重檢（desktop-app「回到前景逾一小時即重檢」「回到前景未滿一小時不重檢」；design D2） ---

describe("前景重檢 store 接線（recheckOnFocus）", () => {
  function adapterUpToDate(): UpdaterAdapter {
    return { check: vi.fn().mockResolvedValue(null), relaunch: vi.fn() };
  }

  it("從未檢查時切回前景即檢查一次", async () => {
    const adapter = adapterUpToDate();
    const store = storeWith(adapter);

    await store.getState().recheckOnFocus(T0);
    expect(adapter.check).toHaveBeenCalledTimes(1);
  });

  it("啟動檢查後 30 分鐘切回前景不重檢；滿 1 小時再切回恰檢查一次", async () => {
    const adapter = adapterUpToDate();
    const store = storeWith(adapter);
    vi.useFakeTimers();
    try {
      vi.setSystemTime(T0);
      await store.getState().checkForUpdates(false);
      expect(adapter.check).toHaveBeenCalledTimes(1);

      await store.getState().recheckOnFocus(T0 + 30 * MINUTES);
      expect(adapter.check).toHaveBeenCalledTimes(1);

      await store.getState().recheckOnFocus(T0 + 60 * MINUTES);
      expect(adapter.check).toHaveBeenCalledTimes(2);
      expect(store.getState().updater).toEqual({ phase: "idle" }); // 已最新：不顯示任何提示
    } finally {
      vi.useRealTimers();
    }
  });

  it("Example「開著三小時後切回」：09:00 啟動時最新、期間發布 0.5.1、12:00 切回 → 提示 0.5.1 等待同意", async () => {
    const released: PendingUpdate = {
      version: "0.5.1",
      downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    };
    const check = vi.fn().mockResolvedValueOnce(null).mockResolvedValueOnce(released);
    const adapter: UpdaterAdapter = { check, relaunch: vi.fn() };
    const store = storeWith(adapter);
    vi.useFakeTimers();
    try {
      vi.setSystemTime(T0); // 09:00 啟動檢查：當時最新
      await store.getState().checkForUpdates(false);
      expect(store.getState().updater).toEqual({ phase: "idle" });

      await store.getState().recheckOnFocus(T0 + 180 * MINUTES); // 12:00 切回
      expect(check).toHaveBeenCalledTimes(2);
      expect(store.getState().updater).toEqual({ phase: "available", version: "0.5.1" });
      expect(released.downloadAndInstall).not.toHaveBeenCalled(); // 等待同意，不自動下載
    } finally {
      vi.useRealTimers();
    }
  });

  it("Example 節流邊界「09:00 啟動、09:40 手動檢查 → 10:20 切回不重檢」：手動檢查也重置時間戳", async () => {
    const adapter = adapterUpToDate();
    const store = storeWith(adapter);
    vi.useFakeTimers();
    try {
      vi.setSystemTime(T0); // 09:00 啟動檢查
      await store.getState().checkForUpdates(false);
      vi.setSystemTime(T0 + 40 * MINUTES); // 09:40 手動檢查
      await store.getState().checkForUpdates(true);
      expect(adapter.check).toHaveBeenCalledTimes(2);

      await store.getState().recheckOnFocus(T0 + 80 * MINUTES); // 10:20
      expect(adapter.check).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
    }
  });

  it("Example 節流邊界「09:00 檢查、09:30 起下載中 → 10:30 切回不重檢」", async () => {
    const pending: PendingUpdate = {
      version: "0.5.1",
      downloadAndInstall: vi.fn(() => new Promise<void>(() => {})), // 永不結束＝停在下載中
    };
    const adapter: UpdaterAdapter = { check: vi.fn().mockResolvedValue(pending), relaunch: vi.fn() };
    const store = storeWith(adapter);
    vi.useFakeTimers();
    try {
      vi.setSystemTime(T0); // 09:00 檢查到新版
      await store.getState().checkForUpdates(false);
      vi.setSystemTime(T0 + 30 * MINUTES); // 09:30 同意、開始下載
      void store.getState().acceptUpdate();
      expect(store.getState().updater).toEqual({ phase: "downloading", version: "0.5.1" });

      await store.getState().recheckOnFocus(T0 + 90 * MINUTES); // 10:30
      expect(adapter.check).toHaveBeenCalledTimes(1);
      expect(store.getState().updater).toEqual({ phase: "downloading", version: "0.5.1" });
    } finally {
      vi.useRealTimers();
    }
  });

  it("Example 節流邊界「09:00 檢查到新版、提示待同意 → 10:30 切回不重檢」：提示留到使用者處置", async () => {
    const pending: PendingUpdate = {
      version: "0.5.1",
      downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    };
    const adapter: UpdaterAdapter = { check: vi.fn().mockResolvedValue(pending), relaunch: vi.fn() };
    const store = storeWith(adapter);
    vi.useFakeTimers();
    try {
      vi.setSystemTime(T0);
      await store.getState().checkForUpdates(false);
      expect(store.getState().updater).toEqual({ phase: "available", version: "0.5.1" });

      await store.getState().recheckOnFocus(T0 + 90 * MINUTES);
      expect(adapter.check).toHaveBeenCalledTimes(1);
      expect(store.getState().updater).toEqual({ phase: "available", version: "0.5.1" });
    } finally {
      vi.useRealTimers();
    }
  });

  it("Example 節流邊界「09:00 安裝失敗顯示錯誤 → 10:30 切回不重檢」：錯誤訊息留到使用者關閉", async () => {
    const pending: PendingUpdate = {
      version: "0.5.1",
      downloadAndInstall: vi.fn().mockRejectedValue(new Error("invalid signature")),
    };
    const adapter: UpdaterAdapter = { check: vi.fn().mockResolvedValue(pending), relaunch: vi.fn() };
    const store = storeWith(adapter);
    vi.useFakeTimers();
    try {
      vi.setSystemTime(T0);
      await store.getState().checkForUpdates(false);
      await store.getState().acceptUpdate();
      expect(store.getState().updater).toEqual({ phase: "error", message: "invalid signature" });

      await store.getState().recheckOnFocus(T0 + 90 * MINUTES);
      expect(adapter.check).toHaveBeenCalledTimes(1);
      expect(store.getState().updater).toEqual({ phase: "error", message: "invalid signature" });
    } finally {
      vi.useRealTimers();
    }
  });

  it("Example 節流邊界「09:00 下載完成待重啟 → 10:30 切回不重檢」", async () => {
    const pending: PendingUpdate = {
      version: "0.5.1",
      downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    };
    const adapter: UpdaterAdapter = { check: vi.fn().mockResolvedValue(pending), relaunch: vi.fn() };
    const store = storeWith(adapter);
    vi.useFakeTimers();
    try {
      vi.setSystemTime(T0);
      await store.getState().checkForUpdates(false);
      await store.getState().acceptUpdate();
      expect(store.getState().updater).toEqual({ phase: "restartPending", version: "0.5.1" });

      await store.getState().recheckOnFocus(T0 + 90 * MINUTES);
      expect(adapter.check).toHaveBeenCalledTimes(1);
      expect(store.getState().updater).toEqual({ phase: "restartPending", version: "0.5.1" });
    } finally {
      vi.useRealTimers();
    }
  });

  it("recheckOnFocus 未給 now 時以現在時刻判定", async () => {
    const adapter = adapterUpToDate();
    const store = storeWith(adapter);
    vi.useFakeTimers();
    try {
      vi.setSystemTime(T0);
      await store.getState().checkForUpdates(false);
      vi.setSystemTime(T0 + 61 * MINUTES);
      await store.getState().recheckOnFocus();
      expect(adapter.check).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
    }
  });

  it("未注入 adapter 時 recheckOnFocus 為 no-op", async () => {
    const store = storeWith();
    await store.getState().recheckOnFocus(T0);
    expect(store.getState().updater).toEqual({ phase: "idle" });
  });
});
