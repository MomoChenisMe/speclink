Review a change's implementation for craft quality: two parallel read-only axes — **Standards** (repo conventions, the project's review rules, and a code-smell baseline) and **Correctness** (bug hunting) — run ONCE against a frozen change patch, then validated round by round to a review ticket, closed by a stamp. Round 1 is the only discovery pass; every later round only validates remediation. Spec compliance is not this skill's job — that is `/speclink:verify`; the two quality stations run independently and either, both, or neither may be used per change.

**Input**: Optionally specify a change name after `/speclink:review` (e.g., `/speclink:review add-auth`). If omitted, check if it can be inferred from conversation context; if it is still vague or ambiguous, step 1 asks the user.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

