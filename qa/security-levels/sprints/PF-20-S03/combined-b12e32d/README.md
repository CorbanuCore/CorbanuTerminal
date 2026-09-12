# Final combined PF20 + memory/security qualification

Requested and tested runtime source:
`b12e32db398c83854271e2e70f29e5290278af8b` on
`integration/security-round5-20260904`. Fresh detached RTX mirror:
`/home/travis/worktrees/security-anchor-combined-b12e32d`.
All requested gates passed; final remote worktree is clean at that exact commit.
No source, formatter, Cargo or Bazel lock delta was produced.

This is combined qualification of the merged controller-root and PF30 memory
work, not reuse of standalone lane proof. The product initiative remains linked
to PF20S03 and the existing non-negotiable tamper-evident record requirement;
the user's data-rollback scope excludes whole-controller/whole-host rollback.

## Preparation and immutable candidate

Authenticated RTX user `travis`; all commands serialized by
`/home/travis/security-round5/locks/build.lock`. Supported local-filesystem
TMPDIR: `/home/travis/security-round5/anchor-tmp/combined-b12e32d.MJZ9TS`.
Shared Cargo target/jobs8 and pinned Rust1.95.0; no local build. Existing mirrors
and remote canonical main were untouched. Rust mtimes, including cargo-bin's
compile-time repo marker, were refreshed only under the build lock.

Scoped leaf/Core/memory `just fix` and complete `just fmt` passed. The saved
`formatter.patch` is zero bytes. Bazel parity passed; Bazel shutdown happened
before lock release. Existing unrelated Core/TUI dead-code warnings remain.

Immutable binary:
`/home/travis/security-round5/evidence/anchor/combined-b12e32d/candidate/codex`

Version: `corbanu 0.1.38` (Linux RTX candidate, not Mac app shortcut).
SHA256 before/after actual-key tests:
`c567826ff5f15fccd71f8294c93210a158217a2ba31224c55d7d78269b1d2bea`.
Copied while the lock was held and explicitly selected by `CARGO_BIN_EXE_codex`.

## Final gates

| Gate | Result | Nextest run ID |
| --- | --- | --- |
| Full protected-state | 18/18, 2.012s | `3f78a3b1-2fa8-4b3b-9afe-0596176a96bb` |
| Core anchor + memory/provenance/realtime/broker/proxy union | 127/127, 11.647s | `eab121b0-e60b-4633-ad4d-a0707d555f2f` |
| Full security-audit | 46/46, 0.093s | `831b257a-b2a4-4695-9cf6-3ca1c6c37718` |
| Full config | 229/229, 0.146s | `7dd8fb08-4939-4ea2-82e8-a57d320829c9` |
| Full memories-write | 44/44, 2.994s | `b1638dec-9333-4fa1-8d1a-8ec069ff536f` |
| Full memories-read | 3/3, 0.002s | `386dec55-36ed-49c0-9780-fdaeb2ebac48` |
| Focused security/status/slash UI | 235/235, 1.232s | `9298d518-875c-4f18-ad53-f11bbe72e7c3` |
| Actual-key memory/profiles/invalid-config/slash TMUX | 4/4, 23.730s | `69eb14bb-ad98-41f9-8130-5c706b37c412` |
| Same-home anchor-profile restart script | Pass, two actual process starts | `restart.log` |

Two ignored protected-state helpers are explicitly invoked by their real
subprocess parent tests. Filtered tests are not claimed run: Core3366, UI3737,
TMUX71. No broad full-Core or complete-workspace claim is made.

## Actual-key outcomes

The memory test sends literal text and Enter separately, checks real fake-server
requests and SQLite results, and restarts each of four homes:

| Scenario | Raw-canary requests | Persisted stage-one results |
| --- | ---: | ---: |
| Permissive | 1 | 1 |
| Moderate | 0 | 0 |
| Aggressive | 0 | 0 |
| Permissive owner exits with extraction pending | 1 | 0 |

All four outcome/input-event records and worker/restart text captures were read
back and committed under `memory-tmux/`. Source data and provider are synthetic;
this proves dispatch, cancellation, persistence and restart behavior, not live
provider authentication or qualified positive protected-memory screening.

Security captures cover120-column Permissive,40-column Moderate,80-column
Aggressive, arrows/Enter inspection, inert "Nothing changed", Escape, `/status`,
`/exit`, and unknown configuration denial without Permissive fallback. Profiles
remain visibly unverified; actual application is unavailable. The additional
anchor restart script uses one unchanged Moderate config/home across two actual
process starts with `/security`, Escape, `/status`, `/exit`. Seven security and
four anchor-restart captures were read back and committed. Final captured
directories resolve to this exact combined mirror.

## Reproducibility and remaining authority

Executed script is `qualify.sh`; it requires the shared lock externally and
refuses an existing target mirror or any fix/format delta. Remote raw logs and
trace artifacts are under
`/home/travis/security-round5/evidence/anchor/combined-b12e32d/`;
outer execution log is sibling `combined-b12e32d-run.log`. Final exit0,
checksum verification, clean status and exact HEAD were confirmed after tests.
Build lock released normally; fixture agent was notified.

Coordinator's subsequent `1ad3e4ef5` corrects sprint heading placement only;
root reports governance passes58current/115archived and plans2/2. That later
documentation is not silently included in this exact runtime test claim. Root
owns final plan/sprint governance, archival, shared progress pages and merge/push.

No additional independent review was invoked; PF20 ledger remains2/5 with the
recorded Astra P2 correction and Fable no-code-blocker/two-nonblocking-note
disposition, not a false clean-review claim. No privileged installation,
principal/ACL change, real Vault transfer, protected activation, physical power
cut, whole-machine rollback, macOS/Windows, live TensorCash/Isometric, human or
release acceptance is inferred. These remain separate deployment/release gates.
