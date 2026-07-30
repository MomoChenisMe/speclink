// 更新狀態機（desktop-app spec「桌面自動更新」，design D6）：純 reducer、不依賴
// Tauri。自動檢查失敗靜默回閒置、手動檢查失敗才呈現無法檢查；簽章驗證失敗轉
// 錯誤態（不進待重啟＝既有安裝不受影響）；同意前不下載。
import { describe, it, expect, vi } from "vitest";

import {
  initialUpdaterState,
  reduceUpdater,
  type UpdaterState,
} from "../core/updater";
import type { PendingUpdate, UpdaterAdapter } from "../adapter/updater";
import { createAppStore } from "../store";

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
