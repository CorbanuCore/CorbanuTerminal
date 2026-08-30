# PF-19-S02 dispatch revocation fence evidence

- Date: 2026-08-30
- Allocation base: `5521b681fff0ecb50b17c10bc1dd1356cbecc1b6`
- Implementation parent after fast-forward: `77f56da1ecddf6093184280b541339e1869ca7b3`
- Implementation commit: `baeed85e4a826077d73428a41ef1c37b6e795747`
- Contract version: `DISPATCH_FENCE_SCHEMA_VERSION = 1`
- TUI applicability: none; this is an adapter-neutral policy contract.

## Contract result

`DispatchFence` binds one bounded run identifier, one validated grant or mandate,
and the accepted revocation generation. Its state machine permits only these
active transitions:

```text
queued -> admitted
admitted -> established_channel | uploading
established_channel -> uploading | established_channel (channel write)
uploading -> uploading (upload write) | established_channel (finish upload)
active | fenced -> completed | unknown_financial_outcome
```

Every protected admission, channel write, and upload write rechecks the exact
run/authority binding and current `RevocationState` generation. Consumers must
perform that check while holding the same state read guard through the actual
effect; that is the linearization point. A revoked authority or active kill
switch moves the work to `fenced`. Any generation change denies the old
operation. An unaffected sibling can continue only after a trusted explicit
refresh proves its grant or mandate remains valid at the new generation.

`RevocationState::apply_restriction` applies the restriction before invoking the
audit callback. The returned `RestrictionApplication` preserves whether audit
was recorded or unavailable. Audit failure therefore cannot delay the stop.
Completed and unknown financial outcomes are terminal, idempotent observations;
they cannot be relabeled as each other or as cancelled by revocation.

## Changed paths

- `codex-rs/security-policy/src/revocation.rs`
- `codex-rs/security-policy/src/security_policy_tests.rs`
- `docs/sprints/current/p0-security-levels/pf-19-s02-dispatch-revocation-fence.md`
- `qa/security-levels/sprints/PF-19-S02/evidence.md`

The PF-19-S01 archive and evidence are unchanged. No OpenClaw source was copied;
its pinned open-channel gap informed the clean-room Corbanu contract only.

## Final-tree automated evidence

All caches, targets, temporary files, logs, and captures were placed below
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-revocation-fence/`.

| Check | Result | CorbanuDrive artifact SHA-256 |
| --- | --- | --- |
| `cd codex-rs && just fix -p codex-security-policy` | PASS; Clippy completed | `f6330cdcebcbca3f6cfaf8492a38d5399e90ffd9c50b7f73bbc7a4ece76131f7` |
| `cd codex-rs && just fmt` | PASS; no output and no diff | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `cd codex-rs && just test -p codex-security-policy revocation` | PASS; 9 passed, 36 filtered | `49e020853f97ae924462e0843958ab7821f2148f8a69086fb7cc130ddb01aec6` |
| `cd codex-rs && just test -p codex-security-policy` | PASS; 45 passed, 0 skipped | `24e89764512a13870a5481c9c5f847965010029ccfa0ab973121d2ba626c186a` |

The same focused command ran in the real PTY session
`pf19-revocation-tests` with command text and Enter sent separately: 9/9 passed.
Capture:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-revocation-fence/tmux-artifacts/focused-tests-pane.txt`,
SHA-256 `e1a2ff6b3f6ebdc5f1f8331424bd10896af3e42d68f35565d2ec1866c819048d`.

## Consumer and integration handoff

The lane intentionally did not edit the shared `security-policy/src/lib.rs`.
The integration owner must re-export these symbols before a downstream crate can
consume the contract: `DISPATCH_FENCE_SCHEMA_VERSION`,
`DispatchAuthorityKind`, `DispatchFence`, `DispatchPhase`,
`ProtectedDispatchStep`, `RestrictionApplication`, and
`RestrictionAuditStatus`. The integration owner then reruns the complete policy
suite and governance checks before archiving PF-19-S02.

This sprint proves the pure contract and deterministic interleavings only. It
does not claim transport adoption, cross-process propagation, durable restart,
financial reconciliation, kill-switch TUI behavior, or final release
qualification. Those remain with the named consumer sprints and PF-26.
