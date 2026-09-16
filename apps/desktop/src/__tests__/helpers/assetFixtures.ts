// 技能檔探測的共用 fixture（desktop-app spec「指令檔過期提示」）：store 面
// （store.test.ts）與 App 面（App.test.tsx）測試共用同一份過期探測回報。
import type { AssetProbeResult } from "../../adapter/workspace";

/** 過期探測：claude 工具的受管檔落後引擎，兩個檔案內容有異。 */
export const STALE_PROBE: AssetProbeResult = {
  status: "stale",
  currentVersion: "v1.3.0",
  tools: [{ tool: "claude", workspaceVersion: "v0.9.0", stale: true, newer: false, missing: false }],
  differingFiles: ["CLAUDE.md", ".claude/skills/speclink-apply/SKILL.md"],
};
