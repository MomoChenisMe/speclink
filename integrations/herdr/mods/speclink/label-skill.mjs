#!/usr/bin/env node
// Claude Code 和 Codex 的 hook：看送出的訊息、模型叫用的技能和跑的 speclink 指令，幫這個終端機貼標籤，
// 面板才知道它在做哪個變更或討論。規則在 model.js 的 hookLabels。
// 設定方式見 integrations/herdr/README.md。不在 Herdr 裡或貼不上時什麼都不做。
// 不可以輸出任何東西：hook 的輸出會變成對話內容。
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { hookLabels } from "./model.js";

const terminal = process.env.HC_TERMINAL_ID;
if (terminal) {
  try {
    const labels = hookLabels(JSON.parse(readFileSync(0, "utf8")));
    if (labels) execFileSync("hc", ["label", terminal, ...labels], { stdio: "ignore", timeout: 3000 });
  } catch {
    // 貼不上標籤不能擋住使用者送出訊息。
  }
}
