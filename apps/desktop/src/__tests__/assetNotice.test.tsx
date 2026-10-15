// 技能檔提示卡與確認框（desktop-app spec「指令檔過期提示」；desktop-notice-relocation
// design D1）：卡在專案欄底部、三態文案帶檔案數、整張可點開框；框內三列資訊、按鈕依
// 狀態（過期／缺失：稍後、保留現狀、更新／安裝；較新：保留現狀、更新 Speclink）；更新
// 失敗的錯誤留在內容卡下方可重試。原 assetUpdatePrompt.test.tsx 的案例逐條搬到這裡。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, SEMANTIC_TONE } from "@speclink/ui";

import type { AssetPromptState } from "../assetPrompt";
import { AssetNoticeCard } from "../components/AssetNoticeCard";
import { AssetNoticeDialog, type AssetNoticeDialogProps } from "../components/AssetNoticeDialog";
import { APP_MESSAGES } from "../i18n/messages";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const STALE: AssetPromptState = { kind: "stale", fileCount: 3, version: "v1.3.0", projectVersion: "v0.9.0" };
const MISSING: AssetPromptState = { kind: "missing", fileCount: 12, version: "v1.3.0", projectVersion: null };
const NEWER: AssetPromptState = { kind: "newer", fileCount: 2, version: "v1.3.0", projectVersion: "v1.4.0" };

const button = (name: string) => screen.getByRole("button", { name }) as HTMLButtonElement;

describe("AssetNoticeCard", () => {
  it.each([
    ["過期", STALE, "技能檔是舊版", "更新會改寫 3 個檔案"],
    ["缺失", MISSING, "還沒安裝技能檔", "安裝會新建 12 個檔案"],
    ["較新", NEWER, "技能檔比 Speclink 新", "更新 Speclink 才能安全改寫"],
  ])("%s態：一句標題、一句說明（代入檔案數）", (_label, prompt, title, desc) => {
    render(<AssetNoticeCard prompt={prompt} onOpen={vi.fn()} />);
    const card = screen.getByTestId("asset-notice-card");
    expect(card.getAttribute("aria-label")).toBe(title);
    expect(card.textContent).toContain(title);
    expect(card.textContent).toContain(desc);
  });

  it("整張可點：呼叫 onOpen；琥珀淡底、琥珀圖示、非 dialog 語意", () => {
    const onOpen = vi.fn();
    render(<AssetNoticeCard prompt={STALE} onOpen={onOpen} />);
    const card = screen.getByRole("button", { name: "技能檔是舊版" });
    fireEvent.click(card);
    expect(onOpen).toHaveBeenCalledTimes(1);
    expect(card.className).toContain("bg-status-warning/10");
    expect(card.className).toContain("border-status-warning/40");
    expect(card.querySelector("svg")?.getAttribute("class")).toContain(SEMANTIC_TONE.warning);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("無提示時不渲染任何內容", () => {
    render(<AssetNoticeCard prompt={null} onOpen={vi.fn()} />);
    expect(screen.queryByTestId("asset-notice-card")).toBeNull();
  });
});

function dialogProps(over: Partial<AssetNoticeDialogProps> = {}): AssetNoticeDialogProps {
  return {
    open: true,
    onOpenChange: vi.fn(),
    prompt: STALE,
    busy: false,
    error: null,
    onApply: vi.fn(),
    onDismiss: vi.fn(),
    onOpenSettingsUpdate: vi.fn(),
    ...over,
  };
}

const rowValues = () =>
  within(screen.getByTestId("asset-notice-rows"))
    .getAllByTestId("asset-notice-value")
    .map((el) => el.textContent);

describe("AssetNoticeDialog", () => {
  it("框內三列：這個專案的技能檔版本、你的 Speclink 版本、會改寫的檔案數——前兩列同一套技能檔版號", () => {
    render(<AssetNoticeDialog {...dialogProps()} />);
    const rows = screen.getByTestId("asset-notice-rows");
    for (const label of ["這個專案的技能檔版本", "你的 Speclink 版本", "會改寫的檔案數"]) {
      expect(rows.textContent).toContain(label);
    }
    // 第二列是 Speclink 這版帶的技能檔版號（探測的 currentVersion），不是 app 發版號：兩列比得出新舊。
    expect(rowValues()).toEqual(["v0.9.0", "v1.3.0", "3"]);
  });

  it("從未安裝：專案技能檔版本顯示「—」", () => {
    render(<AssetNoticeDialog {...dialogProps({ prompt: MISSING })} />);
    expect(rowValues()).toEqual(["—", "v1.3.0", "12"]);
  });

  it("較新態：第三列改為會被換回舊內容的檔案數", () => {
    render(<AssetNoticeDialog {...dialogProps({ prompt: NEWER })} />);
    const rows = screen.getByTestId("asset-notice-rows");
    expect(rows.textContent).toContain("會被換回舊內容的檔案數");
    expect(rows.textContent).not.toContain("會改寫的檔案數");
    expect(rowValues()).toEqual(["v1.4.0", "v1.3.0", "2"]);
  });

  it.each([
    ["過期", STALE, ["稍後", "保留現狀", "更新技能檔"]],
    ["缺失", MISSING, ["稍後", "保留現狀", "安裝技能檔"]],
    ["較新", NEWER, ["保留現狀", "更新 Speclink"]],
  ])("%s態的按鈕依序", (_label, prompt, labels) => {
    render(<AssetNoticeDialog {...dialogProps({ prompt })} />);
    const footer = screen.getByTestId("asset-notice-actions");
    expect(within(footer).getAllByRole("button").map((b) => b.textContent)).toEqual(labels);
  });

  it("較新態：標題與說明以「app 本體是舊版」語意呈現，無任何改寫動作", () => {
    render(<AssetNoticeDialog {...dialogProps({ prompt: NEWER })} />);
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("比你的 Speclink 新");
    expect(dialog.textContent).toContain("你的 Speclink 是舊版");
    expect(dialog.querySelector("svg")?.getAttribute("class")).toContain(SEMANTIC_TONE.warning);
    expect(screen.queryByRole("button", { name: "更新技能檔" })).toBeNull();
    expect(screen.queryByRole("button", { name: "安裝技能檔" })).toBeNull();
  });

  it("「稍後」只關框：呼叫 onOpenChange(false)，不 dismiss、不 apply", () => {
    const p = dialogProps();
    render(<AssetNoticeDialog {...p} />);
    fireEvent.click(button("稍後"));
    expect(p.onOpenChange).toHaveBeenCalledWith(false);
    expect(p.onDismiss).not.toHaveBeenCalled();
    expect(p.onApply).not.toHaveBeenCalled();
  });

  it("「保留現狀」呼叫 onDismiss 並關框，不觸發再生", () => {
    const p = dialogProps();
    render(<AssetNoticeDialog {...p} />);
    fireEvent.click(button("保留現狀"));
    expect(p.onDismiss).toHaveBeenCalledTimes(1);
    expect(p.onOpenChange).toHaveBeenLastCalledWith(false);
    expect(p.onApply).not.toHaveBeenCalled();
  });

  it("「更新技能檔」呼叫 onApply；框不自行關閉（成功由提示消失收合、失敗留在框內）", () => {
    const p = dialogProps();
    render(<AssetNoticeDialog {...p} />);
    fireEvent.click(button("更新技能檔"));
    expect(p.onApply).toHaveBeenCalledTimes(1);
    expect(p.onOpenChange).not.toHaveBeenCalled();
    expect(p.onDismiss).not.toHaveBeenCalled();
  });

  it("缺失態「安裝技能檔」同樣走 onApply（從未安裝的專案不以更新稱之）", () => {
    const p = dialogProps({ prompt: MISSING });
    render(<AssetNoticeDialog {...p} />);
    expect(screen.queryByRole("button", { name: "更新技能檔" })).toBeNull();
    fireEvent.click(button("安裝技能檔"));
    expect(p.onApply).toHaveBeenCalledTimes(1);
  });

  it("較新態「更新 Speclink」呼叫 onOpenSettingsUpdate 並關框，不觸發再生", () => {
    const p = dialogProps({ prompt: NEWER });
    render(<AssetNoticeDialog {...p} />);
    fireEvent.click(button("更新 Speclink"));
    expect(p.onOpenSettingsUpdate).toHaveBeenCalledTimes(1);
    expect(p.onOpenChange).toHaveBeenLastCalledWith(false);
    expect(p.onApply).not.toHaveBeenCalled();
    expect(p.onDismiss).not.toHaveBeenCalled();
  });

  it("busy：全部動作鍵 disabled，避免重複觸發", () => {
    const p = dialogProps({ busy: true });
    render(<AssetNoticeDialog {...p} />);
    for (const name of ["稍後", "保留現狀", "更新技能檔"]) expect(button(name).disabled).toBe(true);
    fireEvent.click(button("更新技能檔"));
    expect(p.onApply).not.toHaveBeenCalled();
  });

  it("error：錯誤顯示於內容卡下方、紅字；主動作仍可重試", () => {
    const p = dialogProps({ error: "CLAUDE.md: permission denied" });
    render(<AssetNoticeDialog {...p} />);
    const rows = screen.getByTestId("asset-notice-rows");
    const alert = screen.getByRole("alert");
    expect(alert.textContent).toContain("permission denied");
    expect(alert.className).toContain("destructive");
    expect(rows.compareDocumentPosition(alert) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    const actions = screen.getByTestId("asset-notice-actions");
    expect(alert.compareDocumentPosition(actions) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    fireEvent.click(button("更新技能檔"));
    expect(p.onApply).toHaveBeenCalledTimes(1);
  });

  it("無提示時不渲染框", () => {
    render(<AssetNoticeDialog {...dialogProps({ prompt: null })} />);
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });
});
