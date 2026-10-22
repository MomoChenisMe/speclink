/** Every built-in picker shares these names and generated skill roots. */
const TOOL_ROOTS = {
  claude: ".claude",
  codex: ".agents",
  copilot: ".github",
  antigravity: ".agent",
} as const;
export type BuiltinTool = keyof typeof TOOL_ROOTS;
export const BUILTIN_TOOLS = Object.keys(TOOL_ROOTS) as BuiltinTool[];
export const builtinToolLabel = (tool: BuiltinTool): string => `${tool} [${TOOL_ROOTS[tool]}]`;
