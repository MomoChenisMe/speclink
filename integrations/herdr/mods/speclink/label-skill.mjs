#!/usr/bin/env node
// Claude Code 的 UserPromptSubmit hook：送出 /speclink-xxx <變更> 時幫這個終端機貼標籤，面板才知道它在做哪個變更。
// 設定方式見 integrations/herdr/README.md。不在 Herdr 裡或貼不上時什麼都不做。
// 不可以輸出任何東西：這個 hook 的輸出會變成對話內容。
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { promptLabels } from "./model.js";

const terminal = process.env.HC_TERMINAL_ID;
if (terminal) {
  try {
    const labels = promptLabels(JSON.parse(readFileSync(0, "utf8")).prompt ?? "");
    if (labels) execFileSync("hc", ["label", terminal, ...labels], { stdio: "ignore", timeout: 3000 });
  } catch {
    // 貼不上標籤不能擋住使用者送出訊息。
  }
}
