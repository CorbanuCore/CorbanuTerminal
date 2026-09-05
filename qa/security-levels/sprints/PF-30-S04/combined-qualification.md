# Final combined memory/security qualification

Requested and checked-out source: `6a6bb029d8f3e0c16653ce335d252f45b4d7326f`.
RTX mirror: `/home/travis/worktrees/security-round5-integration`.
Evidence root: `/home/travis/security-round5/evidence/integration/6a6bb029d/`.

## Completed preparation

- Initial remote integration checkout was clean at dd2adb72b; fetched and checked out exact requested commit.
- Scoped Core `just fix`, memories-write `just fix` and full `just fmt` passed. `formatter.patch` is empty, worktree clean; no content/lockfile changes.
- Build lock shared across lanes; Rust mtimes refreshed inside lock to prevent stale cross-worktree dependency artifacts.
- Format TMPDIR: `/home/travis/security-round5/integration-tmp/run.KdMvii`.
- Test TMPDIR: `/home/travis/security-round5/integration-tmp/run.kUQC1t`.
- No local builds, no new independent reviews, no repository/doc edits by the qualification agent.

## Final gates

All requested combined gates passed. Final worktree remains clean at the exact
requested source; shared build lock released normally. No content or lockfile
change, no additional review and no local build occurred.

| Gate | Result | Nextest run / log |
| --- | --- | --- |
| Scoped Core and memories-write fix; full fmt | Pass, empty formatter patch | `fix-core.log`, `fix-memories.log`, `fmt.log` |
| Focused Core memory/provenance/realtime/broker/proxy | 110/110; 3381 filtered, 11.554s | `db3c3d98-30dc-4482-a592-597c529ecc6e` |
| Full memories-write | 44/44, 2.775s | `6943c161-66ed-472e-9678-86fb8325ecf0` |
| Full memories-read | 3/3, 0.002s | `09c9e34f-724b-496f-b792-7a2b1b4507ad` |
| Locked CLI build | Pass | `cli-build.log` |
| Focused security/status/slash UI units | 235/235; 3737 filtered, 1.205s | `b82c130c-5ff2-42a1-9f5e-d73e54de5b1a` |
| Actual-key TMUX | 4/4; 71 filtered, 23.755s | `b73ed422-fe5b-4d5d-bce1-79878276add5` |
| Plan/sprint governance, diff check, clean worktree | Pass; active2/current59/archived114 | Final command output |

## Immutable tested binary and captures

Launch path: `/home/travis/security-round5/evidence/integration/6a6bb029d/candidate/codex`.
Version: `corbanu 0.1.38` (Linux RTX candidate, not Mac app shortcut).
SHA-256 before and after actual-key tests:
`90d6a1f7f72c5397ff858583c038b2615c8fb034f57a890d6595d6b98afccd4f`.
Copied while the shared lock was held and explicitly selected by `CARGO_BIN_EXE_codex`.

Four TMUX tests are: memory worker policy (11.118s), observation-only security
profiles at normal/narrow widths (10.625s), unknown configuration denial (0.543s),
and single-Enter slash/status/exit smoke (1.467s). The memory test includes four
scenarios and four same-home restarts, with separately delivered literal text
and Enter keys, actual fake-provider request counts and SQLite output counts:

| Scenario | Raw-canary requests | Persisted stage-one outputs |
| --- | ---: | ---: |
| Permissive | 1 | 1 |
| Moderate | 0 | 0 |
| Aggressive | 0 | 0 |
| Permissive owner exit while extraction pending | 1 | 0 |

Final keys/captures/outcomes and synthetic traces: `memory-tmux/` under evidence
root. Four outcome and four input-event files were read back after completion.
Seven `security-tmux/` captures were read back: requested-only profiles and status
at 120/40/80 columns plus unknown configuration error. They explicitly say live
effective protection is unverified and applying profiles is unavailable; no
healthy protected status is asserted. Narrow security text wraps, Enter is inert,
Esc/reopen and unchanged configuration are exercised by the typed harness.

This is final combined qualification of the merged memory+provenance+UI tree,
not merely reuse of the standalone branch's results. Coordinator can record
these gates and archive PF-30-S04; no new source changes are proposed.

Commands are preserved in sibling `qualify-combined-format.sh` and `qualify-combined-tests.sh`; they are executed on RTX over the authenticated SSH master, not locally.

Core filter: `test(pf_30_s04) | test(pf_30_s01) | test(realtime_conversation) | test(broker_client) | test(network_proxy_credential)`.
Full memory-write/read; locked CLI copied before lock release; focused TUI filter `test(security_view) | test(status::tests) | test(slash_command)`.
Actual-key TMUX filter: `test(tmux_memory_worker_policy) | test(security_profiles) | test(tmux_smoke_single_enter_dispatches_slash_command_and_exits_cleanly)`.

No broad full-Core claim (known unrelated baseline failures), no human, live-provider, Mac shortcut, cross-platform, release or positive protected-memory acceptance inferred.
