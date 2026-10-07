**Verdict: APPROVE WITH FIXES**

I found no way for a model, tool, app-server request or project content to create an offer or reach `confirm`. The only caller is the review's Enter, and `confirm` checks the offer against Core's own copy. Offers in the TUI are read only from the in-process static, so a remote app-server gets no grant option. The problems below are in how the grant is tied to the approval, how Enter is handled in the review, and how the review is displayed.

1. **Medium: the grant is not tied to the approval that asked for it.** `core/src/security/grant_offer.rs:236-291`, `tui/src/bottom_pane/approval_overlay.rs:819-845`
   - **Problem:** `confirm` adds the grant to the ledger while the approval is still open, and the TUI sends `Accept` afterwards. The offers lock is released before `issue_labelled` runs, so the approval can end between the check and the issue. Other ways to get an orphaned grant are a turn interrupt, a deny from another client, or the `Accept` never arriving.
   - **Problem, continued:** `admit` matches only thread and digest. A "once" grant can therefore be used by a different run of the same command: a parallel call with the same command, folder and permissions, or a later run that needs no approval (requirement `Skip`). That run would lift the protected-path rules without the human approving it.
   - **Fix:** `confirm` should only record the human's choice (uses, plus the offer it confirmed) on the `Pending` entry. The orchestrator should issue or apply the grant itself, and only when `resolve_tool_apporval` returns approved for this `call_id`, before dropping `OfferGuard`. A "once" grant should then lift the rules for this run directly and never go into the ledger. If you keep the current design, `OfferGuard::drop` should revoke any grant issued from that offer when the approval was not accepted.

2. **Medium: two Enters, or a held Enter, grant and run without the review being read.** `approval_overlay.rs:799-805`, `:883-886`
   - **Problem:** the review bypasses `TypingGuard`. In terminals without keyboard enhancement, a held key arrives as repeated `Press` events, so the `kind != Press` filter does not help. Enter on the grant option opens the review, and the next Enter immediately confirms the grant and approves the command.
   - **Problem, continued:** the guard's own stated rule ("a double or held Enter cannot answer a request the user has not seen") is broken by this screen.
   - **Fix:** open the review in a "fresh choice" state. Enter should do nothing until a deliberate non-Enter key is pressed (for example, an arrow key to highlight a "Grant" row, or a dedicated key). Add a test sending Enter, Enter and checking that nothing is granted.

3. **Medium: the review can be cut off while Enter still grants.** `approval_overlay.rs:951-964`, `tui/src/security/grant_view.rs` (Command field)
   - **Problem:** for `bash -lc`, `strip_bash_lc_and_escape` returns the raw script. A long or multi-line script (harmless first line, many blank lines, then something else) is clipped by `Paragraph` to the pane height. The rest of the command and the footer disappear, but Enter still grants for the whole script.
   - **Fix:** if the lines are taller than the available area, show a truncation marker and block Enter until the person scrolls to the end. Alternatively, show the command escaped onto a bounded number of lines, with a "N more lines" note and a way to expand it.
   - **Fix, continued:** escape control characters. Also escape the `/security` label (`grant_view.rs:164`): the `shlex` join keeps raw newlines inside quotes.

4. **Low: offers with the same ID can overwrite each other, and the TUI does not check the offer matches the request.** `grant_offer.rs:213-221`, `approval_overlay.rs:1242-1248`
   - **Problem:** `register` replaces an existing `(thread, call_id)` offer. Non-OpenAI providers can produce duplicate call IDs for parallel calls. The review could then show command B under the approval header for command A, and A's guard would remove B's offer.
   - **Problem, continued:** the grant option also appears when `Accept` is not in `available_decisions`, but the review always sends `Accept`.
   - **Fix:** if the key already exists, remove it and offer nothing for either call. In `grant_offer()`, require `offer.command == request.command` and that `Accept` is in `request.available_decisions`.

5. **Low: the grant uses the session's current actor chain, not the one shown.** `grant_offer.rs:247-256`
   - **Problem:** `issuer` and the grant's actor chain come from the live state, which is not compared with `shown.actor_chain`. They only line up because the epoch is unchanged.
   - **Fix:** build the labels from the live chain and return `GrantError::Changed` if they differ from the offer.

6. **Low: `/security` can list grants that no longer work.** `aggressive.rs:323`
   - **Problem:** `held()` does not check epoch, revocation generation or the kill switch. It relies on `revoke_all` having run when the change was committed. A grant pushed after that commit (the race in finding 1), or a kill switch propagated without `adopt`, still shows as held. It also lists grants from ended sessions and every other session in the process.
   - **Fix:** in `held()`, keep only entries that are live against each thread's current policy snapshot, or prune when state changes. Show only the current session tree's grants.

7. **Low: issuing a grant relies on convention, not the type system.** `core/src/lib.rs:90-91`
   - **Problem:** `codex_core::security_grant::{offer, confirm}` are `pub`. Any in-process crate (app-server handlers, the MCP server) could mint a grant from an open offer.
   - **Fix:** require a token type that only the TUI can construct, or at least add a test or lint that fails if `security_grant::confirm` is referenced outside `tui/src/security/grant_view.rs`.
   - **Also:** the moved doc comment "PF-29-S01 protected-mode inventory…" now sits on the `security_grant` re-export. Restore it to `security_level_change` / `protected_preflight` and give `security_grant` its own comment.

8. **Low: the TUI can issue a grant and then fail to approve.** `approval_overlay.rs:819-825`
   - **Problem:** the grant is issued before checking that `current_request` is still an `Exec`. If it is not, the grant is left behind with no `Accept` sent.
   - **Fix:** resolve the request first, then call `review.confirm()`.

9. **Medium: tests are missing.**
   - **TUI:** nothing covers a successful confirm. Add a test-only `register` that sends `Accept` and checks that the history cell appears and the queue advances. Also missing:
     - `dismiss_app_server_request` while the review is open (it should close the review);
     - Ctrl+C in the review (it should send `Cancel`);
     - double or held Enter;
     - a clipped or long command.
   - **Core:**
     - no orchestrator-level test shows the offer key matches the TUI request ID (`call_id` with `approval_id = None`, and no offer for network or escalation approvals that carry an `approval_id`);
     - no orchestrator-level test shows the offered digest is the one `post_taint_read_denials` admits;
     - nothing checks that a declined approval or a dropped future leaves no usable grant;
     - nothing covers `confirm` racing with the guard drop;
     - nothing covers two concurrent calls with the same digest under a "once" grant.

**Behaviour with `security_levels` off, or below Aggressive:** nothing changes for the user. The grant option needs both the feature flag and an Aggressive offer. Core still builds the grant operation (including copies of the command and folder) before approval and registers or drops offers under Aggressive regardless of the flag. That has no effect beyond the cost of the copies.

Moving `aggressive_grant_operation` before approval is safe: if the level rises to Aggressive during the approval, nothing is admitted and the rules still apply.