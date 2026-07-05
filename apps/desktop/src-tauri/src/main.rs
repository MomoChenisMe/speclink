// Windows release build 不開 console 視窗。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    speclink_desktop_lib::run()
}
