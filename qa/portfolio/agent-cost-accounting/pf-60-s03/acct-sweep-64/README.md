# acct-sweep-64: throttle-check contention and incremental day-90 expiry (2026-10-03)

Moved from `.codex-work/workers-20261002/acct-sweep-64.*` and `.codex-work/acct64/` on 2026-10-08.
Logs renamed `.log` -> `.txt` (the repo ignores `*.log`).

Commits (on main via PR #138, merge `3ecc3c4066`): `d56cdee121`, `4c748fe5e4`.

| File | What it is |
|---|---|
| `BRIEF.md` | Worker brief |
| `RESULT.md` | Worker result: changes, day-90 measurements, tests, review disposition, installs, open items |
| `review-brief.md`, `review.md` | Independent review brief and verdict (approve with nits; both must-fix Minors fixed) |
| `core-tests.txt` | Core test summaries |
| `smoke/` | Three concurrent installed-binary `corbanu exec` runs, all `EXIT:0` |

Open from it (carried in the S03 Remaining list): thread deletion still runs the full locked sweep (~9 s on the live
ledger); no core test for an interrupt during a lease write; no test for two processes expiring at once.
