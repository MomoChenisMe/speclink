//! 系統匣面板的 Windows 定位與開閉競態計算（tray-status-menu「面板樣式」平台差異——
//! Windows；design D3／D4）。Tauri 殼只做視窗操作，可單元測試的兩段純計算放這裡。

use std::time::{Duration, Instant};

/// 失焦收起後的開啟抑制門檻（D4）：在 Windows 點系統匣圖示會先讓面板失焦收起、
/// 點擊事件才送到——門檻內的開啟要求就是這次點擊，忽略它面板才關得掉。
pub const REOPEN_SUPPRESS: Duration = Duration::from_millis(300);

/// 把面板左緣 x 限制在 work area 內（D3）：超出右緣往左貼齊、超出左緣往右貼齊；
/// 面板比 work area 還寬時貼齊左緣。座標與寬度皆為實體像素（與 Tauri 的
/// PhysicalPosition<i32>／PhysicalSize<u32> 同型）。
pub fn clamp_panel_x(candidate_x: i32, panel_width: u32, area_left: i32, area_width: u32) -> i32 {
    let max_x = area_left + area_width as i32 - panel_width as i32;
    candidate_x.min(max_x).max(area_left)
}

/// 開啟要求是否落在失焦收起後的抑制窗口內（D4）。沒有收起紀錄即不忽略。
pub fn should_ignore_open(last_hidden: Option<Instant>, now: Instant) -> bool {
    last_hidden.is_some_and(|hidden| now.saturating_duration_since(hidden) < REOPEN_SUPPRESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    // --- clamp_panel_x：水平限制在圖示所在螢幕的 work area 內（D3） ---

    #[test]
    fn clamp_keeps_a_position_already_inside_the_work_area() {
        assert_eq!(clamp_panel_x(840, 320, 0, 1920), 840);
    }

    #[test]
    fn clamp_moves_left_to_hug_the_right_edge() {
        assert_eq!(clamp_panel_x(1700, 320, 0, 1920), 1600);
    }

    #[test]
    fn clamp_moves_right_to_hug_the_left_edge() {
        assert_eq!(clamp_panel_x(-60, 320, 0, 1920), 0);
    }

    #[test]
    fn clamp_hugs_the_left_edge_when_the_panel_is_wider_than_the_work_area() {
        assert_eq!(clamp_panel_x(840, 320, 0, 300), 0);
    }

    // --- should_ignore_open：失焦收起後的重開抑制（D4） ---

    #[test]
    fn open_is_not_ignored_without_a_hide_record() {
        assert!(!should_ignore_open(None, Instant::now()));
    }

    #[test]
    fn open_is_ignored_within_the_suppress_window() {
        let hidden = Instant::now();
        for ms in [120, 299] {
            assert!(
                should_ignore_open(Some(hidden), hidden + Duration::from_millis(ms)),
                "{ms}ms after hide should be ignored"
            );
        }
    }

    #[test]
    fn open_is_not_ignored_at_the_threshold() {
        let hidden = Instant::now();
        assert_eq!(REOPEN_SUPPRESS, Duration::from_millis(300));
        assert!(!should_ignore_open(
            Some(hidden),
            hidden + Duration::from_millis(300)
        ));
    }
}
