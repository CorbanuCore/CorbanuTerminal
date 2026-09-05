# RTX qualification — staged PF-30-S01

Source candidate: `e592cf75a`, allocation `4f263ca73`.
Host: RTX Linux, `travis@100.99.88.49`. No local builds.
All builds, fixes, formatter and test commands were serialized by
`flock /home/travis/security-round5/locks/build.lock`.

Use a fresh `TMPDIR` from `/home/travis/security-round5/provenance-tmp/`
for every run, outside the polluted shared `/home/travis/security-round5/tmp`.
The shared Cargo target is
`/home/travis/repos/CorbanuTerminal-harness/codex-rs/target`.
The tested CLI was copied while holding the lock, before another lane could
replace the shared executable.

## Final scoped proof

Commands, from `codex-rs` with Rust 1.95.0 and eight build jobs:

```sh
just fix -p codex-core
just fmt
just test -p codex-core -E 'test(pf_30_s01) | test(realtime_conversation)' --retries 0 --test-threads 4
cargo build --locked -p codex-cli --bin codex
just fmt-check
```

Results: 88/88 focused Core tests passed (22 provenance, 66 other realtime).
The new regressions prove protected WebSocket/WebRTC startup rejects before any
loopback connection, and an actual Permissive WebSocket sends normal text but
rejects the next frame after a live increase to Moderate. This is a per-operation
boundary, not a claim of retroactive revocation or immediate idle-socket closure.
Scoped formatting and locked CLI build passed.

Final actual-key supporting smoke:

```sh
CARGO_BIN_EXE_codex=/home/travis/security-round5/evidence/provenance/candidate-e592cf75a/codex \
CORBANU_TMUX_REQUIRED=1 \
CORBANU_TMUX_ARTIFACT_DIR=/home/travis/security-round5/evidence/provenance/tmux-realtime-final \
RUST_LOG=trace just test -p codex-tui --test all \
  -E 'test(tmux_smoke_single_enter_dispatches_slash_command_and_exits_cleanly)' \
  --retries 0 --test-threads 1
```

Result: 1/1 passed. The typed TmuxServer fixture enters `/status`, presses Enter,
observes its response, then enters `/exit` and presses Enter for clean exit. It
does not substitute an exec API for the terminal interaction. The harness only
retains detailed pane artifacts on failure; no retained passing screenshots are
claimed. This supporting smoke is not complete human acceptance.

`python3 docs/plans/check.py` and `python3 docs/sprints/check.py` passed:
two active plans, 58 current sprints, 113 archived sprints.

## Full-suite classification

Full protocol passed: 285/285. Full Core, with fixture `test_stdio_server` built,
fresh TMPDIR and four workers: 3,455 passed, five failed, eight skipped. All five
failures were independently reproduced on allocation `4f263ca73` with the same
fixture and isolated environment. The `suite::request_permissions` tests are:

- `partial_request_permissions_grants_do_not_preapprove_new_permissions`
- `request_permissions_grants_apply_to_later_exec_command_calls`
- `request_permissions_grants_apply_to_later_shell_command_calls`
- `request_permissions_grants_apply_to_later_shell_command_calls_without_inline_permission_feature`
- `request_permissions_preapprove_explicit_exec_permissions_outside_on_request`

This is evidence of existing failures, not a wholly green full Core suite.
Earlier runs were superseded after repairing a missing test fixture, reducing
parallel test workers and isolating a polluted shared temporary directory.
Unrelated files and shared temporary data were not removed or patched.

## Evidence integrity

Logs below are under `/home/travis/security-round5/evidence/provenance`.

| Artifact | SHA-256 |
| --- | --- |
| `focused-4.log` (protocol plus earlier provenance) | `b4db9078dca1b3a1eba18442a033ac518982227745345dabf0c71e8321a63a82` |
| `review-remediation-1.log` (session lifecycle) | `b5fe8b523c0d1368dc01c0ded93a3b407fd45336b6cb5aac03e580deba35f50e` |
| `full-core-4.log` | `250cead22824d05986db13349d01ad3e071fca46a7141efabe5906c9ff4bee85` |
| `baseline-request-permissions.log` | `6bcad6ed0417d7783947c4a65f086ba1eb8db39985e8bf6447f09800dfd8fea7` |
| `realtime-remediation-1.log` | `bc0abf97b8284d9cfd537eaf89143fc718bfa2afc0daa7e52452e57d263d933c` |
| `realtime-final-tmux.log` | `171551d8490741aac3c9b5c2ade7f3458f209c4ad3aa876170518b548ccac40d` |
| `candidate-e592cf75a/codex` | `a30efcf7ed089837546c744249a2aa1fb9460d119a659793be7880332160a75d` |

## Review and completion limits

Three of five allowed reviews used: Astra High, then two Fable 5.1 High
structured Corbanu reviews inside private TMUX. Review 3 verified the prior
fixes but reported the separate unbound stage-one memory worker. The finding and
scope escalation remain visible in `review-disposition.md`; no overall clean
review is claimed. Production screening delivery, complete-input segmentation,
fine-grained source coverage, memory/persistent lineage and post-taint control
remain unfinished. Keep PF-30-S01 `in_progress`; do not treat fixture screening
as production qualification or enable protected modes from this evidence alone.
