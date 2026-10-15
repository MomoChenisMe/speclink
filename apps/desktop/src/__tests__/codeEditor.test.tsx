// CodeEditor（desktop-settings-editors design D1；spec「表單控制項與按鈕以主題化元件呈現」）：
// 測元件外殼行為——初值、onChange 全文、外部 value 同步、Mod-s、唯讀、狀態列、行號；不測算繪。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, act } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { CodeEditor } from "../components/CodeEditor";
import { APP_MESSAGES } from "../i18n/messages";
import { editorText, editorView, pressSave, typeInEditor } from "./helpers/codemirror";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper: zhWrapper });

function mount(over: Partial<Parameters<typeof CodeEditor>[0]> = {}) {
  const onChange = vi.fn();
  const onSave = vi.fn();
  const props = {
    value: "# 標題\n\n第二段",
    onChange,
    onSave,
    mode: "markdown" as const,
    ariaLabel: "專案說明",
    ...over,
  };
  const utils = render(
    <div data-testid="host">
      <CodeEditor {...props} />
    </div>,
  );
  return { ...utils, onChange, onSave, host: screen.getByTestId("host"), props };
}

describe("CodeEditor", () => {
  it("掛載後呈現初值，內容區為帶名稱的 textbox", () => {
    const { host } = mount();
    expect(editorText(host)).toBe("# 標題\n\n第二段");
    expect(screen.getByRole("textbox", { name: "專案說明" })).toBeTruthy();
  });

  it("輸入後 onChange 收到全文", () => {
    const { host, onChange } = mount();
    typeInEditor(host, "\n加一行", { append: true });
    expect(onChange).toHaveBeenLastCalledWith("# 標題\n\n第二段\n加一行");
  });

  it("value 外部變動反映到編輯器，且不回送 onChange", () => {
    const { host, onChange, rerender, props } = mount();
    rerender(
      <div data-testid="host">
        <CodeEditor {...props} value="外部換掉的內容" />
      </div>,
    );
    expect(editorText(host)).toBe("外部換掉的內容");
    expect(onChange).not.toHaveBeenCalled();
  });

  it("Mod-s 呼叫 onSave 並攔下瀏覽器預設", () => {
    const { host, onSave } = mount();
    const event = pressSave(host);
    expect(onSave).toHaveBeenCalledTimes(1);
    expect(event.defaultPrevented).toBe(true);
  });

  it("readOnly 時 Mod-s 不呼叫 onSave，也不攔下事件（交給頁層處理）", () => {
    const { host, onSave } = mount({ readOnly: true });
    const event = pressSave(host);
    expect(onSave).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
  });

  it("readOnly 不可輸入，但內容區仍可聚焦（焦點不被移走）", () => {
    const { host } = mount({ readOnly: true });
    const view = editorView(host);
    expect(view.state.readOnly).toBe(true);
    expect(view.contentDOM.getAttribute("aria-readonly")).toBe("true");
    expect(view.contentDOM.getAttribute("contenteditable")).toBe("true");
  });

  it("狀態列顯示模式、自動換行與游標行欄", () => {
    const { host } = mount();
    expect(host.textContent).toContain("markdown · 自動換行");
    expect(host.textContent).toContain("第 1 行，第 1 欄");
    const view = editorView(host);
    act(() => view.dispatch({ selection: { anchor: view.state.doc.line(3).from + 2 } }));
    expect(host.textContent).toContain("第 3 行，第 3 欄");
  });

  it("純文字模式的狀態列標示純文字", () => {
    const { host } = mount({ mode: "plain", value: "一行" });
    expect(host.textContent).toContain("純文字 · 自動換行");
  });

  it("markdown 模式有行號節點", () => {
    const { host } = mount();
    expect(host.querySelector(".cm-lineNumbers .cm-gutterElement")).not.toBeNull();
  });

  it("markdown 模式的標題行帶語法上色節點，純文字模式沒有", () => {
    const { host, unmount } = mount();
    expect(host.querySelector(".cm-line span")).not.toBeNull();
    unmount();
    const plain = mount({ mode: "plain" });
    expect(plain.host.querySelector(".cm-line span")).toBeNull();
  });
});
