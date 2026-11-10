/** 內建 AI 工具的勾選清單：初始化確認、啟用確認、checkout 綁定與設定頁 AI 工具卡共用。
 * 名稱與引擎 `Tool::parse` 認得的內建名一致，標籤直接顯示原始工具名。 */
export const BUILTIN_TOOLS = ["claude", "codex", "copilot", "pi", "cursor"] as const;

/** 內建工具的產品顯示名（子代理模型頁的工具欄）；產品名不翻譯。 */
export const BUILTIN_TOOL_NAMES: Record<(typeof BUILTIN_TOOLS)[number], string> = {
  claude: "Claude Code",
  codex: "Codex",
  copilot: "Copilot",
  pi: "pi",
  cursor: "Cursor",
};
