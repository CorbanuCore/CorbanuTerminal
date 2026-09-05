# RTX verification — PF20S03

Final implementation source: `8d6967179` on `feat/security-local-anchor`.
This report/capture commit is documentation only. Exact source/build diff from
`601602fa7e53fcb5b41753a0b3607addd45d4415`: 16 files, 2,107 added lines,
including 707 test lines. No existing provider/cache/input-shaping flow changed.

Host: RTX `100.99.88.49`, user `travis`; no elevation, service/principal/ACL setup,
TPM operations or real credentials. Remote mirror:
`/home/travis/worktrees/security-local-anchor`. All builds/fix/fmt/test/Bazel used
`flock /home/travis/security-round5/locks/build.lock`, fresh per-run TMPDIR under
`/home/travis/security-round5/anchor-tmp`, and shared Cargo target/jobs8. There
were no local builds. Bazel was shut down before releasing the lock.

## Automated gates

| Command | Result | Remote log under `/home/travis/security-round5/evidence/anchor/` |
| --- | --- | --- |
| `just fix -p codex-protected-state` | Pass | `review1-remediation-proof.log` |
| `just fix -p codex-core` | Pass; no new warnings | `final-proof.log` |
| `just fmt` | Pass; remote formatting synchronized exactly | `review1-remediation-proof.log` |
| `just test -p codex-protected-state --retries 0 --test-threads 4` | 18 passed; two child helpers invoked by actual parent subprocess tests | `review1-remediation-proof.log` |
| `just test -p codex-core -E 'test(pf20_s03) \| test(authoritative_state_tests)' --retries 0 --test-threads 4` | 17 passed | `review1-remediation-proof.log` |
| `just test -p codex-security-audit --retries 0 --test-threads 4` | 46 passed | `final-proof.log` |
| `just test -p codex-config --retries 0 --test-threads 4` | 229 passed | `final-proof.log` |
| `just bazel-lock-update` / `just bazel-lock-check` | Pass; no MODULE delta | `tmux-proof.log` |
| `just codex --version` | Build/run pass; v0.1.38 | `review1-remediation-proof.log` |

Cargo.lock contains only the delegated new leaf dependency package and Core edge.
No dependency versions or shared policy protocol were changed. Config/audit
suites ran before final descriptor/socket nonblocking corrections; they do not
depend on this new leaf. Leaf/Core and actual-key gates reran afterward.

The first Core fixture failed because its temp directory was not private enough;
explicit0700 corrected the fixture without weakening product checks. Initial
Bazel failed to resolve its executable; PATH was corrected to the existing
`/home/travis/security-round5/tools/bazel`. Neither failure is hidden as a pass.
Existing unrelated Core/TUI dead-code warnings remain.

## Real subprocess and injected-fault coverage

- One-shot enrollment, authenticated Genesis, exact full CAS, restart, wrong
  namespace, foreign owner/key, stale compare, overflow and generation regression.
- Missing/corrupt independent registry/key/head/lock, interrupted enrollment,
  torn pending state, symlink replacement, live directory permission drift and
  nonregular FIFO rejection without blocking.
- Real separate-process exclusive-lock contention and real post-exec child IPC;
  parent-created socketpair negative control; frame replay/cross-generation/
  oversized frame and absolute partial-frame deadline rejection.
- Real saturated-listener backlog rejects bootstrap immediately, while the
  post-exec child uses the same nonblocking-connect helper on its successful path.
- Definite authenticated conflict vs uncertain lost receipt; client consumption
  with no blind retries. Injected no-space/partial-write/file-sync/post-rename
  directory-sync/post-durable receipt failure with withheld acknowledgment.
- Real PF41 journal rejects restored/deleted data against retained controller
  checkpoint. Core policy adapter preserves complete anchor-first payload and
  exact recovery with an explicitly synthetic platform report.

These do not prove separate-principal confinement, privileged deployment, physical
power-loss durability, actual disk-full behavior or whole-machine rollback
resistance. PF41's pre-existing first-record ambiguous-recovery limit remains.

## Actual-key supporting TMUX

Immutable binary:
`/home/travis/security-round5/evidence/anchor/candidate-review1/codex`

SHA256: `42fe2d0168a4f9079f6faa4f4a7b6aa41deabe6f1322d0e921b8c40b4cfc4076`.

Built and copied while holding the shared lock. Runtime
`CARGO_BIN_EXE_codex` explicitly selects that copy. Tests use synthetic API-key
fixtures, no live inference, private TMUX sockets, trace logs and separate literal
text/Enter events. The initial shared-cache test utility retained another
worktree's compile-time repo marker; it was refreshed and profiles rerun. Final
captures show this exact lane directory, not the stale marker.

`just test -p codex-tui --test all -E 'test(security_profiles)' --retries 0 --test-threads 1`
with `CORBANU_TMUX_REQUIRED=1` passed2/2. This covers 120-column Permissive,
40-column Moderate,80-column Aggressive, Down/Enter inspection, Nothing changed,
Escape, `/status`, `/exit`, and unknown startup level denial without fallback.

`python3 qa/security-levels/sprints/PF-20-S03/tmux_restart.py --binary <immutable> --repo <mirror> --evidence <remote evidence>/tmux-review1-restart`
passed two actual process starts on one unchanged Moderate config/home, each
using `/security`, Escape, `/status`, `/exit`. Both remain visibly unverified.
Eleven final raw captures are in `tmux/`. No graphical screenshot or real
protected-session success is claimed by these text/actual-key fixtures.

## Outstanding gates

Broker confirms the proposed root-anchor/two-root/distinct-child topology is
consumable; root coordinator explicitly accepted it as the staged PF20 design.
It supersedes the earlier nonlogin-controller proposal for this staged design
only. Actual service installation remains unapproved and requires a revised
exact privileged manifest. No same-process dual-domain requirement exists;
no speculative endpoints were added. Idle ten-second expiry requires explicit
rebootstrap and never retries an ambiguous CAS. Astra High review1 found one
verified P2, corrected and tested in `8d6967179`; it was not a clean result.
Fable5.1 High review2 then reported patch correct/no code blockers, with two
nonblocking P3 notes and helper exit1. The test TMPDIR requirement is now also
documented in the crate README. Root owns the shared ledger/progress refresh.
Current ledger2/5; see `reviews.md`, exact structured results and TMUX exit pane.
No new source change or extra review followed documentation-only dispositions.
Coordinator combined-tree qualification,
eventual privileged two-principal qualification and release/human acceptance
remain separate. Sprint stays in_progress.
