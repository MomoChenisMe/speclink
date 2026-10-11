// 程式碼編輯器（desktop-settings-editors design D1）：包 CodeMirror 6——markdown／純文字兩模式、
// 行號、自動換行、Mod-s 儲存與底部狀態列；配色只取設計 token，蓋掉套件基底主題的每個顏色
// （spec「表單控制項與按鈕以主題化元件呈現」）。
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Annotation, Compartment, EditorState } from "@codemirror/state";
import {
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { markdownLanguage } from "@codemirror/lang-markdown";
import { tags } from "@lezer/highlight";
import { useI18n } from "@speclink/ui";

export interface CodeEditorProps {
  value: string;
  onChange: (value: string) => void;
  mode: "markdown" | "plain";
  readOnly?: boolean;
  onSave?: () => void;
  /** 內容區最小高度（行數）。 */
  minLines?: number;
  ariaLabel: string;
  autoFocus?: boolean;
}

const LINE_HEIGHT = 1.7;

const tokenTheme = EditorView.theme({
  "&": { backgroundColor: "transparent", color: "var(--foreground)", fontSize: "13px" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: String(LINE_HEIGHT) },
  ".cm-content": { caretColor: "var(--foreground)", padding: "6px 0" },
  ".cm-line": { padding: "0 16px 0 12px" },
  ".cm-gutters": {
    backgroundColor: "var(--muted)",
    color: "var(--muted-foreground)",
    borderRight: "1px solid var(--border)",
  },
  ".cm-lineNumbers .cm-gutterElement": { padding: "0 10px 0 12px" },
  ".cm-activeLine": { backgroundColor: "color-mix(in srgb, var(--primary) 6%, transparent)" },
  ".cm-activeLineGutter": { backgroundColor: "transparent", color: "var(--foreground)" },
  ".cm-specialChar": { color: "var(--destructive)" },
});

const markdownHighlight = HighlightStyle.define([
  { tag: tags.heading, fontWeight: "600", color: "var(--foreground)" },
  { tag: tags.monospace, backgroundColor: "var(--muted)", borderRadius: "3px" },
  { tag: [tags.link, tags.url], color: "var(--primary)" },
  { tag: tags.strong, fontWeight: "600" },
  { tag: tags.emphasis, fontStyle: "italic" },
]);

/** 外部 value 同步的 transaction 標記：這類變動不回送 onChange。 */
const syncFromProps = Annotation.define<boolean>();

const cursorOf = (state: EditorState) => {
  const head = state.selection.main.head;
  const line = state.doc.lineAt(head);
  return { line: line.number, col: head - line.from + 1 };
};

export function CodeEditor({
  value,
  onChange,
  mode,
  readOnly = false,
  onSave,
  minLines,
  ariaLabel,
  autoFocus,
}: CodeEditorProps) {
  const { t } = useI18n();
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  // 唯讀只設 EditorState.readOnly：輸入被忽略，但內容區仍可聚焦——關掉 editable 會讓
  // WebView 把焦點移走，儲存失敗恢復可編輯後焦點也回不來。
  const readOnlyMode = useRef(new Compartment()).current;
  // 編輯器只建一次，擴充內的回呼經 ref 取最新 props。
  const latest = useRef({ onChange, onSave });
  latest.current = { onChange, onSave };
  const [cursor, setCursor] = useState({ line: 1, col: 1 });

  useEffect(() => {
    const created = new EditorView({
      parent: host.current!,
      state: EditorState.create({
        doc: value,
        extensions: [
          lineNumbers(),
          highlightActiveLine(),
          highlightActiveLineGutter(),
          EditorView.lineWrapping,
          history(),
          keymap.of([
            {
              key: "Mod-s",
              // 唯讀時不攔：事件交給頁層，由它決定是否儲存。
              run: (view) => {
                if (!latest.current.onSave || view.state.readOnly) return false;
                latest.current.onSave();
                return true;
              },
            },
            ...defaultKeymap,
            ...historyKeymap,
            indentWithTab,
          ]),
          readOnlyMode.of(EditorState.readOnly.of(readOnly)),
          EditorView.contentAttributes.of({
            "aria-label": ariaLabel,
            ...(minLines ? { style: `min-height: ${minLines * LINE_HEIGHT}em` } : {}),
          }),
          mode === "markdown" ? [markdownLanguage, syntaxHighlighting(markdownHighlight)] : [],
          tokenTheme,
          EditorView.updateListener.of((update) => {
            if (update.docChanged && !update.transactions.some((tr) => tr.annotation(syncFromProps))) {
              latest.current.onChange(update.state.doc.toString());
            }
            if (update.docChanged || update.selectionSet) setCursor(cursorOf(update.state));
          }),
        ],
      }),
    });
    view.current = created;
    if (autoFocus) created.focus();
    return () => {
      created.destroy();
      view.current = null;
    };
    // 只在掛載時建立：mode／ariaLabel／minLines／autoFocus 由呼叫端固定，value 與 readOnly 由下方同步。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // layout effect：commit 後立刻同步，不留讓新按鍵插進來、再被舊 value 蓋掉的空檔。
  useLayoutEffect(() => {
    const current = view.current;
    if (!current || current.state.doc.toString() === value) return;
    current.dispatch({
      changes: { from: 0, to: current.state.doc.length, insert: value },
      selection: { anchor: Math.min(current.state.selection.main.head, value.length) },
      annotations: syncFromProps.of(true),
    });
  }, [value]);

  useEffect(() => {
    view.current?.dispatch({ effects: readOnlyMode.reconfigure(EditorState.readOnly.of(readOnly)) });
  }, [readOnly, readOnlyMode]);

  return (
    <div
      data-slot="code-editor"
      className="flex flex-col overflow-hidden rounded-xl border border-border bg-card font-mono transition-colors focus-within:border-ring focus-within:ring-2 focus-within:ring-ring/30"
    >
      <div ref={host} />
      <div className="flex items-center justify-between gap-3 border-t border-border px-3 py-1.5 text-[11px] text-muted-foreground">
        <span>
          {t(mode === "markdown" ? "settings.editor.markdownMode" : "settings.editor.plainMode")} ·{" "}
          {t("settings.editor.wrap")}
        </span>
        <span>
          {t("settings.editor.cursor")
            .replace("{line}", String(cursor.line))
            .replace("{col}", String(cursor.col))}
        </span>
      </div>
    </div>
  );
}
