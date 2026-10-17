# OWASP / CWE Preliminary Security Review

[繁體中文](security-review.zh-TW.md). Built-in offline rules, Desktop/CLI operations and independent redacted reports are implemented. The user's real Desktop, disconnected-network and visual acceptance run remains open in task 7.4 of `openspec/changes/add-owasp-cwe-preliminary-review/tasks.md`.

## Operation

1. Start the development Desktop with `npm run dev:desktop` and open an authorized test project.
2. Open project settings → Output Policy. Enable OWASP / CWE review beneath audit, select Web + API and **Top 10 2025 only**, leaving API Top 10, ASVS and CWE off. Choose spec/code/API-boundary reviews independently. Save and reopen to confirm persistence.
3. Open application settings → Security standards data. Explicitly download Top 10 2025; review source/version/license information. Nothing downloads merely by opening this page or starting a review.
4. For Remote projects, explicitly prepare the offline policy/spec snapshot in the change's Verify tab while connected. This uses existing authentication and the consistent Context Snapshot API. Repeating preparation refreshes the snapshot; failures preserve the old complete generation. No checkout means source-unavailable/NOT_VERIFIED, never scanning server paths or another project.
5. Disconnect the network and use **Run basic review** in the change's Verify tab. No verify.md or AI Agent is required. Cancel if needed; a started review preserves a partial report with reasons.
6. Inspect exact versions, confidence, evidence, checks, coverage and missing-data reasons. Source reveal revalidates the original project root and file identity; it does not execute content.
7. Export JSON/Markdown or copy a redacted summary. Export refuses existing paths and links: choose a fresh filename. Confirm test secrets never appear in output.
8. Check English/Traditional Chinese, light/dark themes, cancellation, reopening settings and the original TDD/audit/worktree/archive behavior on the actual platform. Automation cannot attest to these manual results.

The shared UI takes an optional `SecurityReviewAdapter`. server-web without local capabilities does not display a scan affordance. Older servers without `securityReview` do not start security work; policy controls show `unsupported_version`, while other settings remain available. Events and completion are bound to both jobId and projectKey, including after a project switch.

CLI examples:

```sh
speclink security data list --json
speclink security data refresh --json
speclink security data install owasp_top10 --version 2025 --json
speclink security data import /absolute/path/standards.zip --json
speclink security prepare --root /authorized/checkout --change demo --json
speclink security run --root /authorized/checkout --change demo --json
speclink security rules --json
speclink security report REPORT_ULID --format markdown
speclink security data remove owasp_top10 --version 2025 --json
```

`run`, `report`, `rules` and installed-data queries are offline. Only explicit refresh/install and Remote preparation can connect. Stdout is one JSON value or the selected human format; progress goes to stderr. Complete runs, including potential findings, exit 0; incomplete/unverified coverage exits 1; cancellation exits 130; syntax errors remain 2. No external scanner or project script is invoked.

## Policy and compatibility

The sole policy source is `openspec/config.yaml` → `security_verification`. Missing/null defaults to disabled. Rust, Desktop and CLI share the typed model and leaf-patch behavior. Remote writes preserve editor permissions and revision/CAS semantics. Unknown fields, comments and untouched policies survive round trips. The patch command accepts a YAML mapping up to 64 KiB through stdin, optionally `--dry-run`.

```yaml
security_verification:
  enabled: true
  scope: web-api
  standards:
    owasp_top10: {enabled: true, version: "2025"}
    owasp_api_top10: {enabled: false}
    owasp_asvs: {enabled: false}
    cwe: {enabled: false}
  reviews: {spec: true, code: true, api_boundary: true}
  exclude_dirs: []
  disabled_rules: []
```

Scopes are off/web-api/api/web. Scope off is unspecified scope, not the master switch. Each standard is independent: Top 10 2021/2025, API Top 10 2019/2023, ASVS 4.0.3/5.0.0 with L1–L3, CWE installed versions. No ASVS or CWE prerequisite exists for Top 10. All standards off remains explicitly enabled-but-unconfigured; independently enabled own rules are reported separately. Saving a valid policy does not require installed data. Missing data cannot become PASS.

## Sources and licensing

`crates/engine/speclink-core/assets/security/sources.json` is SpecLink's source/version/hash/attribution catalog. Official full text is not bundled or relicensed as MIT. Users explicitly download/import it into application data. Raw and parsed representations retain source URL, version/revision, license name/URL, attribution, download time, hashes, integrity mode, installation state, format and modification notice. Share-alike derivatives remain separate from application code.

| Data | Fixed source | Applicable license |
| --- | --- | --- |
| Top 10 2021/2025 | OWASP/Top10 commit `3a31f35346c3f4e90f350395124382fa53af2afe` | [CC BY-SA 4.0 repository LICENSE](https://github.com/OWASP/Top10/blob/3a31f35346c3f4e90f350395124382fa53af2afe/LICENSE), not a different website footer |
| API Top 10 2019/2023 | OWASP/API-Security commit `33cea37b2ffde3e2e62f6b5d79a029a22a846633` | [CC BY-SA 4.0](https://github.com/OWASP/API-Security/blob/33cea37b2ffde3e2e62f6b5d79a029a22a846633/LICENSE) |
| ASVS 4.0.3 | `v4.0.3_release`; license commit `d31389d213b320cc2ef0cb1c30792cc80104d8ff` | [CC BY-SA 3.0](https://github.com/OWASP/ASVS/blob/d31389d213b320cc2ef0cb1c30792cc80104d8ff/LICENSE.md), not the newer license |
| ASVS 5.0.0 | `v5.0.0_release`; license commit `5cf9b032440be53ce345ab3c130fda46ba1ce7a2` | [CC BY-SA 4.0](https://github.com/OWASP/ASVS/blob/5cf9b032440be53ce345ab3c130fda46ba1ce7a2/LICENSE.md) |
| CWE 4.20 | Fixed MITRE XML ZIP; version checked against XML root | [MITRE CWE Terms of Use](https://cwe.mitre.org/about/termsofuse.html); retain copyright, terms and disclaimer |

Top/API mappings require a MITRE definition link with a matching CWE label inside the selected version's explicit mapped/related-CWE section. Narrative references are not mappings; the current API source documents do not provide these formal mappings. IDs always include standard and version. ASVS 4.0.3 parses separate L1/L2/L3.Required flags; 5.0.0 expands the minimum level cumulatively. Empty structural sections create no requirements, while an entirely empty standard is rejected. ASVS requirements without a reliable direct checker stay NOT_VERIFIED. Release assets lacking official checksums use source-TLS plus recorded actual hashes, never a claimed official checksum.

Refresh checks only maintainer-approved candidate versions through official machine-readable endpoints. It cannot replace download URLs, hashes, licenses or rules with arbitrary remote data. New versions/formats require a catalog/parser update. Failed downloads/parses leave existing installed data usable.

An offline ZIP package contains `manifest.json`, root `LICENSE` and the fixed files under `raw/`; an empty `raw/` directory entry is allowed. Manifest fields are `formatVersion:1`, `standardId`, `version`, `sourceRevision`, `sourceUrl`, `sourceFormat`, `licenseName`, `licenseUrl`, `attribution`, `entries:[{name,sha256}]`. Source/license metadata must match the catalog and every file must have a matching digest. CWE can contain the original XML ZIP or decoded XML; both must agree if supplied together. Import records **imported** trust, not publisher authenticity. Reject ZIP64, encryption, multi-disk, unsupported compression, duplicate/case-colliding paths, links, executables, special files, unlisted paths and decompression bombs. Official CWE 4.20's single XML entry has mode 100750: only its dedicated inner decoder ignores this permission metadata; no executable permissions are installed, and the outer package still rejects executable entries.

## Architecture, persistence and safety

| Layer / modified locations | Responsibility |
| --- | --- |
| `speclink-core/src/security/`, `assets/security/` | Pure policy/parser/rules/report/preparation models; 19 own rules and source catalog; no filesystem, HTTP or process execution |
| `speclink-fs/src/security.rs`, `security/{scan,prepared,reports}.rs` | Authorized bounded input, no-follow opens, ZIP validation, atomic generations, locks/pins/references, persistence and export |
| `speclink-host/src/{security,security_data}.rs`, context/policy | Shared offline orchestration, deadlines/cancellation and explicit data transport; Host receives repository interfaces instead of depending on Fs |
| CLI `verbs/security.rs`, config/instructions/main | Offline commands, explicit authenticated Remote prepare, one Ctrl+C handler |
| Desktop core/security, Tauri/security_commands + remote | Background jobs and thin IPC, immutable target identity, report/reveal/export boundaries |
| Desktop adapter/security, WorkflowPage, SecurityDataPage, session/App | Existing-policy settings, application data management and Desktop wiring |
| UI security types/Panel, ChangeDetailPage, i18n | Optional adapter, independent Verify information and plain-text evidence |
| Protocol binding/query, remote client, server routes/identity, instructions and five skill sources | Capability/version compatibility, advisory workflow integration and regenerated Codex/Claude/Copilot assets/goldens |

Device paths use the same CLI/Desktop resolver: macOS `~/Library/Application Support/speclink`, Windows `%LOCALAPPDATA%/speclink`, Linux `$XDG_DATA_HOME/speclink` or `~/.local/share/speclink`, existing portable data convention. Missing absolute application data paths fail explicitly, without a project-directory fallback.

Standards live under `security/standards/<id>/<version>/generations/<ULID>`. Staging, size/structure/version/hash checks and atomic current-pointer replacement precede publication. Version locks and reader pins protect active generations. Cleanup only removes staging older than 24 hours under locks and unpinned old generations. Post-publication cleanup errors return successful installation plus warnings. Windows uses ReplaceFileW, never delete-first replacement. File::try_lock requires Rust 1.89+. Known local policy references and prepared snapshots protect version deletion; external policy edits require rereading to refresh references. Unknown/unopened Remote projects are not enumerated; deletion reports this scope honestly and never changes policies or reports.

Remote snapshots bind origin/project/repo/canonical checkout/change and retain policy revision, timestamp and per-document digests. `policy.json` and `snapshot.json` are separate immutable files; disabled/code-only runs need not read all specs. Preparation limits policy to 64 KiB, each spec to 1 MiB, combined specs to 100 MiB and the actual JSON response to 256 MiB. Auth failures preserve old snapshots; run creates no client, performs no handshake and changes no Context Projection.

Reports live under `security/reports/<project-key>/<ULID>`. JSON, Markdown and a hash manifest publish as one generation; latest is a recoverable index. A failed index update does not undo a published report. Read checks both formats and hashes. There is no automatic report deletion. Reports survive archive and never write quality tickets or stamps. Evidence is masked again at the report boundary: secrets entirely, other evidence's string literals, URL credentials and token-like values, up to 400 characters. Unsafe-to-segment evidence is fully masked. Markdown escapes HTML/formatting; UI renders source as text. Source text exists only in transient scan memory. Export refuses overwrite; Unix publishes a fully written temporary file atomically, while Windows uses exclusive, verified-handle creation (real platform validation pending).

## Coverage and limits

| Area | Coverage / bound |
| --- | --- |
| Code | Rust/TypeScript/JavaScript: potential hardcoded secrets, SQL concatenation, unsafe HTML, shell concatenation, full error disclosure, explicitly unsafe settings. JSON/YAML: listed unsafe settings only. Other languages NOT_VERIFIED |
| SDD specs | Selected proposal/design/tasks/specs; fixed Traditional Chinese/English/Japanese phrases identify missing applicable authentication, authorization, type/length, sensitive-data, error and acceptance requirements; absence is not a confirmed source vulnerability |
| API | Seven static OpenAPI 3.0/3.1 JSON/YAML checks; internal refs only. External/cyclic/unresolved refs and unsupported combinations/custom schema remain NOT_VERIFIED; no API calls. Runtime behavior remains unverified |
| ASVS | Exact version/level requirements retained as NOT_VERIFIED without a reliable direct checker; no certification |
| Input | 1 MiB/file, 100 MiB total, 10,000 files, depth 32, 60 seconds; additional bounded directory enumeration. Symlinks/reparse, binary/unsupported files, errors and limits retain reasons |
| Rules | 100 findings/file, 10,000 total; bounded long-line/window/OpenAPI node/operation handling. Disabled rules and enabled review families remain independent |
| Data | 64 MiB downloaded, 256 MiB unpacked, 128 entries, XML 128 MiB, others 2 MiB/file, 120 seconds; no XML entities/DTDs/commands |
| Exclusions | .git, node_modules, target, dist, build, .speclink/context and recognized generated markers; root-relative exclude_dirs can add paths |
| Reports | 1,000 reports / 200 MiB total, no automatic deletion; storage-limit errors preserve previous reports |

States: **PASS** only covers rules actually executed; **FINDING** is a potential risk or missing requirement needing confirmation; **NOT_VERIFIED** retains unsupported/incomplete/error reasons; **NOT_APPLICABLE** needs a reason; **DISABLED** is user choice. None implies absence of vulnerabilities, an OWASP certification or complete ASVS compliance.

SDD `securityVerification` contains enabled, scope, exact selected standards, enabled review families, executionMode, fixed requirements and dataAvailability. `device_local_check_required` means Engine/server cannot assert this device's installation status: query `security data list`. Downloaded documents are untrusted data, never Agent instructions. AI is optional and separate: users may manually share the redacted summary with their configured Agent; networked AI is not called offline. Preliminary findings add no Archive gate and do not change existing TDD/audit/quality-ticket/stale-stamp behavior.

## Maintenance and validation

Add a unique SL ID, languages, type, severity, description, remediation and trustworthy CWE references to `assets/security/rules.json`. A selector only chooses a checker implemented in application code; external regex/code/shell/plugins are not accepted. Write positive, negative, false-positive and unsupported tests first, implement bounded/cancellable checks, and update the rule version/report tests. No reliable official mapping means retain only the SL ID. Add new data versions only after checking actual per-version licensing, fixed sources and formats, with parser/offline fixture tests.

Common reason codes: missing_data, unsupported_version/language, invalid_package/format, version_mismatch, integrity_error, license_missing, busy, in_use, storage_limit, budget_exceeded, cancelled, missing_preparation, binding_mismatch, invalid_config, source_unavailable. Desktop exposes code/message/stage/target; unknown failures collapse to review_error/data_error without raw server bodies or secrets. Scan failures never become PASS.

Dependencies reuse already installed versions: quick-xml 0.41.0 (MIT, Rust 1.79), ureq 2.12.1 (MIT/Apache-2.0, Rust 1.71), webpki-roots 0.26.11 (CDLA-Permissive-2.0), zip 4.6.1 (MIT, Rust 1.82) with deflate-flate2 only, flate2 1.1.9 (MIT/Apache-2.0, Rust 1.67), libc/windows-sys and existing serialization/hash/time crates. Only three new lockfile packages were required for CLI cancellation: ctrlc 3.5.2 (MIT/Apache-2.0, Rust 1.69), nix 0.31.3 (MIT, Rust 1.69; signal/process only), cfg_aliases 0.2.2 (MIT build helper; no declared MSRV, built successfully). ctrlc has no termination feature. Actual registry manifests/licenses and resolver diff were checked. These are ordinary format/OS libraries, never scanners. Desktop's third-party notice generator includes packaged dependencies.

Ordinary tests use synthetic self-owned fixtures, no official full-text redistribution and no network. The installed-data/CLI journey labels synthetic data Imported. Explicitly obtained official fixed files can be checked with `SPECLINK_SECURITY_OFFICIAL_FIXTURES=<dir>` and the ignored `security_standards_official_repository_snapshot` / `security_data_official_fixtures_install_and_read_offline` tests. Directory format: `<standard_id>_<version>/<fixed filename>` plus LICENSE; CWE also supplies the original ZIP and decoded XML. These tests do not download or touch real user application data. CLI SIGINT is tested with a real Unix signal, exit 130 and a saved partial report; Windows signal/reparse/export paths were not exercised on this macOS host.

Detailed command results and the full changed-file list are recorded in the [Traditional Chinese delivery record](security-review.zh-TW.md). Pending work is the user's real Desktop acceptance and other-platform runtime checks. Dependency vulnerability scanning, DAST, executing APIs, full semantic analysis, complete ASVS verification, Security Gate and automatic AI review are outside this implementation.

Final macOS validation (2026-10-17): 2,826 ordinary Rust tests passed plus two explicitly enabled official-data fixture tests; UI 1,323, Desktop 1,234, server-web 244 and script tests 364 passed. Both TypeScript checks, both frontend builds and the native Desktop cargo check passed. Assets v1.49.1 were regenerated and checked without update flags. Three PostgreSQL cases were skipped because no test instance was configured; Windows and the real Desktop acceptance remain unexecuted. Global cargo fmt check still fails on pre-existing formatting (confirmed against HEAD), while git diff --check passes. Artifact analysis: no Critical/Warning, 44 Suggestions; validation: valid with similar-name hints. See the Chinese record for exact commands, counts and limitations.

Hands-on feedback follow-up: the active download/reinstall row now immediately reads “Downloading” until completion. Project settings provide six device-local icon colors plus Default; selected custom icons use deep backgrounds and white text, while unselected backgrounds mix in 50% white with dark text, and the tray shares the preference. Traditional Chinese/system Traditional Chinese follow Taiwan wording conventions. Desktop regression: 1,244 tests in 49 files passed; script checks 364 passed; TypeScript and production build passed. The original manual acceptance task remains open.

Second icon-color follow-up (superseded by the third adjustment below): all six custom colors now use deep backgrounds and white text in the rail, tray and settings previews, independent of selection. Purple and pink retain distinct hues; stored preferences work without migration. Nine regression cases failed before the fix; targeted Desktop tests passed (274 tests in five files), as did 364 script tests, TypeScript, the production build and git diff --check. The existing large-chunk build warning remains; real Desktop visual acceptance is still pending.

Third icon-color adjustment: unselected backgrounds mix the deep color with 50% white and use dark text of the same hue; selected icons retain deep backgrounds with white text. The rail, tray and settings previews share this behavior without a preference migration. Targeted Desktop tests (274 in five files), TypeScript and the production build passed; all six mixed colors appear in the built CSS. Script checks (364) and git diff --check passed. The existing large-chunk warning and pending visual acceptance remain.

First quality-round fixes: source/license links reuse the system-browser opener and error feedback, with permissions limited to official OWASP and MITRE locations and tested against the source catalog. Standard association now checks cancellation/deadline callbacks before and during matching, preserving NOT_VERIFIED reasons. Merged reviews share a 100-finding quota per finding path and the existing 10,000 global quota; the Host regression fixture drops from 200 to 100 findings without consuming another file's quota. The instructions spec now documents the existing standards/reviews array summary, distinct from the unchanged full workflow-config show policy. CLI tests verify this shape for each artifact.

Fix validation: core/host/desktop-core 1,625 tests, CLI instructions_policy 9 and Desktop 1,253 tests in 49 files passed; one official-data fixture remains ignored. Desktop TypeScript, production build and native cargo check passed, with the existing large-chunk warning. Reproduction files stay in temporary storage. Spec validation, 364 script checks and git diff --check passed. Other packages' complete suites remain CI guarded; this does not claim Windows runtime or exhaustive scenario verification, and quality stamps await validation and the user's decision.
