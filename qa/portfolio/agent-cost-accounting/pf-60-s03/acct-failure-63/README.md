# acct-failure-63: "Native Anthropic accounting failed" under state-DB contention (2026-10-03)

Moved from `.codex-work/workers-20261002/acct-failure-63.*` and `.codex-work/acct63/` on 2026-10-08.
Logs renamed `.log` -> `.txt` (the repo ignores `*.log`).

Commits (on main via PR #138, merge `3ecc3c4066`): `f4d6c5888a`, `a956468dad`, `7bef7a8dfd`, `79aef0daf3`.

| File | What it is |
|---|---|
| `BRIEF.md` | Worker brief |
| `RESULT.md` | Worker result: root cause, fix, tests, review disposition, installs, functional rounds |
| `review-brief.md`, `review.md` | Independent review brief and verdict (APPROVE WITH NITS; one major left open: day-90 sweep, closed by acct-sweep-64) |
| `old-baseline.txt`, `old-contended.txt`, `new-contended.txt` | Reproduction: old binary fails (`EXIT:1`, two accounting-failed lines) under a held lock; new binary retries and exits 0 |
| `state-tests.txt`, `core-lib-tests.txt`, `core-suite-tests.txt` | Test summaries. The three `policy_tests` failures in `core-lib-tests.txt` are the parallel-run failures also present at base; they pass with `--test-threads=1` |
| `functional/` | Three concurrent installed-binary `corbanu exec` runs per round (A, B across the UTC hour), all `EXIT:0` |

The review diff referenced in `review.md` was not moved (76 KB; reproducible with `git diff 21162588cd..79aef0daf3`).
