**Verdict: REQUEST CHANGES.** Two bugs can show a green "Protected" badge when protection isn't fully in place, which breaks decision (4). Everything else is should-fix or nit.

This is a read-only review of `git diff origin/main...HEAD` at commits `91d34a4347` and `3c352e9c75`. I did not build anything or run any tests.

1. **`tui/src/security/inspector.rs:706-717`, 749 — blocking. The badge can claim Aggressive while Core enforces less.**
   - `in_force()` takes the higher of the session's launch level and Core's level. So if this run started Aggressive but Core is still at Permissive or Moderate, the badge comes out `Protected("Aggressive")`. That can happen: the picker already checks for a Core level that lags behind a saved Aggressive, in `core_behind()` / `core_commit_needed`.
   - The live-policy row even says "Permissive in force" while the badge is green.
   - Under Core Permissive, `ModelClient::post_taint_state()` returns `None`, so the post-taint gates do nothing. Yet the "Untrusted content" row still says protected actions need your fresh approval.
   - **Fix:** when the policy is live and Core's level (or the root agent's level) is below the launch level, add a degraded reason such as "Core enforces X; Aggressive's protected-action gates are not active". Also make the taint row respect Core's level: below Moderate it should read off or degraded, not "on". Add a test with launch level Aggressive and a live Core tree at Permissive.

2. **`tui/src/bottom_pane/security_view.rs:235-243` and `security_inspector.rs:91-137` — blocking. Stale facts can stay green on screen.**
   - Staleness is only checked when the pane redraws. While the inspector is open, `next_frame_delay` returns `None` unless a save is in progress. In an idle session nothing redraws, so a green badge stays up long after 30 seconds.
   - **Fix:** while the inspector is open, return roughly `STALE_AFTER_SECONDS - age + 1` from `next_frame_delay`, or a 1-second tick so the age counter updates too. Add a test that the inspector requests a frame.

3. **`inspector.rs:592-675` and `security_inspector.rs:107` — should-fix. The yellow "partial" badge claims enforcement it hasn't seen.**
   - With no live tree (stored state or a remote app server), the badge reads "◐ Aggressive enforced; …" even though nothing was observed.
   - **Fix:** if the policy isn't live, show "Aggressive configured (not observed here)", or make that case degraded.

4. **`core/src/security/inspection.rs:229` and `inspector.rs:667-674` — should-fix. A poisoned lock shows as a huge taint count.**
   - When the untrusted-content registry's lock is poisoned, `Generation(u64::MAX)` is displayed as "session tainted (18446744073709551615 batches)" with an "on" tag.
   - **Fix:** add a separate `TaintFacts::Poisoned` and show it as degraded with the text "registry unreadable; treated as tainted".

5. **`inspection.rs:191-200` — should-fix. The model key broker row reports installation, not health.**
   - `model_key_broker_installed()` only checks that a broker object was registered at startup. It doesn't check that the broker process is alive, yet the row says "[on] provider keys are brokered" and labels itself "observed".
   - **Fix:** reword to "broker installed; health not probed" and mark it unobserved, or probe it.

6. **`inspection.rs:159-166` — nit. Grants for a stopped child agent are still listed as "[on]".**
   - Those grants can't be used because the agent is denied everything.
   - **Fix:** skip them, or mark them suspended.

7. **`core/src/security/inspection_tests.rs:262-275` — should-fix. The denial test is order-sensitive on shared global state.**
   - `starts_with` assumes the newest launch denial with no thread is this test's own. `launch_contract_tests.rs` records such denials into the same global list, so under plain `cargo test` threads (not nextest's process-per-test) it can fail at random. The 32-entry cap can also push entries out.
   - **Fix:** assert that each expected entry is present rather than in a fixed order, or use `#[serial]`.

8. **`inspector.rs:684-695` — nit. Preflight blocker text includes file paths.**
   - The "not clean" and "unverified" rows show raw blocker text, for example "agent commands can still read <path>". These are preflight findings rather than denials, but they're shown in a panel meant to be secret- and path-free.
   - **Fix:** use `Boundary::summary()` here and leave the details to the preflight screen.

9. **`core/src/tools/orchestrator.rs:214-221` — nit. Interrupted approvals are logged as "declined by you".**
   - Any error from `resolve_tool_apporval`, including an aborted turn, is recorded as a human decline.
   - **Fix:** record aborts as "declined by you (turn interrupted)", the same text the drop guard uses.

10. **`inspection.rs:286-300` — nit. Launch denials have no thread attached.**
    - They appear in every session's inspector in the same process, which matters for an app server hosting several sessions.
    - **Fix:** pass the thread through where callers have it, or note "this process" in the row (it already does). Acceptable as is.

**No problems found in these areas:**
- **Locking:** INGRESS, DENIALS, LEDGER and the policy-tree lock are never held at the same time on the inspector's read path, and DENIALS takes no other lock. No deadlock risk.
- **The drop guard (`PendingProtectedAction`):** every path that creates `HumanCheck` either asks the person and calls `resolve`, or drops it only on interruption. No spurious "declined" entries.
- **Read-only:** the inspector's only keys are Esc, `r` (refresh) and scrolling. Nothing writes state; the test checks the Corbanu home stays empty.
- **Flag-off:** the inspector is only attached when the picker exists, and the new recording hooks only fire on paths that are already gated.
- **Required labels:** nested-agent refuse/pass, the "not contained" rows and the P1 "not available" rows match decisions 1–3.
- **Repository conventions:** `/*param*/` comments and `pretty_assertions` are used as required.