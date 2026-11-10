// 子代理模型的草稿與儲存（desktop-subagent-model-page design「子代理模型的草稿與儲存」）：
// 由現值建草稿、來源判定、存回合併、相同判定、下拉選項組成——頁面與 hook 共用的純函式。
import { describe, expect, it } from "vitest";

import type { SubagentModelConfig } from "../adapter/workspace";
import {
  SUBAGENT_CUSTOM,
  SUBAGENT_INHERIT,
  cellChoice,
  choiceOptions,
  fromDraft,
  modelSource,
  sameSubagentModel,
  toDraft,
} from "../views/project-settings/subagentModel";

const BUILTIN = { claude: ["sonnet", "opus", "haiku", "fable"] };
const EMPTY: SubagentModelConfig = { review: {}, verify: {}, choices: {} };
const LABELS = { inherit: "沿用主對話", builtin: "內建", choices: "自訂清單", custom: "自行輸入…" };

const CURRENT: SubagentModelConfig = {
  review: { claude: "opus", codex: "gpt-6.1-sol", copilot: "gpt-5.6" },
  verify: { claude: "sonnet" },
  choices: { codex: ["gpt-6.1-sol", "gpt-6-astra"] },
};

describe("toDraft", () => {
  it("沒值 inherit、在內建或自訂清單 pick、其他 custom；清單一行一個", () => {
    const draft = toDraft(CURRENT, BUILTIN);
    expect(draft.cells.review.claude).toEqual({ mode: "pick", value: "opus" });
    expect(draft.cells.review.codex).toEqual({ mode: "pick", value: "gpt-6.1-sol" });
    expect(draft.cells.review.copilot).toEqual({ mode: "custom", value: "gpt-5.6" });
    expect(draft.cells.review.pi).toEqual({ mode: "inherit" });
    expect(draft.cells.verify.claude).toEqual({ mode: "pick", value: "sonnet" });
    expect(draft.cells.verify.codex).toEqual({ mode: "inherit" });
    expect(draft.lists).toEqual({ claude: "", codex: "gpt-6.1-sol\ngpt-6-astra", copilot: "", pi: "", cursor: "" });
  });
});

describe("modelSource", () => {
  it("優先序：內建 > 自訂清單 > 自行輸入", () => {
    expect(modelSource("opus", BUILTIN.claude, ["opus"])).toBe("builtin");
    expect(modelSource("gpt-6.1-sol", [], ["gpt-6.1-sol"])).toBe("choices");
    expect(modelSource("gpt-5.6", [], ["gpt-6.1-sol"])).toBe("custom");
  });
});

describe("cellChoice", () => {
  it("下拉的值：inherit 與 custom 各為哨兵、pick 為名稱", () => {
    const options = choiceOptions(BUILTIN.claude, [], LABELS);
    expect(cellChoice({ mode: "inherit" }, options)).toBe(SUBAGENT_INHERIT);
    expect(cellChoice({ mode: "pick", value: "opus" }, options)).toBe("opus");
    expect(cellChoice({ mode: "custom", value: "opus" }, options)).toBe(SUBAGENT_CUSTOM);
  });

  it("編輯中從清單刪掉已選用的名稱：改判自行輸入", () => {
    const options = choiceOptions([], ["gpt-6-astra"], LABELS);
    expect(cellChoice({ mode: "pick", value: "gpt-6.1-sol" }, options)).toBe(SUBAGENT_CUSTOM);
  });
});

describe("fromDraft", () => {
  it("沒改任何東西：存回結果與現值相同", () => {
    const saved = fromDraft(toDraft(CURRENT, BUILTIN), CURRENT);
    expect(saved).toEqual(CURRENT);
    expect(sameSubagentModel(saved, CURRENT)).toBe(true);
  });

  it("inherit 與留空的 custom 移除該格；custom 去頭尾空白；pick 寫名稱", () => {
    const draft = toDraft(CURRENT, BUILTIN);
    draft.cells.verify.claude = { mode: "inherit" };
    draft.cells.review.copilot = { mode: "custom", value: "   " };
    draft.cells.verify.copilot = { mode: "custom", value: " gpt-5.6 " };
    draft.cells.verify.codex = { mode: "pick", value: "gpt-6-astra" };
    expect(fromDraft(draft, CURRENT)).toEqual({
      review: { claude: "opus", codex: "gpt-6.1-sol" },
      verify: { copilot: "gpt-5.6", codex: "gpt-6-astra" },
      choices: { codex: ["gpt-6.1-sol", "gpt-6-astra"] },
    });
  });

  it("清單一行一條、去空白、略過空行；清空的清單移除鍵", () => {
    const draft = toDraft(CURRENT, BUILTIN);
    draft.lists.claude = "  claude-x \n\n   \nclaude-y";
    draft.lists.codex = "  \n";
    expect(fromDraft(draft, CURRENT).choices).toEqual({ claude: ["claude-x", "claude-y"] });
  });

  it("沒改過的清單沿用現值（不重新拆行）", () => {
    const current: SubagentModelConfig = { ...EMPTY, choices: { codex: ["a\nb"] } };
    expect(fromDraft(toDraft(current, BUILTIN), current).choices).toEqual({ codex: ["a\nb"] });
  });

  it("只覆寫內建工具的格子與清單：未顯示的工具名稱原樣帶回", () => {
    const current: SubagentModelConfig = {
      review: { "my-agent": "model-a" },
      verify: { "my-agent": "model-b" },
      choices: { "my-agent": ["model-a"] },
    };
    const draft = toDraft(current, BUILTIN);
    draft.cells.review.claude = { mode: "pick", value: "opus" };
    expect(fromDraft(draft, current)).toEqual({
      review: { "my-agent": "model-a", claude: "opus" },
      verify: { "my-agent": "model-b" },
      choices: { "my-agent": ["model-a"] },
    });
  });
});

describe("sameSubagentModel", () => {
  it("內容相同（鍵序不同）為真；任一格或清單不同為假", () => {
    const reordered: SubagentModelConfig = {
      review: { copilot: "gpt-5.6", codex: "gpt-6.1-sol", claude: "opus" },
      verify: { claude: "sonnet" },
      choices: { codex: ["gpt-6.1-sol", "gpt-6-astra"] },
    };
    expect(sameSubagentModel(reordered, CURRENT)).toBe(true);
    expect(sameSubagentModel({ ...CURRENT, verify: {} }, CURRENT)).toBe(false);
    expect(sameSubagentModel({ ...CURRENT, choices: { codex: ["gpt-6-astra", "gpt-6.1-sol"] } }, CURRENT)).toBe(false);
  });
});

describe("choiceOptions", () => {
  // spec Example「各工具的下拉選項」——逐列對應。
  const flat = (options: ReturnType<typeof choiceOptions>) =>
    options.map((entry) =>
      "options" in entry ? `${entry.label}：${entry.options.map((o) => o.label).join("、")}` : entry.label,
    );

  it.each([
    ["claude", BUILTIN.claude, [], ["沿用主對話", "內建：sonnet、opus、haiku、fable", "自行輸入…"]],
    ["codex", [], ["gpt-6.1-sol", "gpt-6-astra"], ["沿用主對話", "自訂清單：gpt-6.1-sol、gpt-6-astra", "自行輸入…"]],
    ["copilot", [], [], ["沿用主對話", "自行輸入…"]],
    ["claude", BUILTIN.claude, ["opus", "claude-x"], ["沿用主對話", "內建：sonnet、opus、haiku、fable", "自訂清單：claude-x", "自行輸入…"]],
  ])("%s：內建 %j、自訂 %j", (_tool, builtin, custom, expected) => {
    expect(flat(choiceOptions(builtin, custom, LABELS))).toEqual(expected);
  });

  it("頭尾兩項的值為哨兵；組內選項的值為名稱；重複名稱與撞到哨兵的名稱不進選項", () => {
    const options = choiceOptions([], ["x", "x", SUBAGENT_CUSTOM], LABELS);
    expect(options[0]).toEqual({ value: SUBAGENT_INHERIT, label: "沿用主對話" });
    expect(options[1]).toEqual({ label: "自訂清單", options: [{ value: "x", label: "x" }] });
    expect(options[2]).toEqual({ value: SUBAGENT_CUSTOM, label: "自行輸入…" });
  });
});
