# Roadmap

[繁體中文](roadmap.zh-TW.md) · **English**

This document collects the directions that Speclink does not complete yet. It describes directions, not a schedule. Thus it has no version numbers and no delivery dates.

Each direction answers three questions:

- **The problem**: why this direction exists.
- **Where it is now**: what works today. [Product status](product-status.md) is the reference for what is available.
- **The next visible step**: what you can check yourself when the step is done.

## <a id="sdk"></a>SDK

**The problem**

People who want to drive the Speclink engine from their own program must be able to install one package, not a full build toolchain. After the install, the calls must be easy to write, and the errors must be clear.

**Where it is now**

- `npm install @speclink/engine` works without a Rust toolchain. npm has native packages for five platforms.
- Tests protect the engine, the Store bridge, and the `dispatch` contract. For usage, see the [Node SDK](sdk-node.md).
- Not done yet:
  - Typed JS methods. Now all verbs go through `dispatch` with string arguments.
  - A contract version handshake: when the JS package and the native binary have different versions, the engine must stop with an error, not fail silently.
  - Limits for the Store bridge: concurrency, timeouts, and cancellation. The bridge has none of these now.
  - Rust users can depend on `speclink-core` only through Git. The crate is not on crates.io.

**The next visible step**

You list changes and read specs with typed methods, and your editor completes the arguments. When the JS package and the native binary have different versions, the error message shows both versions.

## <a id="own-client"></a>Build your own client or server

**The problem**

The desktop app is only one front end of the engine. The CLI, the server, and the Node SDK use the same path for commands, queries, and context. You must be able to build your own tools with it too. Examples: a desktop app for your team flow, a VS Code extension, an internal web board, or a server with your own authentication and database.

**Where it is now**

- One crate, `speclink-core`, holds all the rules. The Host handles authentication, revisions, transactions, and events.
- The [Client Protocol spec](../openspec/specs/client-protocol/spec.md) defines the wire contract for all clients. The [Verb and flag contract](verb-contract.md) shows the local and remote mode of each verb and its output shape.
- Missing:
  - A document that shows how to build a client from zero.
  - A remote HTTP client in JS. Now only the Rust `speclink-remote` client exists.
  - UI components and an integration contract for other teams. `@speclink/ui` is a private package now.

**The next visible step**

You build a small client from the public documents only, without the Speclink source code. The client lists changes, reads a spec, and checks a task. Each place where you get stuck shows a missing document or interface.

## <a id="remote"></a>Remote collaboration

**The problem**

For one person in one repo, Speclink is complete. When several people share one set of specs, some parts of the path are not ready yet.

**Where it is now**

- The Remote CLI works: `link`, `auth`, the read-only projection, and a remote version of most verbs, `plan` and dependency declarations included.
- The desktop app opens remote boards. You can browse changes, check tasks, read and write artifacts, and see the creator, the claimant, and the capability list.
- The Store keeps claims. A claim on a change that another person holds fails, and the message names the holder.
- When the connection drops, the board becomes a read-only snapshot and rejects all writes. After the connection returns, the board updates by itself.
- Not done yet:
  - When you check a task on a remote board in the desktop app, report the touched files (the CLI does this already).
  - Release and takeover of a claim.
  - A view of the audit history. The Store keeps a history of each document that nobody can change, but no API or screen shows "who changed what, when, and with which command".
  - Cleanup of the local projection cache: removal after a time limit, removal at logout or at a project switch, and read access for the owner only. Now the projection is read-only and stays out of Git.
  - A handoff gate from PM to engineer (an approval that expires when somebody edits the document), and a record of the spec version that apply used. Quality stations and claims cover part of this need. A new decision about this item is necessary.

**The next visible step**

You check a task on a remote board in the desktop app, and you can see the files that it touched, the same as in the CLI. When a claimant leaves, you can release or take over the claim.

## <a id="agent-tools"></a>Agent tool integration

**The problem**

Now Speclink works with agents through generated skill files. A skill file gives workflow knowledge to the model as text. It does not give the model tools to call. Only tools let an agent read specs, start changes, and check tasks directly in the conversation, without a full read of a skill each time.

**Where it is now**

- The skills are mature. Claude Code and Codex have a skill for each stage. See the [SDD workflow](workflow.md). Other AI tools can get skills through a custom tool descriptor. See [Configuration](configuration.md).
- The Claude Code plugin `speclink-skills` adds a skill button row and a side panel.
- The tools have no entry point yet: no tool package to install and no MCP adapter.
- The direction under discussion (not final):
  - First, `@speclink/engine` gets one general tool that takes CLI arguments and stdin, with examples that connect it to agent SDKs.
  - The host side sets the identity of each tool call. The model cannot set its own identity.
  - The MCP adapter comes later. Without a checkout, it gives context through MCP resources.

**The next visible step**

You add Speclink as a tool in an agent framework, and you list changes and read specs with tool calls, without asking the model to run the CLI. Tool calls use the same rules as the CLI for the account and for the Store that they can write to.

## <a id="system"></a>System integration

**The problem**

In a company, Speclink cannot be an island. Accounts must come from the existing identity system. Spec changes must send notifications to other tools. Deployment must follow the existing operations.

**Where it is now**

- Server operations work: install, account management, access tokens, device login, and backup and restore all have entry points and end-to-end tests.
- The server sends an SSE event stream (`/events`), so clients see changes when they happen.
- Not done yet:
  - Enterprise SSO (OIDC). Now the server manages its own accounts only.
  - Notifications to other systems, for example webhooks.
  - Runtime plugins: change the Store or add custom behavior without a code change.
  - Multi-node deployment. Now the server is one instance.
  - Online backup without a server stop.
  - A WebSocket transport (low priority).

**The next visible step**

You log in to the Speclink server with the identity provider of your company, without a separate account. The other items have no visible entry point and no order yet. When they move, [Product status](product-status.md) shows it first.

## <a id="related"></a>Related documents

- [Product status](product-status.md): what is available now, with evidence and check dates.
- [Node SDK](sdk-node.md): the current engine interface and how to load it.
- [Verb and flag contract](verb-contract.md): the mode of each verb, the output shapes, and the endpoints.
