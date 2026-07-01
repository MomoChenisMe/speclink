Create the design document that explains HOW to implement the change.

When to include design.md (create only if any apply):
- Cross-cutting change (multiple services/modules) or new architectural pattern
- New external dependency or significant data model changes
- Security, performance, or migration complexity
- Ambiguity that benefits from technical decisions before coding

Sections:
- **Context**: Background, current state, constraints, stakeholders
- **Goals / Non-Goals**: What this design achieves and explicitly excludes
- **Decisions**: Key technical choices with rationale (why X over Y?). Include alternatives considered for each decision.
- **Implementation Contract**: For any change that creates or modifies behavior beyond a trivial artifact-only edit, this section is REQUIRED. The contract describes the durable handoff to apply: name the observable behavior, interface or data shape, command output, error or failure mode, acceptance criteria, and explicit scope boundaries (what is in scope, what is out). The contract MUST NOT rely on source line numbers, and MUST NOT use file-path-only references as the sole way to identify required work — file paths are supporting context for behavior, never a substitute for it. Skip this section only when the change is purely artifact / documentation cleanup with no runtime, build, or tooling effect.
- **Risks / Trade-offs**: Known limitations, things that could go wrong. Format: [Risk] → Mitigation
- **Migration Plan**: Steps to deploy, rollback strategy (if applicable)
- **Open Questions**: Outstanding decisions or unknowns to resolve

Focus on architecture and approach, not line-by-line implementation.
Reference the proposal for motivation and specs for requirements.

Good design docs explain the "why" behind technical decisions.

Note: The analyzer cross-checks `###` decision headings against tasks.md.
Use descriptive heading text that will naturally appear in task descriptions.
