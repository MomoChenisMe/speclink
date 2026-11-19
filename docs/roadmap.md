# Roadmap

[繁體中文](roadmap.zh-TW.md) · **English**

This document collects the directions that Speclink does not complete yet. It describes directions, not a schedule. Thus it has no version numbers and no delivery dates.

Each direction answers three questions:

- **The problem**: why this direction exists.
- **Where it is now**: what works today. [Product status](product-status.md) is the reference for what is available.
- **The next visible step**: what you can check yourself when the step is done.

The [known gaps](product-status.md#known-gaps) in Product status list the items that this version delays, with what you can do today. The items that belong to a later direction have a place below.

## <a id="sdk"></a>SDK

**The problem**

People who want to drive the Speclink engine from their own program must be able to install one package, not a full build toolchain. After the install, the calls must be easy to write, and the errors must be clear.

**Where it is now**

- `npm install @speclink/engine` works without a Rust toolchain. npm has native packages for five platforms.
- The SDK is a toolbox with four parts: the engine, the host kit, the store port contract, and the conformance tests. Tests protect them and the `dispatch` contract. For usage, see the [Node SDK](sdk-node.md).
- Not done yet:
  - Typed JS methods. Now all verbs go through `dispatch` with string arguments, and the results have no types of their own.
  - A contract version handshake: when the JS package and the native binary have different versions, the engine must stop with an error, not fail silently.
  - Timeouts and cancellation for the store functions and the hooks. Now the engine waits until your function returns. If you need a timeout, put it inside the function.
  - Rust users can depend on `speclink-core` only through Git. The crate is not on crates.io.

**The next visible step**

You list changes and read specs with typed methods, and your editor completes the arguments. When the JS package and the native binary have different versions, the error message shows both versions.

## <a id="own-client"></a>Build your own client or embed the engine

**The problem**

Speclink Desktop is only one front end of the engine, and the official server is only for demonstration and trial. You must be able to build your own tools with the same engine. Examples: a desktop app for your team flow, a VS Code extension, an internal web board. You must also be able to embed the engine in your own system, with your own accounts, login, permissions, and database.

**Where it is now**

- One crate, `speclink-core`, holds all the rules.
- You can embed the engine today. The host kit (`createHost`) handles the identity and the scope of each call, and two hooks: one can read data and then allow or refuse a command before it runs, and one only watches after the commit.
- The store port has only three functions: list, read, and commit. You write them for your own database. Then you check them with the conformance tests in the package (`runStoreConformance`).
- There are two entry points. They share one store and one set of hooks:
  - The HTTP entry point (`host.handleRequest`): you add one route, and the original CLI and Speclink Desktop can connect.
  - The call entry point (`host.dispatch`): for your own Agent tool.
- There are three store port examples, all in Rust: SQLite, PostgreSQL, and the file system. The official demonstration server is made of them and the same host kit. See [Store port examples](server-store-drivers.md).
- The [Client Protocol spec](../speclink/specs/client-protocol/spec.md) defines the wire contract for all clients. The [Verb and flag contract](verb-contract.md) shows where each verb works (local projects, remote projects) and its output shape.
- Not done yet:
  - A store port example in JavaScript. Now people who embed with Node write their own with the Rust examples as a reference.
  - Push events between several host instances. The host kit declares polling only by default. For push, the host must supply the stream and the transfer between instances itself.
  - A document that shows how to build a client from zero.
  - A remote HTTP client in JS. Now only the Rust `speclink-remote` client exists.
  - UI components and an integration contract for other teams. `@speclink/ui` is a private package now.

**The next visible step**

You embed with Node, take a store port example in JavaScript, change it to your own database, and it passes the conformance tests. You build a small client from the public documents only, without the Speclink source code. The client lists changes, reads a spec, and checks a task. Each place where you get stuck shows a missing document or interface.

## <a id="remote"></a>Remote collaboration

**The problem**

For one person, a local project is complete. When several people share one set of specs in a remote project, some parts of the path are not ready yet.

**Where it is now**

- The CLI for remote projects works: `link`, `auth`, the read-only projection, and most verbs work in both kinds of project, including `plan`, dependency declarations, `change rank`, and trace.
- There are three login methods: browser login, device login, and access tokens. The Server supplies the skills. After you connect, run `speclink update` to get them.
- Speclink Desktop opens the board of a remote project. You can browse changes, check tasks, read and write artifacts, and see the creator, the claimant, and the capability list. It can also migrate a local project to a Server.
- The Server keeps the claimant. A claim on a change that another person holds fails, and the message names the holder. To release a claim, run `speclink claim <change> --release`. Each person with write access can run it.
- `speclink discuss restore <slug>` makes an archived discussion a live record again. `speclink language replace --stdin` writes the full glossary.
- The CLI and Speclink Desktop use the trust store of the operating system to check the TLS certificate of the Server. If a CA of your organisation issued the certificate, install that CA into the trust store of the operating system, and the connection works.
- You can read the state of review and verify, the branch of a change, and the landed commit. `speclink change landed` reports the landed commit.
- When the connection drops, the board becomes a read-only snapshot and rejects all writes. After the connection returns, the board updates by itself.
- Not done yet:
  - When you check a task of a remote project in Speclink Desktop, report the touched files (the CLI does this already).
  - A button in Speclink Desktop that releases a claim. Now you run `speclink claim <change> --release` in the CLI.
  - Manual generation for remote projects. Now only the tour mode works. The spec baseline (`baseline`) also works only in a local project.
  - Custom artifact workflows for remote projects. Now only the two built-in schemas work.
  - Import and export verbs in the CLI. Now you move a project with Speclink Desktop or with the program of the host.
  - A view of the audit history. Each commit gives the store the actor and the command, but no API or screen shows "who changed what, when, and with which command".
  - Cleanup of the local projection cache: removal after a time limit, removal at logout or at a project switch, and read access for the owner only. Now the projection is read-only and stays out of Git.
  - A handoff gate from PM to engineer (an approval that expires when somebody edits the document), and a record of the spec version that apply used. Quality stations and claims cover part of this need. A new decision about this item is necessary.

**The next visible step**

You check a task of a remote project in Speclink Desktop, and you can see the files that it touched, the same as in the CLI. You can release a claim in Speclink Desktop, and the CLI is not necessary for this. You can generate a manual in a remote project too.

## <a id="agent-tools"></a>Agent tool integration

**The problem**

Speclink works with agents mainly through generated skill files. A skill file gives workflow knowledge to the model as text. It does not give the model tools to call. Only tools let an agent read specs, start changes, and check tasks directly in the conversation, without asking the model to run the CLI.

**Where it is now**

- The skills are mature. Claude Code and Codex have a skill for each stage. See the [SDD workflow](workflow.md). Other AI tools can get skills through a custom tool descriptor. See [Configuration](configuration.md).
- The Claude Code plugin `speclink-skills` adds a skill button row and a side panel.
- The Herdr mod `speclink` adds a panel and connects the sidebar lists to Speclink.
- The tools have a first entry point: `@speclink/engine` exports one general tool definition, `agentTool`. The model fills only the verb arguments and stdin. The program of the host adds the identity and the scope from its own session, and then calls `host.dispatch`. The model cannot set its own identity. See the [Node SDK](sdk-node.md#agent-tool).
- Not done yet:
  - An MCP adapter. Now you must write the program of a host to add Speclink as a tool.
  - An agent without a checkout cannot use the verbs that need a checkout, for example `drift`.

**The next visible step**

You add Speclink to an agent tool that supports MCP, and you list changes and read specs with tool calls, without writing the program of a host.

## <a id="system"></a>System integration

**The problem**

In a company, Speclink cannot be an island. Accounts must come from the existing identity system. Spec changes must send notifications to other tools. Deployment must follow the existing operations. The specs can also be in several repos.

**Where it is now**

- A host that embeds the engine can connect to its own system today: the accounts, the login, and the permissions belong to the host. To notify other tools, the host sends the notification in the after-commit hook or in its own commit function.
- Operations of the demonstration server work: install, account management, access tokens, device login, and backup and restore all have entry points and end-to-end tests.
- The demonstration server sends an SSE event stream (`/events`), so clients see changes when they happen.
- Not done yet:
  - Enterprise SSO (OIDC) for the demonstration server. Now it has only the accounts that it manages itself.
  - Notifications from the demonstration server to other systems, for example webhooks.
  - Runtime plugins: give the demonstration server another store or custom behavior without a code change.
  - Multi-node deployment. Now the demonstration server supports one instance only.
  - Online backup without a server stop.
  - A spec folder outside the checkout: path rules for `spec_dir`. Now it does not check absolute paths or `../`, and the behavior is not defined. Only `engine.exportScope()` of the Node SDK refuses such a value.
  - A read-only reference to another set of specs.
  - A WebSocket transport (low priority).

**The next visible step**

You log in to the demonstration server with the identity provider of your company, without a separate account. The other items have no visible entry point and no order yet. When they move, [Product status](product-status.md) shows it first.

## <a id="related"></a>Related documents

- [Product status](product-status.md): what is available now, with evidence, check dates, and known gaps.
- [Node SDK](sdk-node.md): the interface of the engine and the host kit, and how to load them.
- [Verb and flag contract](verb-contract.md): where each verb works, the output shapes, and the endpoints.
