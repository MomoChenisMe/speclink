# Changelog
**English** · [繁體中文](CHANGELOG.zh-TW.md)

## 0.10.0 (2026-10-16)

### New features

- Every skill now tells the agent to talk to you in the language you use, so long skills such as apply and quality no longer switch to English halfway. `locale` still sets only the language of documents and reports, and it now also covers discussion records, the `context` and `rules` that `/speclink-config` writes, and `/speclink-audit` reports. This fork keeps its system-language default when `locale` is not set; remote work uses the server's language. Run `speclink update` to get the new skill files.

## 0.9.0 (2026-10-15)

### New features

- The desktop app has a new window shell. The left side has an icon rail with one tile per project, and a project column. The app draws its own title bar and breadcrumbs on all three platforms. "Add Workspace" is now "New project".
- Change and discussion details now open as full detail pages instead of a side panel. The breadcrumbs or Esc take you back to the board, and the board keeps its scroll position, search and filters. Use ‹ n/m › to go to the previous or next item in board order.
- Specs and archived items also open as full detail pages. A spec shows each requirement as its own section, with a requirement index and its provenance. To go back, use the back button, ⌘[ (Alt+← on Windows and Linux) or the mouse back button. On macOS, you can also swipe right with two fingers on the trackpad.
- Detail pages now use a three-column layout like a manual: a document tree on the left, a reading card in the middle, and an "On this page" list on the right in wide windows. The change actions move to the right end of the title bar.
- The six documents of a change each get their own view. Proposals and designs show section by section, tasks group into cards, delta specs mark modified and removed parts, and the plan and tickets show summary tiles and hints. The manual-task hint lists only the manual tasks that are not checked yet. The source files do not change.
- The four documents of a discussion each get their own view. The conclusion splits into the decision, cut cards and rejected options. The discussion log becomes a timeline, and derived changes group by cut.
- The spec detail page uses the same three-column layout, and the reading card of the Manual page matches the detail pages.
- Settings now open in a settings mode. App settings and project settings each list their categories in the left column, with one page per category. New options let you pick a light, dark or system appearance and turn automatic update checks on or off.
- Project context and output rules in project settings now use a code editor with line numbers and highlighting. Project context can switch between edit and preview, and ⌘S saves.
- Settings has a new "About Speclink" page with version information, a button that copies it, links to the source code and issue reports, and the licenses. "About Speclink" in the macOS menu opens this page.
- On macOS, the desktop app has its own menu bar. It follows the interface language and has shortcuts to open a project, switch pages, search the board and check for updates. A new "Keyboard Shortcuts" card in Settings lists the shortcuts that work on your platform.
- On Windows, a left click on the system tray icon opens the same panel as on macOS. A right click still opens the native menu.
- A project can write its own review rules and severities in openspec/config.yaml. `/speclink-review` grades findings by these rules, and `/speclink-config` and the desktop settings help you keep them.
- A review ticket records the review rules that the first round used. The Review tab in the desktop app shows which rules the review followed.
- `speclink init` and `speclink update` can install skills for GitHub Copilot. The skills go to `.github/skills/` and use the Copilot command syntax.
- New Claude Code plugin speclink-skills, with this repo as its marketplace. A row of colored skill tabs marks the current step and change. A side panel has Board, Discussions and Quality tabs. Type # in the prompt to pick a proposed change, and the skill row wraps to three rows in a narrow terminal.
- The changelog is now in English and Traditional Chinese. The repo has an English CHANGELOG.md and a Traditional Chinese CHANGELOG.zh-TW.md. The GitHub Release notes show English first, with the Chinese text in a collapsed section. The desktop app shows the entries in your interface language.
- Speclink has a new joined "S" logo and a wordmark for dark backgrounds.
- The server console no longer uses a side drawer. Users and projects open as detail pages with their own URLs, which you can reload or share. Invite, create and rename actions open a dialog in the center of the screen, and the dialog shows the invite link or the key directly.

### Fixes

- The server now blocks every write from the reader role, from the CLI and from the desktop remote board. Content in a wrong format gets a 400 response that tells you what is wrong, not an internal server error.
- In the English interface, counts now use the singular for one item, for example "1 scenario". English labels in the status rows of detail pages no longer overlap the values.
- When you open a change or a discussion from the system tray, the app first closes any dialog in the main window. The dialog and the details no longer show on top of each other.
- You can give `/speclink-ingest` the name of the change to update.

### Improvements

- The desktop app, the server console and the shared components have a new visual style: white surfaces with neutral grays, thin borders and flat components. Markdown uses a new renderer.
- Board columns and cards are now white containers with thin borders. Search and filters move to the page header, and the discussion column remembers if "Promoted" is open.
- The Specs, Archived and Manual pages now show lists with a paging toolbar. You can jump to a page and pick the page size, and the app remembers the page size after a restart.
- The notice about outdated skill files moves to a card at the bottom of the project column. When a new version is available, an update button shows in the icon rail, and you can download and install from it.
- The system tray panel uses the same white cards as the main window.
- The system tray panel shows the Proposed, In progress and Ready sections first. Content cards can collapse, and the frosted-glass effect on macOS is back.
- Promoted discussions in the system tray panel have their own card, so you can tell them apart from open discussions at a glance.
- Clickable items now give the same feedback everywhere. The desktop app keeps the arrow cursor and shows a hover background. The server console uses the hand cursor. Disabled and read-only items have no hover effect.
- Review and verify stamps on a detail page show only in the Quality gates row of the left status area, not again in the header.
- Summary tiles on a reading card share the full row width equally, and the discussion log lists rounds from newest to oldest.
- Task rows open or close by their check state: unchecked tasks are open and checked tasks are closed.
- The step bar on a discussion detail page uses the same color as its status tag, and the conclusion marks the cuts that are not promoted yet.
- The server console shell, page headers and the sign-in, setup, invite and device sign-in pages now look like the desktop app.
- The server console list pages, audit log and settings pages use the list and settings parts of the desktop app. The overview starts with four summary tiles, and revoke buttons show only when the pointer is on that row.
- Each quality-station round now runs only the affected tests and starts its three subagents at the same time, so a round waits about 40 minutes less. After a worktree merge, the stations no longer ask for `--base` every time.
- The 19 skills now follow one writing guide for all models. The rules are shorter and do not repeat, so models from Claude, Codex and Copilot follow them more easily.
- `/speclink-config` confirms all policy settings in one table. Review rules are optional, and the skill helps you decide if a rule belongs in review rules or in output rules.
- When you write review rules and your skill files are older than the engine, the CLI tells you to update them first. The `--dry-run` diff marks only the lines that really change.
- The README is now an English-first open-source home page. The Chinese version is README.zh-TW.md, and the repo adds CONTRIBUTING, SECURITY and issue templates.

## 0.8.0 (2026-09-25)

### New features

- `speclink plan` now builds waves from declared dependencies only. When two changes touch the same capability, plan no longer pushes one of them back. A small change with no dependencies can start at once. When two changes touch the same requirement, plan shows the archive order (which change waits for the other to archive) and name conflicts. The archive and commit skills warn you before an archive. After an archive, they name the later changes that must first rewrite against the specs. Changes with no board order now sort by dependent count, task count and creation time. The new `speclink change rank` puts a change before or after another one. At the end of propose and ingest, the skills move small, urgent changes up. To use the old algorithm, add `--strict-overlap` (local only).
- The Plan tab in the desktop app detail panel now shows four cards: Wave, Prerequisites, Overlaps and Archive order. A badge on the Prerequisites card shows if the change is blocked. The Overlaps card lists which change touches which requirement, and what each side does to it. The Archive order card lists the changes that must archive first. The cards show a warning for a name conflict or a mutual wait. Remote projects show the same information. With an older server, the Prerequisites card also lists the waits that the server finds from overlaps.

## 0.7.0 (2026-09-22)

### New features

- While a review or verification is in progress, the change detail panel in the desktop app shows Review and Verify tabs. They show the findings that hold up the ticket. Each round can collapse, and the last round is open. Each finding shows its severity, path and description, and accepted findings carry an Accepted tag. The tabs go away when the ticket gets its stamp or you discard it. If an archived change did not pass review or verification, you can still see the ticket that it took into the archive.
- `/speclink-discuss` now writes conclusions as lists. The Decision starts with a one-sentence verdict, then one point per line. Rejected alternatives and Deferred give each option and its reason on one line. Each cut that the discussion promotes gets its own bullet. The Conclusion tab in the desktop app no longer shows long blocks of text. Older discussion records stay as they are.

## 0.6.0 (2026-09-17)

### New features

- `speclink plan` calculates the execution order of all changes that are not archived. It groups them into waves (changes in the same wave can run in parallel), lists what blocks each change, and names the next change that is ready to start. `speclink change depends` declares which changes must finish before a change. The apply skill checks plan before it starts and stops when the change is blocked. At the end of propose, the skill records the dependencies that it finds.
- The board, the system tray and the detail panel in the desktop app follow the plan order. Cards show their wave, and blocked changes appear faded. A new Plan tab in the detail panel shows the prerequisites and lets you add or remove them. Drag-and-drop cannot move a card past a declared dependency. The board shows a warning when dependencies form a cycle.
- Remote mode has the same execution order. The server provides plan and writes prerequisites. When the desktop app connects to a server, its board and system tray work the same as locally, and so does the CLI in remote mode. When you add a prerequisite in the Plan tab, the candidates now come from the roster that holds the change. A change in a worktree no longer offers names that the engine rejects.
- After an archive, the archive and commit skills point to the next change that is ready to start. The "archive first, then commit together" flow of the commit skill now suggests an order before it archives and warns you when the manual is out of date.
- apply-with-worktree checks plan before it creates a worktree. With several change names, it separates the changes that can run in parallel from the changes that must wait for prerequisites. A blocked change no longer leaves an empty worktree behind. At the end of ingest, the skill checks the dependencies of the change again and records them.
- The opening scan of `/speclink-discuss` now includes the delta specs of changes in progress. The list of assumptions marks the requirements that "in-progress change X will change". A discussion no longer builds on specs that are about to change. Each change in `speclink list --json` now carries the list of capabilities in its delta.

### Fixes

- When a skill file notice appears (instruction files out of date, missing or newer), the bottom of the window no longer cuts off the board, the specs page, the manual page and the archived page. Each column scrolls to the end and shows the last card and the pagination bar at the bottom.
- On macOS, when you click a change row in the system tray panel to bring up the main window, hover effects, cursor styles and tooltips no longer stop working. The panel closes before the main window opens.
- A global CLI install with Yarn keeps the shim file, so the CLI runs after the install. The Homebrew formula now takes its checksum from the npm registry, so a release rerun stays in sync with the registry. On musl Linux, install.sh tells you that it supports only glibc.

## 0.5.0 (2026-09-14)

### New features

- The Release page now has 6 files instead of 21: one universal macOS dmg for both Apple Silicon and Intel, the Windows installer, Linux AppImages for two architectures, and the .app.tar.gz and latest.json files for automatic updates. The CLI now comes from one source on npm. `npm i -g @speclink/cli` installs the native executable, and Homebrew and `install.sh` get the same package. A Linux machine without a graphical interface can install the CLI with one curl line. The Linux desktop build no longer ships a .deb. If you use the .deb, remove it and install the AppImage.
- The desktop app checks for updates again when its window comes to the front. If you keep the app open for a long time, it still finds new versions. You do not need to click "Check for Updates" in Settings.

## 0.4.0 (2026-09-13)

### New features

- When you promote a discussion into changes in several cuts, add `--last` to the last cut. `discuss promote`, `new change --from-discussion` and `discuss seal` all accept it. When the last change goes to the archive, the discussion record goes with it. You no longer need to run `speclink discuss archive` by hand.

### Improvements

- The Traditional Chinese text now uses one term, 「正式規格」, for the specs that archive merges. It pairs with 「delta 規格」 in a change. The specs page and the manual in the desktop app, the README and the docs use the same words. The English text keeps "specs".

## 0.3.0 (2026-09-11)

### New features

- Each release has a plain-language changelog. The repo adds CHANGELOG.md. After an update, the desktop app shows "What's new in X.Y.Z" on the first launch. You can open it again from "Changelog" in Settings.
- `speclink validate` finds early the deltas that archive rejects: a target requirement that does not exist, a name collision, or a removed scenario that is not declared. You no longer find them only at archive time.
- A manual page can anchor its source to one requirement in a spec. The archive of another section no longer marks the whole page "Possibly outdated". When a regenerated page has the same text, only its timestamp changes.
- `/speclink-improve` adds a sixth friction signal: the grouping of folders and modules gets its own entry criteria.

### Fixes

- `speclink analyze` drops four false positives. It no longer asks you to copy a full design heading into tasks, and it no longer asks for scenarios in REMOVED blocks. The rules for the four dimensions are now part of the spec.
- A discussion that you keep open with `--hold` no longer loses its flag when you promote the next cut. It also does not go into the archive with the change. The board shows these discussions apart from the "promoted" discussions. To close one out, run `speclink discuss archive`.
- `speclink archive --mark-tasks-complete` no longer checks every box in tasks.md when it rejects the archive of a change with a structure that is not valid.
- A status update to a discussion record now changes only the frontmatter. The same string in the body no longer changes by mistake. During an archive, an active change with damaged meta now gets careful handling, so a discussion does not go into the archive by mistake.
- The release flow now polls for a newly published npm package. A registry delay no longer looks like a failure.

## 0.2.0 (2026-09-08)

### New features

- New `/speclink-manual` skill: with one sentence, it generates a wiki-style manual for new team members from the specs. It can also give you a tour of the system in the conversation.
- New Manual page in the desktop app: the sidebar tree, the search and the previous/next links come from the manual pages automatically. The page marks the pages that may be out of date.
- New `speclink trace` verb and `/speclink-trace` skill: they follow "spec → change → discussion → code" to tell you how a feature came to be and why it works this way.
- New `speclink discuss search`: it finds the decisions in archived discussions by keyword. The discuss and improve skills check old discussions when they start, so they do not bring back rejected directions.
- `discuss conclude --hold` keeps a discussion open, so one discussion can promote several changes over time.
- The conclusion now controls the life cycle of a discussion. A discussion without a conclusion does not go into the archive with a change. The board shows discussions that are "promoted but not concluded" as a separate group.
- "Add Workspace" in the desktop app remembers the workspaces that you opened recently. After you close a tab, you can open the workspace again with one click.
- New Workflow tab in the desktop app settings: view, switch, fork, create and delete workflow schemas.
- The provenance tags in the spec, change and discussion drawers of the desktop app are all clickable. You can go from spec to change to discussion and back in three steps.
- Claims in remote mode now save for real. The desktop remote board can claim a change and shows who works on it.
- When you check off a task in remote mode, the server keeps the evidence of the touched files. Drift, commit and archive provenance can find it later.
- `@speclink/engine` is now on npm. You can use the engine in your own scripts without a Rust toolchain.
- createEngine in the Node SDK can take an operator identity and provides stamp verbs. Stamps in multi-user systems are no longer anonymous.
- When you open a new capability, Speclink blocks a duplicate name and suggests similar names. You no longer get two specs with the same meaning.

### Fixes

- `speclink init --force` removes the skill directory of the previous tool when you switch tools. The notice for stale skill files also covers custom descriptors.
- The "Possibly outdated" check for manual pages now compares timestamps. A manual that you regenerate right after an archive no longer gets that mark by mistake.
- Lines that start with "## " in a discussion record no longer mix up the rounds, and they no longer count as the conclusion.
- Review and verification in remote mode now read the evidence on the server. They no longer stop and ask for manual input when no local evidence file exists.
- The baseline skill applies the specs output rules in config.yaml.
- The remote board now shows the creator, the start owner, the capability list and the group for promoted discussions.
- Change rows in the system tray panel now show the worktree marker.
- Quality station stamps support scope files that are not UTF-8.

### Improvements

- The onboard skill is now baseline (`/speclink-baseline`), to keep it apart from Onboard in OpenSpec.
- The handoff lines at the end of skills are clearer. After propose, they check the execution order. After apply, they say that you can archive directly. The worktree wrap-up lists `/speclink-quality`. After archive, they remind you to commit.
- Skill routing now lives in the description and the handoff lines of each skill. Speclink no longer injects a marker.
- The discuss skill has a new flow. It reads the specs first and then scouts. It picks a path by how clear the requirements are. Discussions with several requirements follow a fixed convention.
- The built-in workflow schemas now come down to one canonical schema. Its validation strength matches OpenSpec.
- The apply instructions payload now carries the effective TDD and audit values directly. The old policy keys no longer work.
- The source card in "Add Workspace" in the desktop app now has the name "Server", without the brand prefix.
