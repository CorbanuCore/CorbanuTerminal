# PF20S03 integration handoff

Branch: `feat/security-local-anchor`; immutable base
`601602fa7e53fcb5b41753a0b3607addd45d4415`.
Final runtime/test source: `8d6967179`. Subsequent commits are documentation and
review/capture evidence only. No further source review is requested.

## Delivered and explicitly not delivered

Linux leaf controller-root dependency, exact PF41 root-last/PF20 anchor-first
adapters, authenticated bounded child transport and real synthetic subprocess
proof. Staged root-anchor/distinct-child topology accepted by coordinator and
broker. No actual root installation, worker-principal isolation, Vault migration,
protected activation, whole-machine rollback resistance or non-Linux native
qualification. Revised exact privileged manifest approval is still outstanding.

## Gates and reviews

Leaf18, Core17, audit46 and config229 passed. Full formatter and scoped fixes
passed; exact delegated Cargo.lock edge, no MODULE delta, Bazel update/check
passed and server shut down. Actual-key profiles2 and same-home restart passed,
with eleven final raw captures. Root should run combined-tree gates, not infer
them from the lane build.

Astra1 found an in-scope P2 (blocking connect before deadline), remedied and
tested in `8d6967179`. Fable2 verifies that remedy and reports patch correct/no
code blockers, plus two P3 notes. Both helpers exited1 and are not labeled clean.
Crate README now makes supported-filesystem TMPDIR explicit; root accepted and
owns the shared dashboard/coordination ledger refresh. Budget2/5 used; no further
review needed for these documentation-only dispositions.

## Exact immutable candidate and remote proof

Lane mirror: `/home/travis/worktrees/security-local-anchor`.
Evidence root: `/home/travis/security-round5/evidence/anchor`.
Binary: `candidate-review1/codex` under that evidence root.
SHA256: `42fe2d0168a4f9079f6faa4f4a7b6aa41deabe6f1322d0e921b8c40b4cfc4076`.
Final native/Core/build/restart log: `review1-remediation-proof.log`.
Correct-cwd profile rerun: `review1-exact-tmux-proof.log`.
Audit/config log: `final-proof.log`; Bazel parity log: `tmux-proof.log`.
Review artifacts and complete result/dispositions are committed beside this file.

## Combined-tree commands

Run on RTX under `/home/travis/security-round5/locks/build.lock`, with fresh TMPDIR
on the approved ext-family/XFS scratch root. Preserve
`CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target`, jobs8
and configured Rust1.95.0/just/uv/Bazel PATH. Refresh relevant workspace source
mtimes under the lock if another worktree populated shared artifacts; in
particular `utils/cargo-bin/src/lib.rs` captures a compile-time repository path.
Do not edit source to compensate for that cached marker.

```sh
just test -p codex-protected-state --retries 0 --test-threads 4
just test -p codex-core -E 'test(pf20_s03) | test(authoritative_state_tests)' --retries 0 --test-threads 4
just test -p codex-security-audit --retries 0 --test-threads 4
just test -p codex-config --retries 0 --test-threads 4
just bazel-lock-check
bazel shutdown
just codex --version
```

Copy the newly built **combined** CLI to a distinct evidence directory while
holding the lock and record its hash. Set `CARGO_BIN_EXE_codex` to that immutable
copy, `CORBANU_TMUX_REQUIRED=1`, and distinct `CORBANU_TMUX_ARTIFACT_DIR` /
`CORBANU_SECURITY_UI_EVIDENCE` directories. Then:

```sh
just test -p codex-tui --test all -E 'test(security_profiles)' --retries 0 --test-threads 1
python3 qa/security-levels/sprints/PF-20-S03/tmux_restart.py --binary <combined-immutable-cli> --repo <combined-worktree> --evidence <combined-restart-evidence>
```

The Python command is shown from repository root; `just` resolves its own working
directory. Keep the supporting UI unverified. Root owns final governance checks,
shared progress text2/5, integration/merge/push and bounded sprint archival.
