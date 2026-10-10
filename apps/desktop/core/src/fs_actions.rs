//! 專案層檔案系統動作（design D5）：終端機與編輯器的候選清單，以及依序啟動候選的
//! `launch_first`。Tauri command 只做委派。

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// 執行平台（依編譯目標決定）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOs,
    Windows,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        }
    }
}

/// 一個待 spawn 的候選：程式名與參數。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spawn {
    pub program: String,
    pub args: Vec<String>,
}

fn spawn(program: &str, args: &[&str]) -> Spawn {
    Spawn {
        program: program.to_string(),
        args: args.iter().map(|a| a.to_string()).collect(),
    }
}

/// 在終端機開啟 `path` 的候選（依序嘗試）：macOS 系統 Terminal；Windows 先
/// Windows Terminal、再 cmd（路徑含該殼層語法字元時跳過該候選）；Linux 依序 x-terminal-emulator、gnome-terminal、
/// konsole、xfce4-terminal，各帶工作目錄參數。
pub fn terminal_candidates(platform: Platform, path: &str) -> Vec<Spawn> {
    match platform {
        Platform::MacOs => vec![spawn("open", &["-a", "Terminal", path])],
        // 路徑會經過該殼層的命令列解析：含其語法字元的候選跳過，免得資料夾名
        // 被拆成另一個指令（cmd：& | < > ^ % "；wt：; 是子指令分隔）。
        Platform::Windows => {
            let mut candidates = Vec::new();
            if !path.contains(';') {
                candidates.push(spawn("wt", &["-d", path]));
            }
            if !path.contains(['&', '|', '<', '>', '^', '%', '"']) {
                candidates.push(spawn("cmd", &["/c", "start", "cmd", "/K", "cd", "/d", path]));
            }
            candidates
        }
        Platform::Linux => {
            let wd = format!("--working-directory={path}");
            vec![
                spawn("x-terminal-emulator", &[&wd]),
                spawn("gnome-terminal", &[&wd]),
                spawn("konsole", &["--workdir", path]),
                spawn("xfce4-terminal", &[&wd]),
            ]
        }
    }
}

/// 以編輯器開啟 `path` 的候選（依序嘗試 PATH 上的指令）。Windows 找 PATH 只補
/// `.exe`，VS Code 與 Cursor 在 PATH 上的入口是 `.cmd`，要寫明副檔名。
pub fn editor_candidates(platform: Platform, path: &str) -> Vec<Spawn> {
    let programs = match platform {
        Platform::Windows => ["code.cmd", "cursor.cmd", "zed", "subl"],
        Platform::MacOs | Platform::Linux => ["code", "cursor", "zed", "subl"],
    };
    programs.into_iter().map(|program| spawn(program, &[path])).collect()
}

/// 「以預設 App 開啟」只收 markdown 文件：詳情頁的文件動作只會傳 openspec 下的
/// `.md`，其餘（.app、.command、目錄）交給預設 App 等於執行任意程式，一律拒絕。
/// 傳入的名稱與解析符號連結後的實際目標都要是 `.md`——名為 `proposal.md`、實際指向
/// 可執行檔的連結不得混過；殼層依傳入名稱的副檔名選程式，名為 `x.bat`、指向 `.md` 的
/// 連結同樣拒絕。解析結果只用來檢查：Windows 的解析結果帶 `\\?\` 前綴，殼層與不少預設
/// App 不認，開啟時仍傳原路徑。拒絕時錯誤字串為 i18n 鍵 `fs.notDocument`，缺檔為 OS 原文。
pub fn check_default_open(path: &str) -> Result<(), String> {
    let is_markdown = |p: &Path| p.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
    let target = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    if is_markdown(Path::new(path)) && target.is_file() && is_markdown(&target) {
        Ok(())
    } else {
        Err("fs.notDocument".to_string())
    }
}

/// 啟動後在這段時間內以非零碼退出的候選視同失敗（如終端機不認得工作目錄參數）。
const EARLY_EXIT_WINDOW: Duration = Duration::from_millis(500);

/// 依序啟動候選，第一個沒有立即失敗的即回 Ok。`dir` 不存在時回 `fs.dirMissing`，
/// 全部失敗回 `none_found`（皆為 i18n 鍵）。不讀子程序輸出；仍在執行的子程序另起
/// 執行緒回收，免留殭屍程序。
pub fn launch_first(dir: &str, candidates: Vec<Spawn>, path_env: &str, none_found: &str) -> Result<(), String> {
    if !Path::new(dir).is_dir() {
        return Err("fs.dirMissing".to_string());
    }
    if candidates.iter().any(|candidate| launch(candidate, path_env)) {
        Ok(())
    } else {
        Err(none_found.to_string())
    }
}

fn launch(candidate: &Spawn, path_env: &str) -> bool {
    let mut command = Command::new(&candidate.program);
    command
        .args(&candidate.args)
        .env("PATH", path_env)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // GUI app 啟動主控台程式（cmd、.cmd 包裝）會先閃一個主控台視窗。
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let Ok(mut child) = command.spawn() else {
        return false;
    };
    let deadline = Instant::now() + EARLY_EXIT_WINDOW;
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(_) => break,
        }
    }
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(spawns: &[Spawn]) -> Vec<Vec<&str>> {
        spawns
            .iter()
            .map(|s| {
                std::iter::once(s.program.as_str())
                    .chain(s.args.iter().map(String::as_str))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn default_app_open_only_accepts_existing_markdown_files() {
        let dir = std::env::temp_dir().join(format!("speclink-dtcore-default-open-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("folder.md")).unwrap();
        std::fs::write(dir.join("proposal.md"), "# p").unwrap();
        std::fs::write(dir.join("DESIGN.MD"), "# d").unwrap();
        std::fs::write(dir.join("run.command"), "echo hi").unwrap();
        std::fs::write(dir.join("x.md.sh"), "echo hi").unwrap();
        let at = |name: &str| dir.join(name).to_string_lossy().into_owned();

        for ok in ["proposal.md", "DESIGN.MD"] {
            assert_eq!(check_default_open(&at(ok)), Ok(()), "{ok}");
        }
        for bad in ["run.command", "x.md.sh", "folder.md"] {
            assert_eq!(check_default_open(&at(bad)), Err("fs.notDocument".to_string()), "{bad}");
        }
        assert!(check_default_open(&at("missing.md")).is_err(), "缺檔回 OS 錯誤");
        assert!(check_default_open("").is_err());
        #[cfg(unix)]
        {
            // 名為 .md 的符號連結指向可執行檔：解析連結後再判，不得混過副檔名檢查。
            std::os::unix::fs::symlink(dir.join("run.command"), dir.join("tasks.md")).unwrap();
            assert_eq!(check_default_open(&at("tasks.md")), Err("fs.notDocument".to_string()));
            // 反過來：名稱不是 .md 的連結指向 .md——殼層依傳入名稱的副檔名選程式，同樣拒絕。
            std::os::unix::fs::symlink(dir.join("proposal.md"), dir.join("open.command")).unwrap();
            assert_eq!(check_default_open(&at("open.command")), Err("fs.notDocument".to_string()));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn macos_terminal_is_system_terminal_app() {
        let p = "/Users/a/My Project";
        assert_eq!(
            flat(&terminal_candidates(Platform::MacOs, p)),
            vec![vec!["open", "-a", "Terminal", p]]
        );
    }

    #[test]
    fn windows_terminal_falls_back_to_cmd() {
        let p = r"C:\work\speclink";
        assert_eq!(
            flat(&terminal_candidates(Platform::Windows, p)),
            vec![
                vec!["wt", "-d", p],
                vec!["cmd", "/c", "start", "cmd", "/K", "cd", "/d", p],
            ]
        );
    }

    #[test]
    fn windows_skips_candidates_whose_shell_would_split_the_path() {
        // cmd 把 & | < > ^ % " 當指令語法、wt 把 ; 當子指令分隔：含這些字元的
        // 路徑交給該殼層會被拆成另一個指令，那個候選直接跳過。
        assert_eq!(
            flat(&terminal_candidates(Platform::Windows, r"C:\work\R&D")),
            vec![vec!["wt", "-d", r"C:\work\R&D"]]
        );
        assert_eq!(
            flat(&terminal_candidates(Platform::Windows, r"C:\work\a;b")),
            vec![vec!["cmd", "/c", "start", "cmd", "/K", "cd", "/d", r"C:\work\a;b"]]
        );
        assert!(terminal_candidates(Platform::Windows, r"C:\x;y&z").is_empty());
    }

    #[test]
    fn linux_terminals_in_order_each_with_working_directory() {
        let p = "/home/a/speclink";
        assert_eq!(
            flat(&terminal_candidates(Platform::Linux, p)),
            vec![
                vec!["x-terminal-emulator", "--working-directory=/home/a/speclink"],
                vec!["gnome-terminal", "--working-directory=/home/a/speclink"],
                vec!["konsole", "--workdir", p],
                vec!["xfce4-terminal", "--working-directory=/home/a/speclink"],
            ]
        );
    }

    #[test]
    fn editors_in_order_each_with_path() {
        let p = "/Users/a/speclink";
        for platform in [Platform::MacOs, Platform::Linux] {
            assert_eq!(
                flat(&editor_candidates(platform, p)),
                vec![vec!["code", p], vec!["cursor", p], vec!["zed", p], vec!["subl", p]]
            );
        }
    }

    #[test]
    fn windows_editors_use_cmd_shims_for_code_and_cursor() {
        // Windows 找 PATH 只補 .exe；VS Code 與 Cursor 在 PATH 上的入口是 .cmd。
        let p = r"C:\work\speclink";
        assert_eq!(
            flat(&editor_candidates(Platform::Windows, p)),
            vec![vec!["code.cmd", p], vec!["cursor.cmd", p], vec!["zed", p], vec!["subl", p]]
        );
    }

    #[cfg(unix)]
    mod launch {
        use super::super::*;
        use std::path::PathBuf;

        fn temp_dir(tag: &str) -> PathBuf {
            let dir = std::env::temp_dir().join(format!("speclink-fsact-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            dir
        }

        fn sh(script: &str) -> Spawn {
            spawn("sh", &["-c", script])
        }

        fn path_env() -> String {
            std::env::var("PATH").unwrap_or_default()
        }

        #[test]
        fn missing_dir_is_reported_before_any_spawn() {
            let dir = temp_dir("missing");
            let marker = dir.join("ran");
            let gone = dir.join("gone");
            let result = launch_first(
                gone.to_str().unwrap(),
                vec![sh(&format!("touch '{}'", marker.display()))],
                &path_env(),
                "fs.noTerminal",
            );
            assert_eq!(result, Err("fs.dirMissing".to_string()));
            assert!(!marker.exists());
        }

        #[test]
        fn candidate_that_exits_with_failure_falls_through_to_the_next() {
            let dir = temp_dir("fallthrough");
            let second = dir.join("second");
            let third = dir.join("third");
            let fourth = dir.join("fourth");
            let result = launch_first(
                dir.to_str().unwrap(),
                vec![
                    spawn("speclink-no-such-program", &[]),
                    sh(&format!("touch '{}'; exit 3", second.display())),
                    sh(&format!("touch '{}'", third.display())),
                    sh(&format!("touch '{}'", fourth.display())),
                ],
                &path_env(),
                "fs.noTerminal",
            );
            assert_eq!(result, Ok(()));
            assert!(second.exists());
            assert!(third.exists());
            assert!(!fourth.exists(), "第一個成功的候選之後不再嘗試");
        }

        #[test]
        fn all_candidates_failing_returns_the_none_found_key() {
            let dir = temp_dir("allfail");
            let result = launch_first(
                dir.to_str().unwrap(),
                vec![sh("exit 1"), spawn("speclink-no-such-program", &[])],
                &path_env(),
                "fs.noEditor",
            );
            assert_eq!(result, Err("fs.noEditor".to_string()));
        }

        #[test]
        fn candidate_still_running_after_the_window_counts_as_started() {
            let dir = temp_dir("running");
            let started = std::time::Instant::now();
            let result = launch_first(dir.to_str().unwrap(), vec![sh("sleep 5")], &path_env(), "fs.noTerminal");
            assert_eq!(result, Ok(()));
            assert!(started.elapsed() < std::time::Duration::from_secs(3), "不等子程序結束");
        }
    }
}
