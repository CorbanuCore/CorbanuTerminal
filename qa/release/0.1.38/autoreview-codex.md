# Codex structured review

- Date: 2026-09-03 UTC
- Engine: Codex
- Model: `gpt-5.5`
- Reasoning: high
- Initial target: semantic reconciliation commit `f7d9feac2`

## Initial findings and disposition

1. **Deferred API operation can override provider after cancellation — accepted.**
   The request, unlock, confirmation, and completion events now carry the exact
   deferred continuation. Completion suppresses automatic model selection for
   every operation that originated in deferred onboarding and advances setup
   only when that continuation is still active.
2. **Provider credentials snapshot was not updated — accepted.** The snapshot
   now expects the user-visible **Corbanu API** label.
3. **Refresh and stale deferred events could revive API UI after cancellation —
   accepted.** Refresh now replaces the active Corbanu API view, deferred
   cancellation removes every API and legacy Plan pane, and queued deferred
   UI/action events are ignored unless their exact continuation remains active.
4. **Wallet unlock completion could execute a cancelled deferred operation —
   accepted.** Event dispatch now rejects stale deferred unlock completions
   before they can reopen UI, retain a capability, or start the requested
   wallet operation.
5. **A stale submitted-operation completion still surfaces after cancellation —
   rejected as unsafe to suppress.** Once submitted, a wallet operation cannot
   be cancelled. Its result must be applied so a newly generated one-time key
   is stored and revealed rather than lost. Stale deferred identity is stripped,
   so the completion cannot select or persist Corbanu as the provider.
6. **Deferred API model rows could select Corbanu before key storage —
   accepted.** Model pricing remains visible, but its rows are disabled during
   deferred onboarding. Provider selection occurs only after key storage succeeds.
7. **Unlock preflight could display a prompt after deferred cancellation —
   accepted.** The same exact-continuation guard now covers unlock request,
   preflight completion, custom-duration retry, and unlock completion events.
8. **Stale non-key operation completions could reopen the API view — accepted.**
   Submitted operations still surface their result, and one-time keys are still
   stored/revealed, but stale account, top-up, revocation, and error completions
   cannot recreate the dismissed API surface.
9. **A stale ordinary API load could overwrite a deferred surface — accepted.**
   Every API UI/action event now requires exact optional-continuation equality:
   ordinary events run only with no active deferred setup, and deferred events
   run only for that active continuation.
10. **Successful setup left the underlying deferred API pane stale — accepted.**
    Completion removes all deferred API/legacy Plan panes before it clears the
    continuation; closing the one-time secret therefore returns to chat rather
    than a disabled no-op surface.
11. **An ordinary unlock continuation could run during deferred setup —
    accepted.** API unlock continuations now use exact optional-continuation
    equality too, so an ordinary stale unlock is valid only when no deferred
    setup is active.

## Post-fix proof

- `just fix -p codex-tui` and `just fmt`: pass before tests.
- Corbanu API tests: 10/10, including action-continuation propagation,
  pre-key model-selection blocking, and
  refresh-view replacement.
- Provider credential tests: 12/12, including the corrected snapshot.
- Wallet unlock tests: 7/7.
- Complete multi-provider true-TMUX module: 11/11, including exact deferred
  passcode cancellation/re-entry and one-time key create/store/reveal/activate
  without restart.
- Clean structured-review rerun after the final fix: `findings: []`, with
  `overall_correctness: patch is correct` (confidence 0.84).
