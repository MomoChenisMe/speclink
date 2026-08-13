// CLI 佈署純邏輯（desktop-app spec「安裝 CLI 指令到 PATH」，design D5）：狀態
// 判定與平台分流的佈署計畫。檔案系統與子程序歸 Tauri 殼（收集事實、執行計畫），
// 這裡只做決策——vitest 可測、不依賴 Tauri。
export type CliPlatform = "macos" | "windows" | "linux-appimage" | "linux-deb";

export type CliInstallStatus =
  | { kind: "not-installed" }
  | { kind: "installed"; version: string }
  | { kind: "version-mismatch"; version: string };

export type CliDeployPlan =
  | { action: "symlink"; linkPath: string; targetPath: string }
  | { action: "copy"; destPath: string; sourcePath: string }
  | { action: "none"; reason: "installer-managed" | "package-managed" };

/** 以偵測到的已佈署版本（null＝未偵測到）對 app 版本判定三態。 */
export function cliInstallStatus(
  deployedVersion: string | null,
  appVersion: string,
): CliInstallStatus {
  if (deployedVersion === null) return { kind: "not-installed" };
  if (deployedVersion === appVersion) return { kind: "installed", version: deployedVersion };
  return { kind: "version-mismatch", version: deployedVersion };
}

/** 解析 `speclink --version` 輸出的套件版號（如「speclink 0.1.0 (arm64, engine
 * v1.14.0)」取 0.1.0；括號內的引擎版號不參與佈署判定）；不合形回 null。 */
export function parseCliVersion(output: string): string | null {
  return output.match(/^speclink (\S+)/)?.[1] ?? null;
}

/** 平台分流的佈署計畫（design D5）：macOS symlink（app 路徑固定、更新後自動指向
 * 新版）；AppImage 複製（掛載點隨執行變動、symlink 不可行）；Windows 由 NSIS
 * 安裝器寫 PATH、deb 由包管理器佈署 /usr/bin——後兩者 app 內僅回報狀態。 */
export function cliDeployPlan(
  platform: CliPlatform,
  ctx: { home: string; bundledCliPath: string },
): CliDeployPlan {
  switch (platform) {
    case "macos":
      return {
        action: "symlink",
        linkPath: `${ctx.home}/.local/bin/speclink`,
        targetPath: ctx.bundledCliPath,
      };
    case "linux-appimage":
      return {
        action: "copy",
        destPath: `${ctx.home}/.local/bin/speclink`,
        sourcePath: ctx.bundledCliPath,
      };
    case "windows":
      return { action: "none", reason: "installer-managed" };
    case "linux-deb":
      return { action: "none", reason: "package-managed" };
  }
}

/** 啟動自動佈署判定（spec 修訂「macOS 啟動自動佈署與自我修復」＋「AppImage
 * 版本不符自我修復」，design D12）：macOS 未安裝或版本不符皆自動佈署（symlink
 * 冪等無副作用）；AppImage 僅版本不符時修復——未安裝仍是使用者的顯式動作
 * （複製佈署較重）；Windows／deb 歸安裝器與包管理器。 */
export function needsRedeploy(platform: CliPlatform, status: CliInstallStatus): boolean {
  if (platform === "macos") return status.kind !== "installed";
  return platform === "linux-appimage" && status.kind === "version-mismatch";
}

/** ~/.zprofile 的識別註解：追加行帶著它，冪等判定靠它。 */
export const ZPROFILE_MARKER = "# speclink cli";

export type ZprofilePlan = { action: "append"; line: string } | { action: "none" };

/** macOS PATH 追加決策（design D12）：已在 PATH 或 zprofile 已有識別行→不寫；
 * 否則輸出帶識別註解的一行（zprofile 讀不到視為空檔）。寫檔歸 adapter。 */
export function zprofilePlan(onPath: boolean, zprofile: string | null): ZprofilePlan {
  if (onPath) return { action: "none" };
  if ((zprofile ?? "").includes(ZPROFILE_MARKER)) return { action: "none" };
  return { action: "append", line: `export PATH="$HOME/.local/bin:$PATH" ${ZPROFILE_MARKER}` };
}

/** 佈署目錄是否已在 PATH 上；否則介面提示加入方式。 */
export function isDirOnPath(dir: string, pathEnv: string, delimiter: ":" | ";"): boolean {
  return pathEnv.split(delimiter).includes(dir);
}
