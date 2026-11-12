// 技能檔探測的共用 fixture（desktop-app spec「指令檔過期提示」）：store 面
// （store.test.ts）與 App 面（App.test.tsx）測試共用同一份過期探測回報。
import type { AssetProbeResult } from "../../adapter/workspace";

/** 過期探測：claude 工具的受管檔落後引擎，兩個檔案內容有異。 */
export const STALE_PROBE: AssetProbeResult = {
  status: "stale",
  currentVersion: "v1.3.0",
  tools: [{ tool: "claude", workspaceVersion: "v0.9.0", stale: true, newer: false, missing: false }],
  differingFiles: ["CLAUDE.md", ".claude/skills/speclink-apply/SKILL.md"],
  migration: { state: "none" },
};

/** remote 專案的過期探測：基準是 Server 提供的技能（v1.58.0），一個技能檔和它不同；
 * 這個 app 內建的技能版號是 v1.57.0。 */
export const REMOTE_STALE_PROBE: AssetProbeResult = {
  status: "stale",
  currentVersion: "v1.57.0",
  tools: [{ tool: "claude", workspaceVersion: "v1.56.0", stale: true, newer: false, missing: false }],
  differingFiles: [".claude/skills/speclink-apply/SKILL.md"],
  migration: { state: "none" },
  serverVersion: "v1.58.0",
  appOlderThanServer: false,
};

/** 有 checkout 的 remote 分頁要探測的對象（對應 remoteFixtures 的 REMOTE_KEY）。 */
export const REMOTE_CHECKOUT = { connectionId: "c1", project: "demo", repo: "backend", checkout: "/work/demo" };
