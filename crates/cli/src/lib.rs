//! SpecLink CLI library — clap surface、JSON envelope、exit code 分類、命令實作模組。
//!
//! `main.rs` 是 binary entrypoint；所有可單元測試的邏輯都在這個 lib 模組底下。

pub mod cli;
pub mod commands;
pub mod exit_code;
pub mod output;
pub mod tracing_layer;
