# Verify — docs-embedded-host

## Round 1

**Phase**: discovery
**Patch**: sha256:f91353b988ddec8d98baf478bf9d3bbeba1adaad9323cca5a54ae4c9b431da65
**Scope**: README.md, README.zh-TW.md, docs/assets/figures/en/embedded-shape.svg, docs/assets/figures/en/sdk-contents.svg, docs/assets/figures/en/shared-host-kit.svg, docs/assets/figures/en/skill-path-vs-verb.svg, docs/assets/figures/en/verb-four-steps.svg, docs/assets/figures/zh-TW/embedded-shape.svg, docs/assets/figures/zh-TW/sdk-contents.svg, docs/assets/figures/zh-TW/shared-host-kit.svg, docs/assets/figures/zh-TW/skill-path-vs-verb.svg, docs/assets/figures/zh-TW/verb-four-steps.svg, docs/assets/screenshots/en/server-setup.png, docs/assets/screenshots/zh-TW/server-setup.png, docs/configuration.md, docs/configuration.zh-TW.md, docs/development.md, docs/development.zh-TW.md, docs/getting-started.md, docs/getting-started.zh-TW.md, docs/product-status.md, docs/product-status.zh-TW.md, docs/remote-getting-started.md, docs/remote-getting-started.zh-TW.md, docs/roadmap.md, docs/roadmap.zh-TW.md, docs/sdk-node.md, docs/sdk-node.zh-TW.md, docs/server-backup.md, docs/server-backup.zh-TW.md, docs/server-deployment.md, docs/server-deployment.zh-TW.md, docs/server-store-drivers.md, docs/server-store-drivers.zh-TW.md, docs/verb-contract.md, docs/verb-contract.zh-TW.md, docs/workflow.md, docs/workflow.zh-TW.md, scripts/docs/docs-figures.test.mjs, scripts/docs/docs-parity.test.mjs, scripts/docs/remote-docs.test.mjs
- [SUGGESTION] speclink/changes/docs-embedded-host/proposal.md — Coherence: Impact 的 Modified 列了 `scripts/docs/docs-links.test.mjs`（tasks 2.1 的「涉及」也有），但該檔沒有改動；D9 本來就是「需要才調整」。建議把它從 Impact 拿掉，或註明「檢查後不需要改」。
- [SUGGESTION] docs/product-status.zh-TW.md — Coherence: Design decision not followed: D6（查核日期改成實作當天）。「本機專案的桌面 app」「安裝通路」「示範用 server 的後台、設定與帳號」三列這次改了文字，日期仍是 2026-10-15／2026-10-07／2026-10-07（`docs/product-status.md` 同列相同）。建議兩版一起改成 2026-11-18，或在 D6 寫明只有重新查核內容的列才換日期。
- [SUGGESTION] README.zh-TW.md — Coherence: Design decision not followed: D3（四件事每件連到 Node SDK 文件對應的章節）。第 3、4 件沒有連結（`README.md` 相同）。建議兩版都補上通往 `docs/sdk-node*.md#http-entry` 的連結。
- [SUGGESTION] docs/sdk-node.zh-TW.md — Coherence: Design decision not followed: D4 第 13 節（不複製清單）。「目前的限制」7 項裡有 5 項與 `product-status` 的已知缺口重複（`docs/sdk-node.md` 相同）。建議只留 SDK 自己的限制（musl、回傳值型別）加一行連結，或在 D4 註明保留與 SDK 直接相關的幾項。
- [SUGGESTION] docs/assets/figures/zh-TW/skill-path-vs-verb.svg — Coherence: Design decision not followed: D7（字型清單以 sans-serif 結尾）。`.mono` 類別的清單以 `monospace` 結尾，`docs/assets/figures/en/skill-path-vs-verb.svg` 相同。建議在 D7 補一句「路徑與指令用系統等寬字型、以 monospace 結尾」，或把這兩處改回一般字型。
- [SUGGESTION] docs/verb-contract.md — Correctness: 情境「文件與 CLI 一致」目前成立，但沒有常駐守門；`scripts/docs/docs-surface.test.mjs` 只掃 getting-started 與 workflow。建議把 `docs/verb-contract.md`／`docs/verb-contract.zh-TW.md` 的 `speclink …` 指令納入同一個守門。這項不在 D9 的要求內，可以另案。

## Round 2

**Phase**: validation
**Patch**: sha256:e3830ff8435d728e57a6c39c9ea6cbda8015a5729b714e1f52e13579106fad58
**Scope**: README.md, README.zh-TW.md, docs/getting-started.md, docs/getting-started.zh-TW.md, docs/product-status.md, docs/product-status.zh-TW.md, docs/remote-getting-started.zh-TW.md, docs/sdk-node.md, docs/sdk-node.zh-TW.md, docs/verb-contract.md, docs/verb-contract.zh-TW.md, scripts/docs/docs-figures.test.mjs, scripts/docs/docs-links.mjs, scripts/docs/docs-links.test.mjs, scripts/docs/docs-parity.test.mjs, scripts/docs/docs-surface.test.mjs, scripts/docs/remote-docs.test.mjs

## Round 3

**Phase**: validation
**Patch**: sha256:4b5a3cdc48670b470ded4a5b570bd9b2b37fc749c4f4e15697c9d2abeb1fb85f
**Scope**: scripts/docs/docs-figures.test.mjs
