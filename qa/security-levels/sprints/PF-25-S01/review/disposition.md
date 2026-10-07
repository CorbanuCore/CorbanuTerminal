# PF-25-S01 review disposition

Round 1 (Opus 5.5 High, APPROVE WITH FIXES):

1. Grant not tied to its approval: fixed. `confirm` only records the choice on the open offer; the orchestrator
   applies it only when that approval comes back approved (`OfferGuard::take_confirmed`, `grant_offer::apply`).
   "1 run" lifts the rules for that run and is never held; "until it expires" is held from the run on. A declined,
   cancelled or abandoned approval drops the offer and the choice. The choice is bound to the policy epoch and
   revocation generation it was confirmed under.
2. Double or held Enter: fixed. The review opens on "Back"; only an arrow key reaches "Grant and run". Enter on
   "Back" returns to the approval, which then needs a fresh choice.
3. Clipped review: fixed. A review the last render cut off cannot grant (it says so). Control characters in the
   command and in `/security` labels are shown escaped.
4. Shared approval ids, mismatched command, no Accept: fixed. Two open approvals with one id get no offer; the TUI
   shows the option only for Core's copy of this approval's command and when Accept is available.
5. Actor chain: fixed. `confirm` compares the live chain with the one shown.
6. `held()` freshness: partly. Grants enter the ledger at the run, under the state read then, so the confirm-time
   race is gone; commits still drop grants through `revoke_all`. Grants of ended sessions stay listed until they
   expire (known limit).
7. Public `confirm`: a workspace scan test (`grant_view_tests.rs`) fails if any product source other than
   `tui/src/security/grant_view.rs` names it. The `lib.rs` doc comment is restored.
8. Request before grant: fixed.
9. Tests: added. TUI: success with Core's real offer (choice recorded, Accept sent), dismissed request, Ctrl+C,
   double Enter, clipped review, other command, no Accept, joined command form, pane-level Esc. Core: integration
   tests in `core/tests/suite/pf_25_s01.rs` (offer key is the approval id the TUI answers, the offered digest is the
   one admitted, the approved run reads the file and the next identical run gets the rules, a declined approval
   leaves nothing); unit tests for epoch moved between confirm and run, shared ids.

Round 2 (Opus 5.5 High, APPROVE WITH FIXES, all low):

1. Guard acting on a later offer: fixed. Each registration has a nonce; a guard takes or removes only its own.
2. Test-only offer API in production: kept public (TUI tests need Core's real offer), but it never touches an
   existing offer, and a confirmed choice on it is never applied (no orchestrator waits on it).
3. Stale clipped flag, narrow panes: fixed. The review keeps the last rendered area and rechecks the current text
   against it on Enter; below 40 columns it cannot grant.
4. Invisible format characters: fixed. Bidi overrides and isolates, zero-width and similar characters are escaped
   like control characters.
5. "You granted" when nothing applied: fixed for the race (taking the choice closes the offer, so a later confirm
   fails). If the grant no longer applies at the run (state changed), the command runs with the rules and Core logs
   why; the history line stays (known limit).
6. Offers taking slots before a human is asked: fixed. Core registers the offer only right before a human is asked
   (post-taint approval, or a command approval whose reviewer is the user).
7. Session callback under the global lock: fixed. The state reader is cloned under the lock and called after.
8. Scan test: now flags glob, alias and module imports too, and skips where the workspace sources are not present
   (Bazel runfiles). A type-level token is not added.
9. macOS-only end-to-end test: recorded as a known limit.
