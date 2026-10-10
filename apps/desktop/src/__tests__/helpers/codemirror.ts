// CodeMirror 在 jsdom 的測試輔助（desktop-settings-editors design D1）：jsdom 沒有 Range 的
// 版面量測，CodeMirror 掛載時會直接呼叫——補回空量測；另提供讀寫與快捷鍵的操作。
import { act, fireEvent } from "@testing-library/react";
import { EditorView } from "@codemirror/view";

const emptyRect = () => new DOMRect(0, 0, 0, 0);
const emptyRects = () => Object.assign([], { item: () => null }) as unknown as DOMRectList;
if (!Range.prototype.getClientRects) Range.prototype.getClientRects = emptyRects;
if (!Range.prototype.getBoundingClientRect) Range.prototype.getBoundingClientRect = emptyRect;

/** container 內第一個（或唯一的）編輯器。 */
export function editorView(container: HTMLElement): EditorView {
  const dom = container.querySelector<HTMLElement>(".cm-editor");
  const view = dom && EditorView.findFromDOM(dom);
  if (!view) throw new Error("no CodeMirror editor in container");
  return view;
}

export const editorText = (container: HTMLElement) => editorView(container).state.doc.toString();

/** 在文件尾端輸入（使用者輸入事件）。 */
export function typeInEditor(container: HTMLElement, text: string) {
  const view = editorView(container);
  act(() =>
    view.dispatch({ changes: { from: view.state.doc.length, insert: text }, userEvent: "input.type" }),
  );
}

/** 整份換成 text（使用者輸入事件）。 */
export function replaceEditorText(container: HTMLElement, text: string) {
  const view = editorView(container);
  act(() =>
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: text },
      userEvent: "input.type",
    }),
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
