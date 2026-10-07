**Verdict: APPROVE WITH FIXES**

I read the code only and did not run any tests or builds. All six round-2 fixes hold, but the Folder display change (finding 1) undoes part of the invisible-character fix, so it should be fixed before handoff.

**Fixes that hold:**
- **Nonce-scoped guards:** `take_confirmed`, `Drop` and `confirm`'s second lookup all check the nonce. The A/B/C case from round 2 is covered by a new test.
- **Closing on take:** `take_confirmed` closes the offer, and both of `confirm`'s lookups check that, so a confirm after the take returns `Ended`.
- **Registration only before a human:** offers are registered only in the post-taint branch, or in `NeedsApproval` when the reviewer is the user. `Skip`, `Guardian` and `Forbidden` no longer take slots. The two branches can't both run, so there is no double registration.
- **`fits()` recheck:** it recomputes against the last rendered `(width, height)`, fails closed before the first render, and requires at least 40 columns. An added error line can only make it refuse.
- **State read outside the lock:** the `Arc` is cloned under the lock and called after it is released. The nonce and closed recheck closes the gap.
- **Escaping:** bidi overrides and isolates and zero-width characters are now escaped in the command, the history line and `/security`.

**Findings:**

1. **Medium (regression introduced this round): the Folder field is shown decoded and unescaped.** `core/src/tools/orchestrator.rs:842`, `tui/src/security/grant_view.rs:144`
   - **Problem:** `shown_cwd` now uses `cwd.to_path_buf().display()`. That decodes the path, whereas the old `PathUri` Display was percent-encoded. `grant_view` prints `offer.cwd` without `escape_controls`. The working folder is model-chosen, so:
     - a newline in the folder name makes `textwrap` start new rows, which lets the model inject fake lines into the review;
     - U+202E and other bidi characters reorder the folder text;
     - paths that are not valid UTF-8 are shown lossily.
   - **Why it matters:** the folder decides what relative paths in the command point to, so the review can no longer claim that what is shown is what runs.
   - **Fix:** apply `escape_controls` to every displayed field (`cwd`, the actor chain, resource and digest), or escape in Core when building the offer. Add a test with `\n` and U+202E in the folder.

2. **Low: the source-scan test lost its positive control and has a new gap.** `tui/src/security/grant_view_tests.rs:~82-105`
   - **No positive control:** `grant_view.rs` is now on the allowed list and the test asserts an empty result. If the detector silently stops matching, the test still passes.
   - **New gap:** removing whitespace turns `security_grant::{confirm as c}` into `confirmasc`, which no longer equals `confirm`. The round-2 version, which split on spaces, caught this.
   - **Fix:** before filtering, assert that the raw matches include `tui/src/security/grant_view.rs`. Inside braces, also match names that start with `confirm` followed by `as` (whitespace is stripped).

3. **Low: `open_test_offer` checks and then registers in two steps, and can still block a real offer.** `core/src/security/grant_offer.rs:506-519`
   - **Race:** `contains_key` and `register` take the lock separately. A real offer registered in between is removed as a "shared id".
   - **Pre-emption:** if a test offer is registered first, the real registration removes it and gets nothing, so the real approval has no grant option.
   - **Impact:** reachable only from in-process code, and the effect is denial of service only.
   - **Fix:** do the vacancy check inside `register` under the same lock (an `only_if_vacant` flag), and gate the function behind `cfg` or a feature when possible.

4. **Low: `principal_label` can give two chains the same label.** `core/src/security/grant_offer.rs:200-205`
   - **Problem:** `(Agent, "x")` and `(Agent, "agent:x")` both become `agent:x`. `confirm` detects a chain change only by comparing these labels (`:322`).
   - **Mitigation:** the epoch and revocation-generation check, binding the grant to the live chain, and the recheck in `apply` cover it today.
   - **Fix:** store the structured `ActorChain` in `Pending` and compare that. Use the labels for display only.

5. **Info: some invisible characters are still not escaped.** `core/src/security/grant_offer.rs:436-448`
   - **Missing:** tag characters (U+E0000–E007F), U+FFF9–FFFB, U+2028/2029, U+034F, variation selectors and Hangul fillers.
   - **Impact:** none of these can fake ASCII shell syntax, so the risk is low.
   - **Fix:** escape every character in categories Cf, Zl and Zp (plus the fillers), or switch to an allowlist of printable characters.

6. **Info: the comment at `orchestrator.rs:172` is inaccurate.**
   - **Problem:** it says the offer is not registered for a hook, but permission-request hooks run inside `resolve_tool_apporval` after registration (`approvals.rs:203-231`). An offer therefore holds a slot while a hook runs, and a hook "Allow" approves with no human asked.
   - **Impact:** harmless, because no review is shown, so nothing can be confirmed. It leaves a small version of round-2 finding 6 (a slot taken before a human is asked).
   - **Fix:** correct the comment, or register the offer after the hooks.