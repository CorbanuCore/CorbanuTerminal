# Fable 5.1 Max review

- Date: 2026-09-03 UTC
- Harness: TMUX + Corbanu Terminal
- Model: Claude Fable 5.1 Plan
- Reasoning: max
- Target: uncommitted reconciliation follow-up on
  `integration/reconcile-release-0.1.37`

## Initial findings and disposition

1. **Deferred wallet-passcode cancellation left setup active — accepted.**
   The deferred prompt now emits cancellation for the exact continuation, and
   failed preflight cancels instead of falling into an ordinary wallet surface.
2. **Newly stored Corbanu API keys were not immediately selectable — accepted.**
   Current ordinary completion now activates the provider, refreshes policy from
   the synchronous durable vault write, and only then queues model selection.
3. **The one-time-key success path lacked true-TMUX proof — accepted.** The new
   scenario covers create, store, secure one-time reveal, in-process activation,
   model selection, persistence, and absence of false inactive-provider errors.
4. **`GatewayKey` plaintext was Debug-printable — accepted.** Debug output now
   redacts the key and has a dedicated unit test.
5. **Cache-write pricing was documented but absent from the account row —
   accepted.** The row and snapshots now include cache read and cache write.
6. **Credential storage remains synchronous on the UI thread — recorded as an
   inherited follow-up.** This predates the reconciliation and is not required
   to correct the release-line divergence.

## Follow-up finding and disposition

The first repair used a process-lifetime metadata override. Fable correctly
identified that it could leave Corbanu selectable after disconnecting the key.
The override and helper were removed. The policy test now stores a real key,
refreshes to selectable, deletes it, refreshes again, and proves it becomes
non-selectable. Activation is limited to the still-current ordinary operation;
deferred activation belongs to its exact continuation, while stale submitted
completions only store and reveal their non-recoverable one-time result.

## Final proof

- `codex-wallet`: 17/17.
- Focused TUI wallet/provider/continuation set: 33/33.
- Complete multi-provider true-TMUX module: 11/11 against the freshly built
  branded `corbanu` binary.
- Final focused Fable rereview: `FINAL_REVIEW: CLEAN`.

## macOS release-build follow-up

The first native macOS release build found that
`preferred_platform_store` parameters had been renamed with leading
underscores to satisfy non-macOS linting even though the macOS-only body still
used the original names. The parameters were restored and a
`#[cfg(not(target_os = "macos"))]` sink now consumes them on other platforms.
The native macOS release build and the remote Linux `just fix`/`cargo check`
then passed. A focused TMUX + Corbanu Terminal + Fable 5.1 Max portability
rereview returned `FINAL_REVIEW: CLEAN`.

No actionable Fable finding remains in the reconciled surfaces.
