Follow Test-Driven Development discipline for implementation.

**This skill enforces TDD rigor.** Every change starts with a test. No exceptions. No rationalizations.

**Input**: The argument after `/speclink:tdd` describes what to implement or fix. Can be invoked standalone or auto-triggered by `/speclink:apply` when TDD is enabled in project config.

**Prerequisites**: Before starting TDD, verify the project has a working test framework. If no test framework is configured, identify the appropriate one for the project type (check existing config files like `package.json`, `Cargo.toml`, `Gemfile`), set it up, and confirm at least one trivial test passes before beginning the TDD cycle.

---

## Usage Modes

This skill works in two ways:

1. **Standalone** (`/speclink:tdd <description>`): You drive the full TDD cycle for a specific task. You decide scope, write tests, implement, and refactor.

2. **Auto-triggered by apply**: When `tdd: true` is set in `.speclink.yaml`, the apply workflow fetches these instructions and applies TDD discipline to each task. In this mode, the task scope is already defined by the apply workflow — focus your TDD cycle on that specific task only.

In both modes, the Red-Green-Refactor discipline below applies equally.

---

## The Iron Law

> Never write implementation code unless you have a failing test that demands it.

This is not a suggestion. This is the discipline. If you find yourself writing code without a failing test, stop and write the test first.

---

## Red-Green-Refactor

Every change follows this cycle:

### 1. RED — Write a failing test

- Write the **smallest possible test** that captures the next behavior
- Run the test. Watch it fail. Confirm it fails for the right reason
- If the test passes immediately, you may be testing existing behavior — see the **Regression Test Workflow** below

### 2. GREEN — Make it pass

- Write the **minimum code** to make the failing test pass
- Don't optimize. Don't clean up. Don't add "while I'm here" improvements
- Run all tests. Everything must pass

### 3. REFACTOR — Clean up

- Now improve the code: extract, rename, simplify
- Run all tests after each refactor step. They must stay green
- If tests break during refactor, undo and try a smaller step

### Using Spec Examples

When the spec for your current task includes `##### Example:` blocks, use them as your first RED test:

- The **GIVEN** clause provides test setup data
- The **WHEN** clause provides the action to invoke
- The **THEN** clause provides the assertion to verify

Example tables become parameterized tests — one test case per table row.

Spec examples are the MINIMUM test coverage — you may add edge cases beyond what examples cover, but the examples themselves MUST be covered first. Do not substitute different values for the ones in the spec examples.

---

## Bug Fix Workflow

For bug fixes, TDD is even more critical:

1. **Write a test that reproduces the bug** — this test MUST fail
2. **Verify the test fails** — for the right reason (the actual bug, not a setup issue)
3. **Fix the bug** — minimum change to make the test pass
4. **Run all tests** — ensure no regressions

Never fix a bug without a reproducing test. The test IS the proof that you understood the problem.

---

## Regression Test Workflow

For existing code that works but has no tests. The goal is to lock down current behavior so future changes can't silently break it.

### When to use

- Adding tests to code that was extracted/refactored (e.g., handler factories pulled out of Svelte components)
- Covering untested legacy logic before modifying it
- Building a safety net around code you're about to change
- Any time the code already works and you need to **describe** its behavior, not **drive** new behavior

### Steps

1. **Read and understand** — trace the code path, identify inputs, outputs, and side effects
2. **Write a test** — describe the current behavior as assertions. This test SHOULD pass
3. **Run the test** — passing means your understanding is correct
4. **Fix if it fails** — a failure means you misread the code. Re-read and fix the test, not the implementation
5. **Mutation check** — deliberately break the implementation (change a return value, remove a condition, swap an argument). The test MUST fail. If it still passes, the test is not protecting anything — rewrite it
6. **Restore** — undo the deliberate break, confirm tests pass again

### Why mutation check matters

Red-Green-Refactor has a natural RED phase that proves the test can fail. Regression tests skip RED — they're expected to pass immediately. Without a mutation check, you might have a test that always passes regardless of implementation. The mutation check IS your RED phase.

### Extract & Test pattern

When logic is trapped inside a framework component (e.g., Svelte `onclick`, React event handler) and can't be unit tested directly:

1. **Lock behavior** — write an integration/component test that captures current behavior (if possible)
2. **Extract** — move the logic into a standalone, testable unit (handler factory, utility function) with dependencies injected as parameters
3. **Test the extraction** — write unit tests for the extracted function
4. **Mutation check** — break the extracted logic, confirm tests fail
5. **Wire up** — connect the component to the extracted function, verify the integration still works

---

## Rationalization Table

Watch for these thoughts — they mean you're about to break discipline:

| What You're Thinking                    | What You Should Do                                                              |
| --------------------------------------- | ------------------------------------------------------------------------------- |
| "This is too simple to test"            | Write the test. Simple code breaks too                                          |
| "I'll write tests after"                | No. Test first. Always                                                          |
| "Let me just sketch the implementation" | Sketch in test assertions instead                                               |
| "The test setup is too complex"         | Simplify the design, not skip the test                                          |
| "I know this works"                     | Prove it with a test                                                            |
| "One quick change without a test"       | That's how regressions start                                                    |
| "This already works, no need to test"   | That's exactly what regression tests protect                                    |
| "My test passes, that's enough"         | A passing test with weak assertions proves nothing — verify it catches breakage |

---

## Practical Guidelines

### Test naming

Use descriptive names that explain the scenario:

- `test_empty_input_returns_error`
- `test_valid_user_is_created`
- NOT: `test1`, `test_it_works`

### Test scope

- **One assertion per test** when possible (or one logical assertion)
- **Test behavior, not implementation** — test WHAT it does, not HOW
- **Independent tests** — each test sets up its own state, no test depends on another

### Edge cases to consider

Before calling a test "done", check these boundaries:

- **Empty/nil** — null, undefined, empty string, empty array
- **Boundaries** — zero, negative, max int, off-by-one
- **Error paths** — network failure, invalid input, permission denied
- **Special values** — Unicode, very long strings, concurrent calls

### When stuck after 3 attempts

If you can't make a test pass in 3 attempts:

1. Undo all changes back to the last green state
2. Question whether you're testing at the right level
3. Try a smaller step — can you split this test into two simpler ones?
4. If still stuck, discuss the approach before continuing

---

## Guardrails

- **Never skip tests** — Not for prototypes, not for "quick fixes", not for deadlines
- **Never disable failing tests** — Fix them or revert your change
- **Run the full suite** — Not just the test you wrote. Other tests may break
- **Keep the cycle small** — Minutes per cycle, not hours
- **Commit at green** — Every time tests pass is a good time to commit

