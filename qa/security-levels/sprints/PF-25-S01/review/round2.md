**Verdict: APPROVE WITH FIXES**

All nine round-1 findings are fixed or accepted as documented. In my reading, a grant now applies only after its own approval is approved, under the same epoch and revocation generation, actor chain and operation digest. "1 run" never enters the ledger. A declined, cancelled or abandoned approval drops the offer and the choice. The review opens on Back, so a double or held Enter cannot grant. In the overlay, the request is checked before `confirm`, and `confirm` runs before Accept is sent, so the choice is always recorded before Core can resolve the approval.

This was a read-only review: I read the code and diffs and ran no tests or builds. I found nothing above Low. None of the issues below lets a model, tool or another client cause a grant the human did not confirm.

1. **Low: a guard can take or remove a later offer with the same key.** `core/src/security/grant_offer.rs:237-262`, `:140-151`
   - **Problem:** A registers K. B registers K, which removes A's offer and gives B no guard. C then registers K and gets a fresh entry. A's guard still holds K, so A's `take_confirmed` takes C's choice and A's drop removes C's offer. `apply` checks the operation digest, so the worst case is the same exact command, but the guard can still act on an offer it does not own.
   - **Fix:** Store a unique nonce (an incrementing `u64`) in `Pending` and in the guard. `take_confirmed` and `Drop` should act only when the nonce matches. Alternatively, keep a tombstone for K until every guard for K has dropped.

2. **Low: the test-only offer API ships in production.** `grant_offer.rs:436-470`
   - **Problem:** `open_test_offer` is `pub` and not gated by `cfg`. Any in-process caller can delete a real offer, since registering on an existing key removes it. Combined with finding 1, it can also insert a fake entry under a real guard's key. `apply` rejects that entry because the `"test"` digest prefix and the epoch 0 / chain don't match, so this is defence in depth, not a live grant path.
   - **Fix:** Gate it behind `#[cfg(any(test, feature = "test-support"))]` and turn the feature on only for the TUI's dev-dependency, or have it use a separate map.

3. **Low: the "clipped" flag can be stale, and the check only counts rows.** `tui/src/security/grant_view.rs:61-79`, `:117`
   - **Problem:** `clipped` is set at render time. If `u` (which makes the text longer), an arrow and Enter arrive in one input batch, the check uses the old layout. An error line added after a failed confirm also changes the height. Separately, `field` wraps at `width.max(22)`, so on a pane narrower than 22 columns the lines are cut off sideways without being detected.
   - **Fix:** Store the last rendered `(width, height)` instead of a bool. In `confirm`, recompute `lines(width).len() <= height` and require `width >= 22`.

4. **Low: invisible formatting characters are not escaped.** `grant_offer.rs:418-428`
   - **Problem:** `char::is_control` only matches category Cc. Bidi overrides and isolates (U+202A–202E, U+2066–2069) and zero-width characters pass through into the review, the history cell and `/security` labels. Terminals that support bidi can then display the command differently from what will run.
   - **Fix:** Also escape category Cf, or at least the bidi and zero-width ranges. Add a test.

5. **Low: the TUI can say "You granted" when nothing applied.** `approval_overlay.rs:835-865`, `grant_offer.rs:301-312`
   - **Problem:** If another client approves first, the orchestrator may call `take_confirmed` (getting `None`) before its guard drops. `confirm` still succeeds in that window, so the TUI shows "You granted…" while the command runs with the protected-path rules. This fails closed, so it is safe, but the message is untrue. It also covers the case where `apply` later refuses the grant.
   - **Fix:** Have `take_confirmed` mark the entry as closed so a later `confirm` returns `Ended`. Change the history wording to "requested", or record the outcome when `apply` runs.

6. **Low: offers are registered before Core knows a human will be asked.** `core/src/tools/orchestrator.rs:171-172`
   - **Problem:** Calls that end up as `Skip`, Guardian-reviewed or `Forbidden` still take an offer slot during the `post_taint_action` await. Eight parallel calls like that use up `MAX_OFFERS_PER_THREAD`, and a real approval then gets no grant option. The effect is a denial of service only.
   - **Fix:** Register only in the branches that call `resolve_tool_apporval` with `ApprovalReviewer::User`, just before the call.

7. **Low: a session callback runs under the global offers lock.** `grant_offer.rs:285`
   - **Problem:** `(pending.current)()` upgrades the session and reads the policy state while OFFERS is held. If that temporary `Arc` is the last one, the session's `Drop` runs under a global lock. This is not deadlockable today, but it is fragile.
   - **Fix:** Make `CurrentState` an `Arc<dyn Fn…>`, clone it under the lock, and call it after releasing the lock. The second lookup on line 303 already re-validates the offer.

8. **Low: the source-scan test is easy to get around and probably breaks under Bazel.** `tui/src/security/grant_view_tests.rs:40-87`
   - **Problem:** The scan misses `use …::security_grant as g; g::confirm`, `security_grant::*`, and tab separators inside braces. It also excludes any product file named `*_tests.rs`.
   - **Problem, continued:** It locates the workspace with `env!("CARGO_MANIFEST_DIR")`. Under Bazel, `test_data_extra` includes only `tui/src/**`. The walk then either fails (empty result, so the test goes red) or passes without scanning any other crate. Other tests in this crate use `codex_utils_cargo_bin::find_resource!`.
   - **Fix:** Also flag any `security_grant` or `grant_offer` import that leaves `confirm` unnamed (glob, alias or module import), and resolve the path through `find_resource!`. If Bazel cannot see the workspace, skip the test there explicitly.
   - **Better:** Replace the scan with a type the TUI must pass in, which `confirm` requires.

9. **Info: the end-to-end Core test runs only on macOS.** `core/tests/suite/pf_25_s01.rs` is `#![cfg(target_os = "macos")]`. The guarantee that the offer key matches the approval id and that the digest matches at run time is therefore not exercised on Linux CI.
   - **Fix:** Add a check that does not depend on the platform (key and digest match only, without the canary file read), or record this as a known coverage gap.

Accepted limit (round-1 finding 6): `/security` still lists other sessions' and ended sessions' grants, and can list an entry made stale by a commit that lands between `check` and `push` in `issue_labelled`. `admit` still blocks these, so this is a display-only issue.