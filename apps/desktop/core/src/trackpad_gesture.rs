//! 觸控板手勢階段的判讀與送給前端的事件格式（desktop-swipe-back-finger-lift design D2）。
//! macOS 的 Tauri 殼以 NSEvent 本地監聽讀捲動事件的 phase／momentumPhase 位元，交給
//! [`classify_scroll_phase`] 判讀後廣播 [`TrackpadGestureEvent`]；殼端只做 AppKit 黏合。

use serde::Serialize;

/// 殼廣播、前端 SwipeBack 訂閱的 Tauri 事件名。
pub const TRACKPAD_GESTURE_EVENT: &str = "trackpad-gesture";

// NSEventPhase 的位元值（AppKit 定義；殼端以 `.bits()` 原樣傳入）。
const PHASE_BEGAN: u64 = 0x1;
const PHASE_ENDED: u64 = 0x8;
const PHASE_CANCELLED: u64 = 0x10;
const PHASE_MAY_BEGIN: u64 = 0x20;

/// 手指按下（began）或離開觸控板（ended）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackpadGesturePhase {
    Began,
    Ended,
}

/// `trackpad-gesture` 事件的 payload：`{"phase":"began"|"ended","swipeEnabled":bool}`。
/// swipe_enabled 為系統設定「在頁面之間滑動」是否允許兩指滑動，只在 began 有意義。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackpadGestureEvent {
    pub phase: TrackpadGesturePhase,
    pub swipe_enabled: bool,
}

/// 依捲動事件的 phase／momentumPhase 位元判讀手指按下或離開。慣性事件（momentum 非 0）
/// 不是手指動作；MayBegin／Began 為按下、Ended／Cancelled 為離開；手指移動中（Changed）
/// 與一般滑鼠滾輪（兩者皆 0）回 None。
pub fn classify_scroll_phase(phase_bits: u64, momentum_bits: u64) -> Option<TrackpadGesturePhase> {
    if momentum_bits != 0 {
        None
    } else if phase_bits & (PHASE_MAY_BEGIN | PHASE_BEGAN) != 0 {
        Some(TrackpadGesturePhase::Began)
    } else if phase_bits & (PHASE_ENDED | PHASE_CANCELLED) != 0 {
        Some(TrackpadGesturePhase::Ended)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NSEventPhase 位元值（與 AppKit 相同）。
    const NONE: u64 = 0;
    const BEGAN: u64 = 0x1;
    const CHANGED: u64 = 0x4;
    const ENDED: u64 = 0x8;
    const CANCELLED: u64 = 0x10;
    const MAY_BEGIN: u64 = 0x20;

    #[test]
    fn finger_down_phases_classify_as_began() {
        assert_eq!(classify_scroll_phase(MAY_BEGIN, NONE), Some(TrackpadGesturePhase::Began));
        assert_eq!(classify_scroll_phase(BEGAN, NONE), Some(TrackpadGesturePhase::Began));
    }

    #[test]
    fn finger_up_phases_classify_as_ended() {
        assert_eq!(classify_scroll_phase(ENDED, NONE), Some(TrackpadGesturePhase::Ended));
        assert_eq!(classify_scroll_phase(CANCELLED, NONE), Some(TrackpadGesturePhase::Ended));
    }

    #[test]
    fn moves_momentum_and_plain_wheel_classify_as_none() {
        assert_eq!(classify_scroll_phase(CHANGED, NONE), None, "手指移動中");
        assert_eq!(classify_scroll_phase(NONE, CHANGED), None, "慣性捲動");
        assert_eq!(classify_scroll_phase(NONE, NONE), None, "一般滑鼠滾輪");
    }

    #[test]
    fn unknown_and_stationary_bits_classify_as_none() {
        // 不認得的位元一律不送事件：寧可失去一次滑動，也不誤開始或誤結算。
        const STATIONARY: u64 = 0x2;
        assert_eq!(classify_scroll_phase(STATIONARY, NONE), None);
        assert_eq!(classify_scroll_phase(0x40, NONE), None);
    }

    #[test]
    fn momentum_wins_over_finger_bits() {
        // 慣性事件不得開始或結束一次滑動，即使 phase 位元同時帶了按下／離開。
        assert_eq!(classify_scroll_phase(BEGAN, BEGAN), None);
        assert_eq!(classify_scroll_phase(ENDED, ENDED), None);
    }

    #[test]
    fn event_payload_serializes_to_the_frontend_contract() {
        let event = TrackpadGestureEvent {
            phase: TrackpadGesturePhase::Began,
            swipe_enabled: true,
        };
        assert_eq!(
            serde_json::to_string(&event).unwrap(),
            r#"{"phase":"began","swipeEnabled":true}"#
        );
        assert_eq!(TRACKPAD_GESTURE_EVENT, "trackpad-gesture");
    }
}
