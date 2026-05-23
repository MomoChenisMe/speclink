# Instructions: design

Create the design document that explains **HOW** to implement the change.

## When to create design.md

Create design.md when **any** of the following apply:

- Cross-cutting change touching multiple services or modules
- New architectural pattern, abstraction, or seam
- New external dependency or significant data model changes
- Security, performance, or migration complexity
- Ambiguity that benefits from technical decisions before coding

Trivial artifact-only edits with no runtime / build / tooling effect MAY skip design.md. When skipped, write Non-Goals into proposal.md so rejected approaches stay recorded.

## What to write

Follow the template's sections in order:

- **Context** — background, current state, constraints, what already exists.
- **Goals / Non-Goals** — what this design achieves and what it explicitly excludes.
- **Decisions** — for each key technical choice, write `### Decision: <name>` with a **Why** rationale and an **Alternatives** list. Each alternative names what was considered AND why it was rejected.
- **Implementation Contract** — REQUIRED for behavior-changing work. The contract is the durable handoff to apply: it names the observable behavior, interface or data shape (commands, function signatures, JSON envelopes, file formats), failure modes (error shapes, exit codes, fallback semantics), acceptance criteria (tests, CLI invocations, analyzer checks), and explicit scope boundaries (in-scope vs out-of-scope). The contract MUST NOT rely on source line numbers, and MUST NOT use file-path-only references as the sole way to identify required work.
- **Risks / Trade-offs** — known risks with mitigation; trade-offs accepted and why.
- **Migration Plan** — only when deployment / rollback steps are non-obvious.
- **Open Questions** — outstanding decisions or unknowns. If any block apply, name them.

## Rules

- Every technical decision SHALL include at least one alternative considered. A decision without alternatives is a default, not a decision.
- File paths SHALL be relative to the project root (e.g., `crates/runtime/src/instructions_ops.rs`). Path fragments without anchor (`parser/mod.rs`) fail preflight.
- Reference proposal capabilities and spec requirements by name. Do NOT introduce new behavior not present in the proposal / spec — that is scope creep.
- The analyzer cross-checks `###` decision headings against tasks. Use descriptive heading text that will naturally appear in task descriptions.

## What NOT to write

- Don't describe line-by-line implementation — that is the apply phase's job.
- Don't paste large existing code blocks. Reference them by file + function name.
- Don't write decisions whose alternatives are straw men. If only one option is real, say "no realistic alternative" and explain why.
- Don't leave placeholder language (TBD, TODO, "needs investigation") in committed sections. If you genuinely don't know, mark it under Open Questions.
