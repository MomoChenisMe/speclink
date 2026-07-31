// 面板視窗入口（design D5）：薄渲染層——監聽主視窗推送的 tray-snapshot、
// 動作以 tray-panel-action 事件回流主視窗執行（不自建 store、不直呼資料查詢
// 指令）；複製直呼 clipboard 外掛（Rust 端，面板常態無焦點仍成功、失敗靜默）。
import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { I18nProvider } from "@speclink/ui";

import { TrayPanel } from "./TrayPanel";
import { APP_MESSAGES } from "../i18n/messages";
import { readLocalePreference, resolveUiLocale } from "../i18n/locale";
import type { TraySnapshot } from "../tray";
import "@fontsource-variable/noto-sans-tc";
import "../index.css";

// vibrancy 材質從視窗底透出：文件背景必須透明（index.css 的 body 底色讓開）。
document.documentElement.style.background = "transparent";
document.body.style.background = "transparent";

const PANEL_WIDTH = 320;
const PANEL_MAX_HEIGHT = 640;

function PanelRoot() {
  const [snap, setSnap] = useState<TraySnapshot | null>(null);
  useEffect(() => {
    const unlisten = listen<TraySnapshot>("tray-snapshot", (e) => setSnap(e.payload));
    // 面板 lazy 建立：掛好監聽後向主視窗請求一次快照，避免首開空窗。
    void emit("tray-panel-ready", {});
    return () => {
      void unlisten.then((f) => f());
    };
  }, []);
  // 視窗高度自適應內容（三段式版面）：root 以 h-screen 撐滿視窗，直接量 root
  // 會與 setSize 形成「視窗高⇄root 高」循環——改以「root 現高 − 中段可視高 ＋
  // 中段內容自然高」還原無約束總高（＝頁首＋中段自然內容高＋頁尾＋留白），
  // 上限 640 後由中段內部捲動，失敗靜默。
  useEffect(() => {
    const root = document.getElementById("root");
    if (!root) return;
    const win = getCurrentWindow();
    const fit = () => {
      const scroll = root.querySelector<HTMLElement>('[data-testid="panel-scroll"]');
      if (!scroll) return;
      const natural =
        root.getBoundingClientRect().height - scroll.clientHeight + scroll.scrollHeight;
      const h = Math.min(Math.ceil(natural), PANEL_MAX_HEIGHT);
      if (h > 0) void win.setSize(new LogicalSize(PANEL_WIDTH, h)).catch(() => {});
    };
    const observer = new ResizeObserver(fit);
    // 尺寸會變的是頁首、頁尾與中段內容 wrapper——中段容器本身被 flex 約束、
    // 內容增減不改其外框，觀察容器會漏掉內容變化；setSize 改的是 root 與
    // 中段容器（皆未被觀察），不會自迴圈。
    for (const testid of ["panel-header", "panel-scroll-content", "panel-footer"]) {
      const el = root.querySelector(`[data-testid="${testid}"]`);
      if (el) observer.observe(el);
    }
    fit();
    return () => observer.disconnect();
  }, []);
  const act = (kind: string, id?: string) => {
    void emit("tray-panel-action", { kind, id });
  };
  return (
    <TrayPanel
      snapshot={snap}
      onOpenProject={(key) => act("open-project", key)}
      onOpenChange={(name) => act("open-change", name)}
      onOpenDiscussion={(slug) => act("open-discussion", slug)}
      onOpenApp={() => act("open-app")}
      onOpenProjectSettings={() => act("open-project-settings")}
      onOpenSettings={() => act("open-settings")}
      onQuit={() => act("quit")}
      onAddProject={() => act("add-project")}
      onRetryWorkspace={(key) => act("retry-workspace", key)}
      onOpenRecovery={(key) => act("open-recovery", key)}
      onOpenServerSettings={(connectionId) => act("open-server-settings", connectionId)}
      onReauthenticate={(connectionId) => act("reauthenticate", connectionId)}
      onCopy={(text) => {
        void writeText(text).catch(() => {});
      }}
    />
  );
}

const root = document.getElementById("root");
if (root) {
  const uiLocale = resolveUiLocale(
    readLocalePreference(),
    typeof navigator !== "undefined" ? navigator.language : undefined,
  );
  createRoot(root).render(
    <StrictMode>
      <I18nProvider locale={uiLocale} messages={APP_MESSAGES}>
        <PanelRoot />
      </I18nProvider>
    </StrictMode>,
  );
}
