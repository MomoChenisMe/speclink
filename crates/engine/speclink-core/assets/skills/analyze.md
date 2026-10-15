Analyze artifact consistency for a change. Can be invoked directly or triggered automatically when all artifacts are complete.

**Read-only**: this skill reads artifacts and reports; it never modifies files. Keep the output concise — it runs inline, not as a separate workflow.

**Input**: Optionally specify a change name (e.g., `/speclink:analyze add-auth`). If omitted, infer from conversation context or auto-select if only one active change exists.

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**Steps**

1. **Determine change name**

   If not provided, infer from context or run `speclink list --json` to auto-select. Do not prompt for change selection when it can be inferred.

2. **Run programmatic analysis**

   ```bash
   speclink analyze <change-name> --json
   ```

   This returns structured JSON with:
   - `dimensions`: Array of `{ dimension, status, finding_count }` for Coverage, Consistency, Ambiguity, Gaps
   - `findings`: Array of `{ id, dimension, severity, location, summary, recommendation }`
   - `artifacts_analyzed` / `artifacts_missing`: Which artifacts were available

3. **Present results**

   **Report language**: run `speclink instructions apply --change "<name>" --json` and use its `locale` field (e.g., "Traditional Chinese (繁體中文)") — write the report in that language, prose, headings, and table labels included. Keep severity labels (Critical/Warning/Suggestion), command lines, and code references in English. If the field is absent or the call fails, write in English.

   Format the JSON output as a readable summary:

   ```
   ## Artifact Analysis: <change-name>

   | Dimension     | Status                   |
   |---------------|--------------------------|
   | Coverage      | <status>                 |
   | Consistency   | <status>                 |
   | Ambiguity     | <status>                 |
   | Gaps          | <status>                 |
   ```

   Group findings by severity (Critical > Warning > Suggestion) with locations and recommendations.

4. **Supplement with AI semantic analysis** (optional)

   The programmatic analyzer catches structural issues. For deeper semantic analysis, also read the artifacts and check for:
   - Design decisions that contradict spec requirements
   - Tasks referencing work outside proposal scope
   - Risks in design without corresponding spec coverage
   - Logical inconsistencies between artifacts

   Add any additional findings to the report.

5. **Recommend next steps**
   - Critical findings → "Found N issue(s) worth addressing. Want to fix these before implementing?" (ask with the **AskUserQuestion tool**, or as plain text and wait for the user's response when that tool is not available)
   - Only Warning/Suggestion findings → note them briefly, then recommend proceeding with `/speclink:apply`
   - Clean → "Artifacts look consistent" and suggest `/speclink:apply`

**Passive Trigger**

When `speclink status --change "<name>" --json` shows `isComplete: true`, run this analysis automatically before recommending `/speclink:apply`.

**Guardrails**

Check these before you report:

- [ ] No file was modified ("Read-only").
- [ ] The change name was inferred when possible, not asked for (step 1).
- [ ] The report is in the `locale` language, grouped by severity (step 3).
