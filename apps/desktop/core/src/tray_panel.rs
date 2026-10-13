//! 系統匣面板的 Windows 定位與開閉競態計算（tray-status-menu「面板樣式」平台差異——
//! Windows；design D3／D4）。Tauri 殼只做視窗操作，可單元測試的兩段純計算放這裡。

use std::time::{Duration, Instant};

/// 失焦收起後的開啟抑制門檻（D4）：在 Windows 點系統匣圖示會先讓面板失焦收起、
/// 點擊事件才送到——門檻內的開啟要求就是這次點擊，忽略它面板才關得掉。
const REOPEN_SUPPRESS: Duration = Duration::from_millis(300);

/// 實體像素矩形（與 Tauri 的 PhysicalPosition<i32>／PhysicalSize<u32> 同型）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PxRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// 面板左上角位置（D3）：水平以圖示為中心並限制在 work area 內；垂直貼齊工作列那一側——
/// 圖示在 work area 垂直中線以下（工作列在底部）時面板底部貼齊 work area 下緣，否則
/// （工作列在頂端）面板頂部貼齊 work area 上緣。area 為圖示所在螢幕的 work area；
/// 只看這個螢幕，副螢幕的負座標不會讓面板翻到別的螢幕。
pub fn panel_position(
    icon: PxRect,
    panel_width: u32,
    panel_height: u32,
    area: PxRect,
) -> (i32, i32) {
    let icon_center_x = icon.x + icon.width as i32 / 2;
    let x = clamp_panel_x(
        icon_center_x - panel_width as i32 / 2,
        panel_width,
        area.x,
        area.width,
    );
    let icon_center_y = icon.y + icon.height as i32 / 2;
    let y = if icon_center_y >= area.y + area.height as i32 / 2 {
        area.y + area.height as i32 - panel_height as i32
    } else {
        area.y
    };
    (x, y)
}

/// 把面板左緣 x 限制在 work area 內：超出右緣往左貼齊、超出左緣往右貼齊；
/// 面板比 work area 還寬時貼齊左緣。
fn clamp_panel_x(candidate_x: i32, panel_width: u32, area_left: i32, area_width: u32) -> i32 {
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

    // --- panel_position：圖示所在螢幕 work area 內的面板位置（D3） ---

    /// 底部工作列上、中心在 (center_x, 1056) 的 32×48 圖示；work area 為 0–1920 × 0–1032。
    fn bottom_taskbar_icon(center_x: i32) -> PxRect {
        PxRect {
            x: center_x - 16,
            y: 1032,
            width: 32,
            height: 48,
        }
    }

    const BOTTOM_TASKBAR_AREA: PxRect = PxRect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1032,
    };

    #[test]
    fn position_centers_on_the_icon_and_hugs_the_bottom_taskbar() {
        // Example「水平位置限制」第 1 列：圖示中心 1000 → 左緣 840；
        // Example「垂直位置」第 1 列：工作列在底部 → 上緣 1032 - 420 = 612。
        assert_eq!(
            panel_position(bottom_taskbar_icon(1000), 320, 420, BOTTOM_TASKBAR_AREA),
            (840, 612)
        );
    }

    #[test]
    fn position_hugs_the_right_edge_near_the_right_of_the_work_area() {
        // Example「水平位置限制」第 2 列：圖示中心 1860 → 左緣 1600。
        assert_eq!(
            panel_position(bottom_taskbar_icon(1860), 320, 420, BOTTOM_TASKBAR_AREA).0,
            1600
        );
    }

    #[test]
    fn position_hugs_the_left_edge_near_the_left_of_the_work_area() {
        // Example「水平位置限制」第 3 列：圖示中心 100 → 左緣 0。
        assert_eq!(
            panel_position(bottom_taskbar_icon(100), 320, 420, BOTTOM_TASKBAR_AREA).0,
            0
        );
    }

    #[test]
    fn position_hugs_the_top_taskbar_below_the_icon() {
        // Example「垂直位置」第 2 列：工作列在頂端（圖示中心 y 24、work area 48–1080）→ 上緣 48。
        let icon = PxRect {
            x: 984,
            y: 0,
            width: 32,
            height: 48,
        };
        let area = PxRect {
            x: 0,
            y: 48,
            width: 1920,
            height: 1032,
        };
        assert_eq!(panel_position(icon, 320, 420, area), (840, 48));
    }

    #[test]
    fn position_stays_on_a_monitor_stacked_above_the_primary() {
        // Example「垂直位置」第 3 列：副螢幕在主螢幕上方、工作列在其底部（圖示中心 y -24、
        // work area -1080–-48）→ 上緣 -48 - 420 = -468，留在副螢幕、不翻到主螢幕。
        let icon = PxRect {
            x: 984,
            y: -48,
            width: 32,
            height: 48,
        };
        let area = PxRect {
            x: 0,
            y: -1080,
            width: 1920,
            height: 1032,
        };
        assert_eq!(panel_position(icon, 320, 420, area), (840, -468));
    }

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
