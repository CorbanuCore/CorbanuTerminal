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
