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
admitted -> established_channel
established_channel -> uploading | established_channel (channel write)
uploading -> uploading (upload write) | established_channel (finish upload)
admitted | established_channel | uploading -> completed | unknown_financial_outcome
fenced after admission -> completed | unknown_financial_outcome
```

Every protected admission, channel write, and upload write rechecks the exact
run/authority binding and current `RevocationState` generation. Consumers must
perform that check while holding the same state read guard through the actual
effect; that is the linearization point. A revoked authority or active kill
switch moves the work to `fenced`. Any generation change denies the old
operation. An unaffected sibling can continue only after a trusted explicit
refresh proves its grant or mandate remains valid at the new generation.
Refresh cannot lower a fence generation, and every queue, refresh, admission,
channel, and upload boundary rechecks the grant or mandate validity window.
Queued work fenced before admission cannot be relabeled as completed or
financially unknown; terminal observations remain terminal under later checks.

`RevocationState::apply_restriction` applies the restriction before invoking the
audit callback. The returned `RestrictionApplication` preserves whether audit
was recorded or unavailable. Audit failure therefore cannot delay the stop.
An active kill order superseded by a newer kill-switch order fails visibly with
`RestrictionSuperseded` before state or audit mutation, so clock rollback cannot
turn a non-effective operator stop into a successful response.
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
| `cd codex-rs && just fix -p codex-security-policy` | PASS; Clippy completed without warnings | `5bee77f8ed43b246017c91ed7460b62ccd3b553616170ed3092bc3b807583340` |
| `cd codex-rs && just fmt` | PASS; no output and no diff | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `cd codex-rs && just test -p codex-security-policy revocation` | PASS; 10 passed, 36 filtered | `69ca49c642c87be69d4eb4fb004a83f1379a01ddbcdc745eec5645bee40cbab2` |
| `cd codex-rs && just test -p codex-security-policy` | PASS; 46 passed, 0 skipped | `8395ffad90212cd59a1d1de7607b01bb1fa8f0c980a70171d2a22a6c56adc128` |
| `python3 docs/plans/check.py` | PASS; active 1/2, one slot available | `9386e473c028f912d1685f25a88db8d21e57b8a9ad0929b07fcca3262ecbc8fb` |
| `python3 docs/sprints/check.py` | PASS; 64 current, 91 archived | `a7521385d2c53b4ba1b577fafc1b21953ef1325d024289d2af909facd941b370` |
| `git diff --check` | PASS; no output | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

The same focused command ran in the real PTY session
`pf19-revocation-final` with command text and Enter sent separately: 10/10 passed.
Capture:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-revocation-fence/tmux-artifacts/final-focused-tests-pane.txt`,
SHA-256 `5e694c3f2e28c7461afc3d04d27a63f6ee4dd149a8aeab4ba3361a4556951751`.

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
