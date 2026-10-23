**APPROVE WITH FIXES.** None of these block. All 10 round-1 findings are resolved and I found no regressions.

The fixes check out:
- **Finding 1:** The badge now uses the root agent's Core level. That never overstates protection, because the actual gating level is `max(ingress_level, root snapshot level)`.
- **Finding 2:** The 1-second tick reaches `schedule_active_view_frame`, and each redraw recomputes `now`.
- **Finding 4:** `Unreadable` takes priority over the Permissive guard.
- **Finding 6:** Grants of stopped agents are no longer listed.
- **Finding 7:** The denial test now checks each entry is present, not its position.
- **Finding 8:** The boundary row uses `Boundary::summary()`, which contains no paths.
- **Rename:** Renaming to `held_for_inspector` removes the clash with the `held()` helper in `aggressive_tests.rs`.

This was a read-only review; I did not build or run anything.

1. **`tui/src/security/inspector.rs:799-804` — low. The new degraded reason is wrong when Core is Moderate.**
   - With launch Aggressive and Core Moderate, it says "Aggressive's protected-action gates are not active". But under Moderate the post-taint gates do run (`client.rs:1044`); what is missing is Aggressive's grant and protected-path rules.
   - **Fix:** choose the text by level. For Permissive keep the current text. For Moderate use something like "Core enforces Moderate; Aggressive's grant and protected-path rules are not active".

2. **`inspector.rs:593` and `core_level` at 732-738 — nit. The taint row can understate protection.**
   - Core actually gates at `self.ingress_level.max(snapshot.level)` (`client.rs:880-891`), where the ingress level comes from `config.security_level` (`session.rs:1221`).
   - A commit that lowers the tree to Permissive while the config level stays Moderate or higher leaves the gates running, but the row says "Core is Permissive: protected actions are not gated". The error is on the safe side.
   - **Fix:** `let core_protected = input.configured.max(core_level(facts)) != SecurityLevel::Permissive;`

3. **`core/src/tools/orchestrator.rs:213-221` — low. The finding 9 disposition misses one case.**
   - An `Err` is not always a human decision. A permission-hook `Deny` is still honoured on the fresh-human-authority path (`tools/approvals.rs:223-230`).
   - That hook refusal goes through `post_taint_outcome(.., "declined")` and shows as "declined by you" in the inspector.
   - **Fix:** have `resolve_tool_apporval` return its `ApprovalResolutionSource`, or record from the hook branch. Map a hook refusal to its own outcome, for example `"refused_hook"` shown as "refused by a permission hook".

4. **Tests — nit. Fixes 5, 6 and 8 have no tests.**
   - **Fix 8:** in `inspector_tests.rs:196-222`, assert the row reads `"protected boundary not clean: 1 blocker; details in the Aggressive review"` and does not contain `OPENAI_API_KEY`.
   - **Fix 5:** with `model_broker: ControlFacts::Enforcing`, assert the row is `State::Unobserved` and the badge is still `Protected`.
   - **Fix 6:** in `core/src/security/inspection_tests.rs`, give a stopped child a grant and assert it is not listed.

5. **`inspector.rs:630-635` — nit. The model key broker row contradicts itself.**
   - It is tagged unobserved but its source says "observed: this process".
   - **Fix:** change the source to "installed: this process".