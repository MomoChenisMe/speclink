// 子代理模型的草稿與儲存（desktop-subagent-model-page design「子代理模型的草稿與儲存」）：
// 頁面與 hook 共用的純函式。草稿只涵蓋內建工具；存回以現值為底，其他工具名稱的格子與清單原樣帶回。
import type { SettingsChoiceGroup, SettingsChoiceOption } from "@speclink/ui";

import type { SubagentModelConfig } from "../../adapter/workspace";
import { BUILTIN_TOOLS } from "../../builtinTools";
import { entriesToText, keepUnedited } from "./lines";

export const SUBAGENT_STATIONS = ["review", "verify"] as const;
export type SubagentStation = (typeof SUBAGENT_STATIONS)[number];

/** 下拉的哨兵值（Radix Select 的值不得為空字串）：沿用主對話、自行輸入…。 */
export const SUBAGENT_INHERIT = "__inherit__";
export const SUBAGENT_CUSTOM = "__custom__";

/** 一格的草稿：沿用主對話、從清單選、自行輸入（輸入框帶 value）。 */
export interface SubagentCell {
  mode: "inherit" | "pick" | "custom";
  value: string;
}

/** 編輯草稿：兩站各內建工具一格，各內建工具的自訂清單一份「一行一個」文字。 */
export interface SubagentDraft {
  cells: Record<SubagentStation, Record<string, SubagentCell>>;
  lists: Record<string, string>;
}

export type ModelSource = "builtin" | "choices" | "custom";

export const EMPTY_SUBAGENT_MODEL: SubagentModelConfig = { review: {}, verify: {}, choices: {} };

/** 值的來源：在內建清單＝內建；否則在自訂清單＝自訂清單；否則＝自行輸入。 */
export function modelSource(value: string, builtin: readonly string[], choices: readonly string[]): ModelSource {
  if (builtin.includes(value)) return "builtin";
  if (choices.includes(value)) return "choices";
  return "custom";
}

export function toDraft(config: SubagentModelConfig, builtin: Record<string, string[]>): SubagentDraft {
  const cell = (station: SubagentStation, tool: string): SubagentCell => {
    const value = config[station][tool];
    if (value === undefined) return { mode: "inherit", value: "" };
    const source = modelSource(value, builtin[tool] ?? [], config.choices[tool] ?? []);
    return { mode: source === "custom" ? "custom" : "pick", value };
  };
  const perTool = <T>(make: (tool: string) => T) =>
    Object.fromEntries(BUILTIN_TOOLS.map((tool) => [tool, make(tool)]));
  return {
    cells: { review: perTool((tool) => cell("review", tool)), verify: perTool((tool) => cell("verify", tool)) },
    lists: perTool((tool) => entriesToText(config.choices[tool] ?? [])),
  };
}

/** 存回：只覆寫內建工具的格子與清單——沿用主對話與留空的自行輸入移除該格，自行輸入去頭尾
 * 空白；清單一行一條、沒改過的沿用現值，清空移除鍵。 */
export function fromDraft(draft: SubagentDraft, current: SubagentModelConfig): SubagentModelConfig {
  const next: SubagentModelConfig = {
    review: { ...current.review },
    verify: { ...current.verify },
    choices: { ...current.choices },
  };
  for (const tool of BUILTIN_TOOLS) {
    for (const station of SUBAGENT_STATIONS) {
      const cell = draft.cells[station][tool];
      const value = cell.mode === "inherit" ? "" : cell.value.trim();
      if (value === "") delete next[station][tool];
      else next[station][tool] = value;
    }
    const names = keepUnedited(draft.lists[tool], current.choices[tool] ?? []);
    if (names.length === 0) delete next.choices[tool];
    else next.choices[tool] = names;
  }
  return next;
}

const sortedEntries = <T>(map: Record<string, T>) =>
  Object.entries(map).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
const canonical = (config: SubagentModelConfig) =>
  JSON.stringify([sortedEntries(config.review), sortedEntries(config.verify), sortedEntries(config.choices)]);

/** 內容相同（不看鍵序）：存回結果與現值相同時不寫檔。 */
export const sameSubagentModel = (a: SubagentModelConfig, b: SubagentModelConfig) => canonical(a) === canonical(b);

/** 下拉選項：沿用主對話、內建組、自訂清單組、自行輸入…；空的組不出現。同名只留在內建組，
 * 重複名稱只列一次，撞到哨兵的名稱不列（Radix Select 的值不得重複）。 */
export function choiceOptions(
  builtin: readonly string[],
  custom: readonly string[],
  labels: { inherit: string; builtin: string; choices: string; custom: string },
): (SettingsChoiceOption | SettingsChoiceGroup)[] {
  const sentinels = [SUBAGENT_INHERIT, SUBAGENT_CUSTOM];
  const group = (label: string, names: string[]): SettingsChoiceGroup[] =>
    names.length === 0 ? [] : [{ label, options: names.map((name) => ({ value: name, label: name })) }];
  const builtinNames = builtin.filter((name) => !sentinels.includes(name));
  const customNames = Array.from(new Set(custom)).filter(
    (name) => !sentinels.includes(name) && !builtinNames.includes(name),
  );
  return [
    { value: SUBAGENT_INHERIT, label: labels.inherit },
    ...group(labels.builtin, builtinNames),
    ...group(labels.choices, customNames),
    { value: SUBAGENT_CUSTOM, label: labels.custom },
  ];
}

/** 一格在下拉上的值：pick 的名稱已不在選項裡（從清單刪掉了）時改判自行輸入。 */
export function cellChoice(cell: SubagentCell, options: (SettingsChoiceOption | SettingsChoiceGroup)[]): string {
  if (cell.mode === "inherit") return SUBAGENT_INHERIT;
  const listed = options.some((entry) => "options" in entry && entry.options.some((o) => o.value === cell.value));
  return cell.mode === "pick" && listed ? cell.value : SUBAGENT_CUSTOM;
}
