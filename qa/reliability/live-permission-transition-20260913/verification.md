# Actual verification and remaining gates

All commands ran in the dedicated worktree unless explicitly noted. Date: 2026-09-13. This is a documentation/design checkpoint; source remains at the supplied base. Tool output is preserved in this task; concise results are transcribed below, not represented as raw runtime artifacts.

| Command / observation | Actual result |
| --- | --- |
| `pwd`, `git status --short`, `git branch --show-current`, `git rev-parse HEAD`, `git worktree list` | Exact assigned coordinates, clean initial state; no other worktree modified |
| `python3 docs/plans/check.py` | Exit 0: `plans: active 3/3; available slots 0` |
| `python3 docs/sprints/check.py` | Exit 0: `sprints: current 115; archived 126` |
| `python3 -B -m unittest discover -s docs/plans/tests -p 'test_*.py'` | Exit 0: 5 tests, 0.017 s, OK |
| `python3 -B -m unittest discover -s docs/sprints/tests -p 'test_*.py'` | Exit 0: 22 tests, 0.038 s, OK |
| `rustup run 1.95.0 rustc --version` | `rustc 1.95.0 (59807616e 2026-04-14)` |
| `rustup run 1.95.0 cargo --version` | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| Locked offline metadata command below | Exit 0; package metadata includes `0.1.42`. Output was truncated; no compilation, dependency completeness or binary qualification inferred |
| `shasum -a 256` on parent-supplied frozen functional cases | Exact supplied hash matches; original read without modification |

Metadata preflight, from `codex-rs`:

```sh
RUSTUP_TOOLCHAIN=1.95.0 CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913/.codex-work/live-permission-transition-target cargo metadata --locked --offline --no-deps --format-version 1
```

That target directory is exclusive to this worker and must remain separate from the active manager's cache. Future Rust test commands must retain the pinned toolchain, private target, `--locked --offline`, and repository `just test` workflow. No compilation or runtime tests were started for this proposal. Do not silently download missing dependencies or substitute the manager's cache.

Several exploratory `rg`/`sed` calls named nonexistent guessed files (including old `session/settings.rs`, `session/session_config.rs`, `session/tasks.rs`, `tui/src/chatwidget/thread_settings.rs`, `core/tests/lib.rs`, `app-server` settings paths and `codex-rs/justfile`). Those searches failed and were corrected using `rg --files`; they are not test failures or passes. No fallback read of auth/session logs occurred.

## Independent acceptance linkage

Original: `/Volumes/CorbanuDrive/Corbanu/.codex-work/permission-transition.eocbhB/frozen-functional-cases.md`.

SHA-256: `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`.

F01–F11 remain **unexecuted**. No case was removed, amended, passed, failed or dispositioned by this implementer. The proposed idle/refusal boundary must be evaluated against all original cases, especially F02/F04 admission safety, F05 outstanding approval, F06 in-flight work, F07 ordering and F10 recovery. Ambiguities remain in the original; the parent owns any product clarification, with additive records only. This mapping does not certify the proposal satisfies the cases.

## Remaining gates / owners

| Gate | State and owner |
| --- | --- |
| Runtime allocation | Blocked pending parent-recorded plan amendment and sole security sprint reservation; PF-27 stays paused |
| Boundary design proof | Not implemented; worker after approval must prove complete quiescence/admission and exact acknowledgement correlation or return scope blocker |
| Durable runtime regression tests | Not authored or run yet; implementer after allocation, synthetic fixtures only |
| Fresh Fable review | Not run; parent arranges exact candidate and preserves shared review ledger, no reset |
| Independent code-blind test design | Parent supplied original and matching hash; design is not acceptance |
| Independent isolated execution | Parent-owned, not started; enforce repository/history/process/IPC/network/credential boundaries with negative probes, positive PTY/package controls and synthetic state |
| Independent evidence review | Not started; reviewer independent of implementer and executor; schema-2 handoff checker later |
| True TUI / live repositories | Not run; TensorCash and Isometric Game applicability and disposable coordinates require recorded allocation; no access to live app/profile |
| Human acceptance | None; not ready for unqualified human testing |
| Finished documentation | None changed; these QA records describe unfinished work |
| Benchmarks / release | Not assessed for a new candidate; no candidate or release authorization, shortcut or package replacement |

The paused runtime lane has zero implementation workers running from this checkpoint. Other worker/process counts were not inspected. The next parent action is allocation and boundary approval, not another request to Travis for routine fix authority.
