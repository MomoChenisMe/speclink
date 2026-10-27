---
topic: remote 模式下 desktop 與 AI 呼叫 CLI 每次都跳 macOS 鑰匙圈授權，按永久允許也不持久
slug: keychain-always-allow-reprompt
status: concluded
created: 2026-07-30
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: remote 模式下 desktop 與 AI 呼叫 CLI 每次都跳 macOS 鑰匙圈授權，按永久允許也不持久

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 remote 模式下，開啟 desktop 或 AI 呼叫 CLI 都會跳出 macOS 鑰匙圈授權框，且「永久允許」不持久。模式選 assumptions：掃到 crates/speclink-remote 的 credentials.rs / auth.rs / refresh.rs 與 Cargo.toml（keyring v3, apple-native），程式碼脈絡充足。相關變更：cli-desktop-credential-sharing（已封存 2026-07-29）——CLI 與 desktop 共用同一批 keychain entries（service `speclink-desktop`）正是該設計的刻意結果。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-30)

**Focus**: 為什麼按了「永久允許」，下次呼叫 CLI／開啟 desktop 仍會跳鑰匙圈授權框？
**Position**: 根因已驗證——兩個執行檔都是 adhoc（linker-signed）簽章，「永久允許」綁定的是簽章身分而非路徑，重建即失效：
- `codesign -d -r-` 實測：`~/.cargo/bin/speclink` 與 `/Applications/Speclink.app` 的 designated requirement 都是 `cdhash H"…"`（二進位內容雜湊），且 Identifier 帶 cargo metadata 雜湊後綴（如 `speclink-b6634622065a0b58`）
- 每次 cargo build / cargo install 重建 → cdhash 變 → macOS 視為全新陌生 app → 先前所有「永久允許」全數作廢
- 一個 origin 有三個獨立 keychain item（refresh / pat / bearer，見 credentials.rs:20-32），各有各的 ACL——按一次只授權一個 item，同一次呼叫可能連跳兩次
- CLI 與 desktop 是兩個二進位、各需授權；共用同批 entries 是 cli-desktop-credential-sharing 的刻意設計
- remote 模式每個 verb 都走 keyring（auth.rs 解析階梯：SPECLINK_TOKEN → keyring refresh → keyring PAT → credentials file；bearer 快取也在 keychain），授權一失效便每呼叫必跳
**Ruled out**: keyring crate 用法錯誤——get/set/delete 均正常，問題不在程式碼而在簽章身分不穩定
**Open**: 只解開發機體感，還是當產品問題處理（發行給其他 macOS 使用者）？

### Round 2 — assumptions (2026-07-30)

**Focus**: 修法範圍——只解開發機體感，還是當產品問題處理？
**Position**: 使用者裁定只解開發機體感，走方向 A（自簽憑證重簽）：
- 建一張自簽 code-signing 憑證，build 後以 `codesign --force -s <憑證> -i <固定識別碼>` 重簽 CLI 與 desktop
- designated requirement 從 cdhash 變成「憑證＋固定 identifier」，跨重建穩定 → 永久允許真正持久（重簽後首次仍會跳一次，授權後即沾黏）
- identifier 必須以 `-i` 固定——預設帶 cargo metadata 雜湊，依賴一動就變，DR 比對會失敗
- 正式版前提確認：Developer ID 簽章發行則無此問題；但若使用者以 cargo install 自建，產物仍是 adhoc，每次升級重跳一次
**Ruled out**: B（Developer ID）——年費 99 美元，對自用開發機超出範圍；C（bearer 移出 keychain）——治標，暫不另開 change；D（macOS 全退檔案存放）——refresh token 是長命憑證，不該放棄 OS 層加密落明文檔
**Open**: 無

## Conclusion

**Decision**: 開發機以自簽 code-signing 憑證＋固定 identifier 在 build 後重簽 CLI（~/.cargo/bin/speclink）與 desktop，讓鑰匙圈「永久允許」跨重建持久；不動產品碼，不開 change。
**Rationale**: 根因是 adhoc（linker-signed）簽章的 designated requirement = cdhash，重建即換身分；任何 keychain ACL 要持久，唯一條件是穩定的簽章身分。自簽憑證純屬開發機工序，成本最低且直擊根因。
**Rejected alternatives**: Developer ID（年費，對自用開發機超出範圍）；bearer 快取移出 keychain（治標不治本）；macOS 全退檔案存放（refresh token 長命，不落明文檔）。
**Deferred**: 正式發行時的簽章策略——Developer ID＋notarization 可徹底免除此問題；cargo install 自建使用者仍為 adhoc、每次升級重跳一次的體感，留待發行規劃。
**Capture to**: CLAUDE.md（開發備忘）
**Next**: 無 change；`speclink discuss archive keychain-always-allow-reprompt`
