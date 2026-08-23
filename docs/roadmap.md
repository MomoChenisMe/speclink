# Speclink Project Roadmap

[繁體中文](roadmap.zh-TW.md) · **English**

This document answers where Speclink is heading. It describes **direction, not a schedule**. Each line states the problem it solves, where it stands now, and what you will see after the next step lands. No version numbers and no delivery dates appear here.

## How to read this / 這份路線圖怎麼讀

- **The problem**: why this line exists. Without it, a direction is just a feature list.
- **Where it stands**: what works today, as judged by [Project Capability Status](product-status.md). That document is the canon for "can I use this yet"; this one does not maintain a second status matrix.
- **The observable next step**: something you can check yourself after it lands. For example: a command that runs, an entry that appears, or a documentation gap that becomes a tutorial. This is deliberate. Progress does not depend on an announcement.

Every line below is judged against **what actually exists today** — code that runs, the canonical specs under `openspec/specs/`, and the verdicts in the project capability status.

This document is the **direction for users**. It answers only whether the capability you are waiting for is on the way. To decide whether something works today, this is not the entry point — check the project capability status.

## SDK / SDK 發布

**The problem**

Driving the Speclink engine from your own program currently means installing a Rust toolchain, building `crates/speclink-node` yourself, and loading it by path. That is a steep price for "write a script that wires specs into an existing pipeline". What you install should be a package, not a build environment.

**Where it stands**

The N-API binding itself works: the Engine and Store bridge are connected and the dispatch contract is covered by tests. What is missing is distribution — `@speclink/engine` is not published to npm, so the [Node SDK documentation](sdk-node.md) demonstrates loading a repo-built artifact rather than an `npm install`. Rust users can already depend on the `speclink-core` crate directly.

**The observable next step**

`npm install @speclink/engine` succeeds in an empty folder, with no Rust toolchain on the machine. The Node SDK documentation then switches its loading example from a repo build to a package import. The Node SDK row in the project capability status also changes.

## Build your own client / 以引擎自建客戶端

**The problem**

The Speclink desktop app is **one frontend over the engine**, not the engine itself. The same typed command/query/context path is used by the CLI, the Server, and the Node binding. So you should be able to build your own thing on it. For example: a desktop app shaped to your team's flow, a VS Code extension, or an internal web board. The official frontend's trade-offs do not have to bind you.

This line shares an engine surface with the previous one, but it asks a different question. The SDK line asks "can I install it". This line asks "can I build my own client from documentation alone".

**Where it stands**

The engine surface is already shared. `speclink-core` is the single implementation of the rules. The Host composes authentication, revisions, transactions, and events. The [canonical Client Protocol](../openspec/specs/client-protocol/spec.md) defines the wire contract across clients. The [Verb and Flag Contract](verb-contract.md) records verb mode assignment and output guarantees.

The path for a third party to attach is what is missing. Today's documentation targets the maintainers of this repo. No "build a client from scratch" tutorial exists. The SDK is not on npm yet (see the previous line).

**The observable next step**

Building a minimal client — list changes, read a spec, check off a task — from public documentation alone, without reading Speclink's own source. If that works, the engine boundary is clear. If it does not work, documentation or interface is still missing. This line exists to close that gap.

## Remote collaboration / 遠端協作

**The problem**

Using Speclink alone in your own repo is already complete; sharing one spec canon across several people is not. Both the command line and the desktop board can now point at a remote — what is left is the stretches of that road still unpaved.

**Where it stands**

The remote command-line path works. Tests cover `link`, `auth`, and the read-only Context Projection. Nearly every verb has a remote arm; the [Verb and Flag Contract](verb-contract.md) records the mode assignment. Server-side installation, accounts, membership, and backup and restore all work too.

The desktop board points at remotes now: after signing in, the chooser picks a Project and Repo and offers either specs-only or a bound local checkout, and the board that opens browses changes, checks tasks, and reads and writes artifacts the way a local one does. The touched files a remote task check-off reports are stored too, and read back from the evidence endpoint.

Still unpaved: capability lists and change metadata are unsupported remotely, a discussion's `promotedTo` is filled with an empty list, offline and conflict handling are unfinished, and checking a task from the desktop remote board does not itself report touched files (only the CLI does), so that path still records no evidence.

**The observable next step**

Check off a task on the desktop remote board and see which files it touched — the way checking one from the CLI already does. And: work done while disconnected has a clear destination, with conflicts you can choose between on reconnect.

## Agent tool integration / Agent 工具整合

**The problem**

Speclink works with Agents today through generated skill files — slash commands in Claude, `$` commands in Codex. That path works. But it writes the workflow knowledge down for the model to read. It does not hand the model a callable tool. Only a callable tool lets an Agent look up a spec, open a change, or check off a task inside a conversation. Only then does the model skip a full re-read of the skill document.

**Where it stands**

The generated-skill side is mature: propose, apply, ingest, the quality stations, and archiving all have skills, across both Agent platforms (see the [Complete SDD Workflow](workflow.md)). The tool side has no usable entry yet — there is no installable Copilot tool package and no MCP adapter.

**The observable next step**

Mount Speclink as a server in an MCP-capable client. Then list changes and read specs with a direct tool call, instead of asking the model to shell out to the CLI. This step also has to close identity. The same rules the CLI uses must decide which account a tool call runs as, and which store it may write to.

## System integration / 系統整合

**The problem**

Speclink inside a company is not an island. Accounts should connect to the identity system already in place. Spec changes should notify other tools. Deployment should follow existing operational practice. Today you bridge all three by hand.

**Where it stands**

Server operations already work: installation, account management, PAT and device login, and backup and restore all have entries and end-to-end tests. But identity covers only the accounts the server manages itself. There is no enterprise SSO. To extend behavior you edit the codebase, because no runtime plugin mechanism exists. The deployment posture is a single instance, with no cluster operations. Backups still need a maintenance window.

**The observable next step**

Signing in to a Speclink server with your company's existing identity provider, without creating a second set of accounts. Two steps come after that: attach custom behavior without an edit to the codebase, and run across multiple nodes. Neither has an observable entry yet, and neither has a committed order. The project capability status shows them first.

## Related documents / 相關文件

- [Project Capability Status](product-status.md): what works today and what does not, with evidence and a last-audited date.
- [Node SDK](sdk-node.md): the engine interface and how to load it today.
- [Verb and Flag Contract](verb-contract.md): verb mode assignment, output guarantees, and endpoint contracts.
