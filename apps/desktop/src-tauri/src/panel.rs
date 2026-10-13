//! 系統匣面板（tray-status-menu「面板樣式」）：lazy 建立 tray-panel webview 視窗，
//! 對外只露出 `toggle` 與 `hide`，各平台的視窗實作分在各自的區塊（tray-panel-windows
//! design D6）。建立失敗、macOS 定位失敗或 Windows 顯示失敗時回單行 Err——前端據此退回
//! 原生選單樣式並於設定頁浮出；Windows 的收起與定位失敗不回錯。
//!
//! - macOS（design D5）：轉為 nonactivating NSPanel——開啟不奪前景 app 焦點、失焦
//!   （resign key）自動收合；window-vibrancy 套 HudWindow 材質、positioner 以 tray
//!   相對位置貼齊圖示下方（座標由前端 handleIconState 餵入）。
//! - Windows（tray-panel-windows design D1／D3／D4）：不透明、無外框、帶系統陰影的
//!   一般視窗；依系統匣圖示位置在其螢幕的 work area 內定位、大小改變就重新定位；
//!   開啟取得焦點、失焦收起，收起後的短暫窗口內忽略開啟要求。
//! - Linux：不提供面板（系統匣恆為原生選單樣式），toggle 回 Err 作守門、hide 無事。

#[cfg(target_os = "macos")]
pub use macos::{hide, toggle};
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use unsupported::{hide, toggle};
#[cfg(target_os = "windows")]
pub use windows::{hide, toggle};

/// 兩平台共用的面板視窗設定（D6）：標籤與 builder 的共同前段——無外框、不可調整大小、
/// 不進工作列、置頂、先隱藏；各平台再疊上自己的外觀設定。
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod common {
    use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Wry};

    pub const PANEL_LABEL: &str = "tray-panel";
    const PANEL_WIDTH: f64 = 320.0;
    const PANEL_HEIGHT: f64 = 420.0;

    pub fn builder(app: &AppHandle) -> WebviewWindowBuilder<'_, Wry, AppHandle> {
        WebviewWindowBuilder::new(app, PANEL_LABEL, WebviewUrl::App("panel.html".into()))
            .title("Speclink")
            .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
            .decorations(false)
            .resizable(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(false)
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use tauri::{AppHandle, Manager};
    use tauri_nspanel::{tauri_panel, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt};
    use tauri_plugin_positioner::{Position, WindowExt};
    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

    use super::common::{builder, PANEL_LABEL};

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

    /// 面板收合入口——交接主視窗前（tray-status-menu「面板交接主視窗後滑鼠互動完整」；
    /// design D2）與失焦自動收合共用。冪等且不會失敗：面板尚未建立或已收合皆無事，
    /// 不像 toggle 會把已收合的面板重新打開；hide 即 orderOut，對已隱藏的視窗本身就是無事。
    pub fn hide(app: &AppHandle) {
        if let Ok(panel) = app.get_webview_panel(PANEL_LABEL) {
            panel.hide();
        }
    }

    /// 建立面板視窗：共用前段（無邊框、不進工作列、置頂、先隱藏）加透明；轉 NSPanel 後套
    /// nonactivating style mask、浮動層級、vibrancy HudWindow 材質與失焦收合。
    fn create(app: &AppHandle) -> Result<(), String> {
        let window = builder(app)
            .transparent(true)
            .focused(false)
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
        handler.window_did_resign_key(move |_notification| hide(&handle));
        panel.set_event_handler(Some(handler.as_ref()));
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::sync::Mutex;
    use std::time::Instant;

    use speclink_desktop_core::tray_panel::{panel_position, should_ignore_open, PxRect};
    use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow, WindowEvent};

    use super::common::{builder, PANEL_LABEL};

    /// 系統匣圖示的 id——與 apps/desktop/src/tray.ts 建立圖示時的 id 一致。
    const TRAY_ID: &str = "speclink-tray";

    /// 面板狀態（D3／D4），隨面板建立以 app.manage 註冊。icon 是開啟當下的圖示位置，
    /// 供大小改變時重新定位：圖示在溢出區（^）時，溢出視窗收起後就查不到原本的位置。
    #[derive(Default)]
    struct PanelState {
        last_hidden: Mutex<Option<Instant>>,
        icon: Mutex<Option<PxRect>>,
    }

    /// 點擊系統匣圖示（左鍵）的開閉入口：不存在則 lazy 建立，存在則 toggle 顯隱。
    /// 開啟要求落在失焦收起後的抑制窗口內即忽略——那次失焦正是這次點擊圖示造成的。
    /// 只有建立與顯示失敗回 Err；收起與定位失敗不退回原生選單（面板仍可用）。
    pub fn toggle(app: &AppHandle) -> Result<(), String> {
        let window = match app.get_webview_window(PANEL_LABEL) {
            Some(window) => window,
            None => create(app)?,
        };
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return Ok(());
        }
        let state = app.state::<PanelState>();
        if should_ignore_open(*state.last_hidden.lock().unwrap(), Instant::now()) {
            return Ok(());
        }
        let icon = icon_rect(app);
        *state.icon.lock().unwrap() = icon;
        place(&window, icon);
        window
            .show()
            .map_err(|e| format!("tray panel show failed: {e}"))?;
        // 取得焦點才收得到失焦（點面板外收起的觸發源）；取不到時面板仍可經點圖示關閉，
        // 不為此退回原生選單。
        let _ = window.set_focus();
        Ok(())
    }

    /// 面板收合入口——交接主視窗前呼叫。冪等且不會失敗：面板尚未建立或已收合皆無事。
    pub fn hide(app: &AppHandle) {
        if let Some(window) = app.get_webview_window(PANEL_LABEL) {
            let _ = window.hide();
        }
    }

    /// 建立面板視窗（D1）：共用前段加系統陰影（Windows 11 由系統畫圓角）、不透明
    /// （builder 預設）；掛上大小改變重新定位與失焦收起，並隨主視窗關閉而關閉。
    fn create(app: &AppHandle) -> Result<WebviewWindow, String> {
        app.manage(PanelState::default());
        let window = builder(app)
            .shadow(true)
            .build()
            .map_err(|e| format!("tray panel window creation failed: {e}"))?;
        let handle = window.clone();
        window.on_window_event(move |event| match event {
            // 前端隨內容 setSize：重做定位，貼齊工作列的那一邊維持貼齊（D3）。
            WindowEvent::Resized(_) if handle.is_visible().unwrap_or(false) => {
                place(&handle, *handle.state::<PanelState>().icon.lock().unwrap());
            }
            WindowEvent::Focused(false) => {
                let _ = handle.hide();
                *handle.state::<PanelState>().last_hidden.lock().unwrap() = Some(Instant::now());
            }
            _ => {}
        });
        // Tauri 在所有視窗都關閉後才結束 app：常駐隱藏的面板會讓主視窗關閉後行程不結束，
        // 故主視窗關閉時一併關閉面板——與加入面板前「關主視窗即結束」一致。
        if let Some(main) = app.get_webview_window("main") {
            let panel = window.clone();
            main.on_window_event(move |event| {
                if let WindowEvent::Destroyed = event {
                    let _ = panel.destroy();
                }
            });
        }
        Ok(window)
    }

    /// 系統匣圖示的實體像素位置；圖示不存在或系統不提供時為 None。
    fn icon_rect(app: &AppHandle) -> Option<PxRect> {
        let rect = app.tray_by_id(TRAY_ID)?.rect().ok()??;
        // 系統匣回報的圖示位置即實體像素，縮放倍率不影響換算。
        let position = rect.position.to_physical::<i32>(1.0);
        let size = rect.size.to_physical::<u32>(1.0);
        Some(PxRect {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        })
    }

    /// 定位（D3）：取圖示所在螢幕的 work area，交 panel_position 算出位置——水平以圖示
    /// 為中心並限制在範圍內、垂直貼齊工作列那一側。圖示位置、螢幕或視窗大小取不到時
    /// 不移動（停在原位），不為定位失敗退回原生選單。
    fn place(window: &WebviewWindow, icon: Option<PxRect>) {
        let Some(icon) = icon else { return };
        let center_x = icon.x as f64 + icon.width as f64 / 2.0;
        let center_y = icon.y as f64 + icon.height as f64 / 2.0;
        let (Ok(Some(monitor)), Ok(size)) = (
            window.monitor_from_point(center_x, center_y),
            window.outer_size(),
        ) else {
            return;
        };
        let area = monitor.work_area();
        let (x, y) = panel_position(
            icon,
            size.width,
            size.height,
            PxRect {
                x: area.position.x,
                y: area.position.y,
                width: area.size.width,
                height: area.size.height,
            },
        );
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported {
    use tauri::AppHandle;

    pub fn toggle(_app: &AppHandle) -> Result<(), String> {
        Err("tray panel is unavailable on this platform".to_string())
    }

    pub fn hide(_app: &AppHandle) {}
}
