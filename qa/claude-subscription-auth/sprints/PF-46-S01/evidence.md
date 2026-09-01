# PF-46-S01 final automated qualification evidence

## Candidate boundary

- Frozen base: `8ae13e168817445205321bae410740cbc3e919b7`.
- Isolated branch: `feat/claude-subscription-auth-isolated`.
- Final implementation repair candidate: `b96326f01`.
- Coordination-only lifecycle mirror: `f36c28770`; it does not import the
  corresponding P0 product implementation into this isolated lineage.
- Candidate binary: `codex-rs/target/debug/corbanu`, version `0.1.35`, SHA-256
  `9517a17105e62697bcdef0da68a26fceb04821b7643e82c8ec86d9a5ef3acf4e`.

## Formatting, static checks, and documentation

- `CARGO_INCREMENTAL=0 just fix -p codex-cli`: passed.
- `CARGO_INCREMENTAL=0 just fix -p codex-tui`: passed; only unrelated existing
  test-target lint warnings were emitted.
- `just fmt` and `git diff --check`: passed.
- `scripts/check-module-bazel-lock.sh`: passed, invocation
  `f93e1bd8-5043-405b-9e6e-12e3ea856e6d`.
- `uv run --isolated --with-requirements requirements-docs.txt mkdocs build
  --strict`: the two stale nav entries reported by Opus were removed; strict
  build then stopped on the pre-existing out-of-scope broken link from the P0
  security plan to a missing QA handoff file. No Claude-auth warning remained.
- `python3 docs/plans/check.py` and `python3 docs/sprints/check.py`: passed on the
  archived final documentation tree.

## Retry-disabled affected tests

| Surface | Result | Nextest run |
| --- | --- | --- |
| Vault Claude auth | 21/21 passed | `0e40f330-2ae9-48ac-bebf-f80427224fa7` |
| CLI Claude OAuth before final repair | 115/115 passed | `9ac89801-5e66-4b9e-b4ee-7b5d30e23d71` |
| TUI Claude login after final repair | 22/22 passed | `f3dc0d72-87b5-42ea-807c-78c12f6a0a9b` |
| CLI Claude OAuth after final repair | 120/120 passed | `63905eed-645a-43e8-bf66-7a5b3a842b6f` |
| Generated Claude Plan settings | 1/1 passed | `2d25b766-21f9-49f2-ad90-6e0c3230e0f9` |
| External bearer cache behavior | 4/4 passed | `b9fb3e75-7d58-428c-8f0e-996827d7c069` |
| Claude Plan cache policy | 1/1 passed | `d2258b6b-ea30-439a-bb0b-b2a7251e2e16` |
| Core provider auth and 401 | 2/2 passed | `1a46d371-8aea-4c7e-8347-b58cbbb6ae6e` |
| Final FreshPerRequest refresh behavior | 3/3 passed | `754b39d7-3e13-449f-8bc8-d0e23b8cb1ce` |
| Final CLI Claude OAuth custody and rotation | 120/120 passed | `93a0303b-3941-4f36-a06e-8ac070cb5209` |
| Final TUI status and bounded-output regressions | 7/7 passed | `b90d9758-c100-4bed-aaed-b40ff8aa0745` |

Rust 1.95 produced one incremental compiler ICE during an earlier CLI attempt.
The affected suites were rerun with `CARGO_INCREMENTAL=0`; test retries remained
zero, so no failing test was hidden or retried.

## Required true Tmux harness

Command used `CORBANU_TMUX_REQUIRED=1`, an isolated
`CORBANU_TMUX_ARTIFACT_DIR`, `CARGO_INCREMENTAL=0`, `-j 1`, and `--retries 0`.
Nextest run `9b9d555f-b3ee-4b7f-8e8a-f7182a33cf8d` passed 2/2:

- compatibility-login selection: 17.991 seconds;
- managed success, cancel, failure, recovery, masked cancel, and restart/resume:
  56.285 seconds.

After the final Opus repairs, retry-disabled serial Nextest run
`34bcd442-2f4e-4fe5-b3d1-5e8445007982` passed the same 2/2 harness cases in
15.546 and 56.334 seconds. A preceding concurrent invocation was excluded from
evidence because the managed case crossed Nextest's 60-second ceiling while
competing with the compatibility case and the local profile retried it; the
recorded serial run used `--test-threads 1 --retries 0`.

The typed harness sends text and Enter separately, uses typed Down/Enter/Escape
keys, runs the real candidate binary with `RUST_LOG=trace`, and checks its unique
synthetic credential canary against viewport, scrollback, isolated home, trace,
and retained artifacts. The successful run emitted no failure bundle beneath
`.codex-work/claude-subscription-auth/f0d5b0b1-final-qualification/tmux`.

## Review history and dispositions

- Structured review of `bb3079eb` reported a possible project-directory helper
  hijack. Rejected with runtime evidence: Core rewrites the built-in Claude Plan
  auth command to the absolute running executable before provider merge, the
  existing `claude_plan_auth_uses_the_runtime_executable_without_path_lookup`
  regression proves it, and the actual trace recorded the absolute helper path.
  Generic custom-provider command semantics were intentionally unchanged.
- Corbanu Terminal with `model_provider="claude-plan"`,
  `model="claude-opus-5-plan"`, reasoning `max`, approval `never`, and read-only
  sandbox found no P0/P1. Its accepted P2/P3 findings were repaired in
  `34535821c`: stale nav entries, literal sprint scope, bounded unterminated
  login output, accurate missing-environment-token recovery, and removal of an
  inert pane-precedence fixture/claim.
- Final structured review then found that the generic 60-second external bearer
  cache could outlive a newly persisted Claude auth choice. `f0d5b0b16` adds a
  provider-owned fresh-per-request cache policy for Claude Plan only, plus
  behavioral and policy regressions; custom command-auth providers retain their
  existing configured cache semantics.
- The exact `41164e358` Corbanu Terminal Opus 5 Max review found transient
  health failures collapsed into reauthorization, a split UTF-8 boundary in the
  bounded login reader, dead FreshPerRequest refresh retention, and ordinary
  heap custody for compatibility credentials. `b96326f01` fixes all four with
  typed transient status mapping, lossy conversion only for the already-bounded
  oversize signal, cache-policy-symmetric refresh behavior, and zeroizing raw
  JSON/access/refresh-token custody plus a redacted credential `Debug` surface.
  Its trace SHA-256 was
  `eaf8ae66d971b73e9719272747300e38596e4acc6a2272715aa303b8084d61e3`.
- The review's proposed removal of the post-resolution authority check was
  rejected because a non-mutating credential read can still race an external
  account replacement. The bare-command/cwd concern was rejected because Core
  rewrites the built-in provider to the absolute runtime executable before
  merge and tests that invariant. Opus independently withdrew its short-token
  redactor finding after reading the explicit no-disclosure regression.
- The immutable final documentation commit is reviewed again against the full
  frozen-base diff before push. A non-clean verdict blocks delivery; final
  external review artifacts and the pushed SHA are reported in the handoff.

## Deliberately unclaimed external evidence

No named human used a live eligible Anthropic account during automated
qualification. TensorCash, Isometric Game, physical Linux/Windows release-host,
release/tag/merge, release ledger, and due-benchmark acceptance remain open on
the active plan. No PR or release was created by this sprint.
