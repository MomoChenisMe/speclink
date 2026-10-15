// CodeMirror 的測試輔助（desktop-settings-editors design D1）：讀寫編輯器內容與按快捷鍵。
// jsdom 缺的 Range 量測補在 vitest.setup.ts。
import { act, fireEvent } from "@testing-library/react";
import { EditorView } from "@codemirror/view";

/** container 內第一個（或唯一的）編輯器。 */
export function editorView(container: HTMLElement): EditorView {
  const dom = container.querySelector<HTMLElement>(".cm-editor");
  const view = dom && EditorView.findFromDOM(dom);
  if (!view) throw new Error("no CodeMirror editor in container");
  return view;
}

export const editorText = (container: HTMLElement) => editorView(container).state.doc.toString();

/** 使用者輸入事件：預設整份換成 text；`append` 時接在文件尾端。 */
export function typeInEditor(container: HTMLElement, text: string, { append = false } = {}) {
  const view = editorView(container);
  const to = view.state.doc.length;
  act(() =>
    view.dispatch({ changes: { from: append ? to : 0, to, insert: text }, userEvent: "input.type" }),
  );
}

/** 在編輯器內按 Mod-s（jsdom 非 macOS，Mod＝Ctrl）；回傳事件供檢查 defaultPrevented。 */
export function pressSave(container: HTMLElement): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { key: "s", ctrlKey: true, bubbles: true, cancelable: true });
  act(() => {
    fireEvent(editorView(container).contentDOM, event);
  });
  return event;
}
