// CLI 佈署接線面（design D5）：Tauri 殼收集環境事實、執行佈署計畫；決策歸
// core/cliInstall。store 只依賴此介面，測試以假 adapter 注入。
import { invoke } from "@tauri-apps/api/core";

import type { CliDeployPlan, CliPlatform } from "../core/cliInstall";

/** Rust 殼一次收齊的環境事實（cli_install_probe）。 */
export interface CliInstallProbe {
  platform: CliPlatform;
  home: string | null;
  pathEnv: string;
  pathDelimiter: ":" | ";";
  bundledCliPath: string | null;
  appVersion: string;
  /** 已佈署 CLI 的 --version 原始輸出；null＝未偵測到。 */
  deployedVersionOutput: string | null;
}

export interface CliInstallAdapter {
  probe: () => Promise<CliInstallProbe>;
  deploy: (plan: Exclude<CliDeployPlan, { action: "none" }>) => Promise<void>;
}

export function tauriCliInstallAdapter(): CliInstallAdapter {
  return {
    probe: () => invoke<CliInstallProbe>("cli_install_probe"),
    deploy: (plan) => invoke("cli_deploy", { plan }),
  };
}
