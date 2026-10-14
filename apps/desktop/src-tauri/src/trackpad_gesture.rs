//! macOS 觸控板手勢階段的原生監聽（desktop-swipe-back-finger-lift design D1、D3）：以 NSEvent
//! 本地監聽讀捲動事件的 phase／momentumPhase，手指按下與離開時各廣播一次 `trackpad-gesture`
//! 給前端 SwipeBack。判讀與事件格式在 speclink-desktop-core 的 trackpad_gesture，此處只做
//! AppKit 黏合。

use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSEvent, NSEventMask};
use speclink_desktop_core::trackpad_gesture::{
    classify_scroll_phase, TrackpadGestureEvent, TrackpadGesturePhase, TRACKPAD_GESTURE_EVENT,
};
use tauri::{AppHandle, Emitter, Manager};

/// 監聽物件：放進 app 管理的狀態持有到 app 結束，不提供移除。
struct ScrollMonitor(#[allow(dead_code)] Retained<AnyObject>);

// SAFETY: 監聽物件建立後只被持有，不讀取、不呼叫任何方法；唯一的操作是 app 結束時的
// release，而 objc 的 retain／release 可跨執行緒。
unsafe impl Send for ScrollMonitor {}
unsafe impl Sync for ScrollMonitor {}

/// 掛上捲動事件的本地監聽。handler 原樣回傳事件，WKWebView 與頁面捲動不受影響；
/// 安裝失敗只記錄，app 照常（失去觸控板滑動返回）。
pub fn install(app: &AppHandle) {
    let emitter = app.clone();
    let handler = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
        // SAFETY: AppKit 交給 handler 的事件在呼叫期間有效。
        let ns_event = unsafe { event.as_ref() };
        let phase = classify_scroll_phase(
            ns_event.phase().bits() as u64,
            ns_event.momentumPhase().bits() as u64,
        );
        if let Some(phase) = phase {
            // 系統設定「在頁面之間滑動」每次按下都重讀，執行中改設定也會生效。
            let swipe_enabled = phase == TrackpadGesturePhase::Began
                && NSEvent::isSwipeTrackingFromScrollEventsEnabled();
            let _ = emitter.emit(
                TRACKPAD_GESTURE_EVENT,
                TrackpadGestureEvent { phase, swipe_enabled },
            );
        }
        event.as_ptr()
    });
    // SAFETY: handler 回傳的是 AppKit 交進來的同一個有效事件指標。
    let monitor = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::ScrollWheel, &handler)
    };
    match monitor {
        Some(monitor) => {
            app.manage(ScrollMonitor(monitor));
        }
        None => eprintln!("speclink-desktop: trackpad gesture monitor unavailable"),
    }
}
