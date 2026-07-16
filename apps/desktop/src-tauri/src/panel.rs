//! 系統匣面板（tray-status-menu「面板樣式（macOS）」；design D5）：lazy 建立
//! tray-panel webview 視窗並轉為 nonactivating NSPanel——開啟不奪前景 app 焦點、
//! 失焦（resign key）自動收合；window-vibrancy 套 HudWindow 材質、positioner 以
//! tray 相對位置貼齊圖示（座標由前端 handleIconState 餵入）。任一步建立失敗
//! 回單行 Err——前端據此退回原生選單樣式並於設定頁浮出。

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_nspanel::{tauri_panel, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt};
use tauri_plugin_positioner::{Position, WindowExt};
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

tauri_panel! {
    // 可成 key（面板內可互動、失焦可偵測）但不可成 main、不 activate app。
    panel!(TrayStatusPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })

    panel_event!(TrayStatusPanelEvents {
        window_did_resign_key(notification: &NSNotification) -> ()
    })
}

const PANEL_LABEL: &str = "tray-panel";
const PANEL_WIDTH: f64 = 320.0;
const PANEL_HEIGHT: f64 = 420.0;

/// 點擊系統匣圖示的開閉入口：不存在則 lazy 建立，存在則 toggle 顯隱。
/// 顯示前依 positioner 的 tray 座標貼齊圖示下方。
pub fn toggle(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_panel(PANEL_LABEL).is_err() {
        create(app)?;
    }
    let panel = app
        .get_webview_panel(PANEL_LABEL)
        .map_err(|_| "tray panel unavailable after creation".to_string())?;
    if panel.is_visible() {
        panel.hide();
        return Ok(());
    }
    if let Some(window) = app.get_webview_window(PANEL_LABEL) {
        window
            .move_window(Position::TrayBottomCenter)
            .map_err(|e| format!("tray panel positioning failed: {e}"))?;
    }
    // nonactivating panel：不 activate app、不把主視窗拉前景；成為 key 才收得到
    // resign key（失焦自動收合的觸發源）——Spotlight 式面板的標準姿勢。
    panel.show_and_make_key();
    Ok(())
}

/// 建立面板視窗：無邊框、透明、不進工作列、置頂、先隱藏；轉 NSPanel 後套
/// nonactivating style mask、浮動層級、vibrancy HudWindow 材質與失焦收合。
fn create(app: &AppHandle) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(app, PANEL_LABEL, WebviewUrl::App("panel.html".into()))
        .title("Speclink")
        .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .focused(false)
        .visible(false)
        // 全透明背景色不可省：wry 僅在明給 background_color 時才清 WKWebView 的
        // underPageBackgroundColor（macOS 12+ 預設不透明淺灰）——只設 transparent
        // 會被這層蓋住 vibrancy（實測為面板完全不透的根因）。
        .background_color(tauri::utils::config::Color(0, 0, 0, 0))
        .build()
        .map_err(|e| format!("tray panel window creation failed: {e}"))?;

    let panel = window
        .to_panel::<TrayStatusPanel>()
        .map_err(|e| format!("tray panel conversion failed: {e}"))?;
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
    panel.set_level(PanelLevel::Floating.value());
    panel.set_hides_on_deactivate(false);
    // vibrancy 前提：NSPanel 本身不透明會蓋掉 NSVisualEffectView——類別交換後
    // 明確重設（builder 的 transparent 設定不保證存續到 panel 類別）。
    panel.set_opaque(false);
    panel.set_transparent(true);
    // 毛玻璃：於類別交換「之後」明確套用（builder 期套用不保證存續；menubar app
    // 的實證路徑即 window-vibrancy 顯式呼叫）。材質選 HudWindow——透感最強、
    // 背後內容經 blur 可辨（design D6：Menu 為真 NSMenu 材質但淺色模式近乎
    // 不透、不滿足「毛玻璃底可透出」，Popover 更不透——實測裁決換用）。
    apply_vibrancy(
        &window,
        NSVisualEffectMaterial::HudWindow,
        Some(NSVisualEffectState::Active),
        Some(13.0),
    )
    .map_err(|e| format!("tray panel vibrancy failed: {e}"))?;

    // 失焦自動收合（spec：點面板外任意處面板收合）。
    let handler = TrayStatusPanelEvents::new();
    let handle = app.clone();
    handler.window_did_resign_key(move |_notification| {
        if let Ok(p) = handle.get_webview_panel(PANEL_LABEL) {
            p.hide();
        }
    });
    panel.set_event_handler(Some(handler.as_ref()));
    Ok(())
}
