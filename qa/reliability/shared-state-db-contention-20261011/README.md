# Shared state DB contention (#359, #324, #299 follow-up), 2026-10-11

Seven or eight `corbanu exec` sessions on one home (578 MB `pfterminal_state_5.sqlite`,
61,042 ledger attempts) logged `developer accounting failed step="open sampling" ... after 2
contended retries over 15874 ms: ... database is locked`, the "shared state database is busy"
note, failed thread-metadata writes and, in the TUI, "failed to check provider request throttle
state". Every model request could lose 15-30 s.

## What held the lock

Measured with `codex_state::db_lock` (new: every `BEGIN IMMEDIATE` logs its wait and hold with
its call site; a hold over 250 ms is a warning) on a `.backup` copy of the live DB, never the live
file.

| Holder | Before | Why |
| --- | --- | --- |
| Every ledger write (`accounting_store.rs` `write`, `open`) | 0.3-0.9 s each, up to 11 s, 4-6 per request | `check_day` and the retention check found the owner's day and the due expiries with `json_extract` over all 61k attempt payloads, under the lock. A debug build also ran SQLite's C code unoptimized (3-4x slower). |
| Each process's first write of every UTC hour | 75 s of CPU before the request (debug) | Whole-ledger validation; every process did it. |
| Conversation delete | full retention sweep per owner, twice with the #308 preflight | #324 |
| Thread metadata, throttle | waited SQLite's 5 s busy timeout (throttle: up to 10 s) behind the above | #359, #351 |

## Fix (see the PR)

- Ledger lookups use two expression indexes on `draft_accounting_attempts` (owner+dispatch,
  dispatch; named `corbanu_lookup_*`, outside the ledger's namespace), created idempotently by the
  writer, not by a ledger migration, so older builds keep reading and writing the ledger. The
  retention check and the expiry batch put their arithmetic on the bound so the index answers them.
  On the 578 MB copy: 154-240 ms under the lock after a lock-free warm-up read; the DB grew 4 MB.
  An older binary (`corbanu-before`) then ran a turn and recorded normally on that DB.
- `libsqlite3-sys` at opt-level 3 and `codex-state`, `sqlx-*`, `serde_json` at opt-level 2 in dev
  builds (developer accounting is debug-only): the hourly validation went from 75 s to 28-29 s.
- Per-process ledger writer (`state/src/runtime/accounting_writer.rs`): a request saves its
  record under `accounting-pending/`, hands it over and waits only while the write meets no lock
  (500 ms short-wait pool, 1 s cap) and nothing is queued ahead; otherwise it goes on and the
  record is written later, never dropped for contention or a failing disk. Records of exited
  processes are claimed and written by the next writer. One process at a time runs the hour's
  validation (file lease). `exec` waits at exit only while its writer makes progress. A record
  the ledger refuses after its request stopped waiting is reported by the conversation's next turn.
- Thread metadata (#359): best-effort updates wait at most 500 ms, are skipped while the DB is
  known busy, and are applied with the thread's next update or at shutdown.
- Throttle (#351): steps wait about 1 s instead of 10 s; adopting legacy keys tries once (500 ms).
- Delete (#324): the hour's validation and due expiries run before the lock; one bounded
  maintenance for all owners instead of a full sweep per owner.

The ledger stays in the state DB. Splitting it out would separate the locks, but conversation
deletion must remove a conversation's rows and its ledger records together, every ledger write
checks the owner's `threads` row, the 540 MB would have to move out of the live DB, and older
builds sharing the home would keep writing the old location. With the indexes a ledger write holds
the lock for milliseconds, so the split is not needed.

## Load test: `load.py`

`load.py --bin <corbanu> --db <copy> --out <dir> --sessions 10 --turns 3 --tool-calls 4 --warmup
[--hold-seconds 20]` runs 10 concurrent `corbanu exec` sessions (3 turns each, 5 model requests
per turn) on one disposable home holding the DB copy, against `mock_anthropic.py`, with developer
accounting on. `--warmup` runs one exec first so the home's other runtime DBs exist and the hour's
validation is done, as on a home in use (ten simultaneous first launches on a brand-new home race
creating `pfterminal_goals_1`/`memories_1`; a separate, pre-existing bug seen with both builds, #549).
`--hold-seconds` holds the write lock from another process 5 s into the run, as a long
maintenance or a backup would. Reports are in `evidence/`. The Mac was shared with other builds
(load average 11-45), so absolute times are noisy.

| N=10, warm home | Before (main) | After | After, lock held 20 s by another process |
| --- | --- | --- | --- |
| Tool response to next request, p50 / p99 / max | 20.1 s / 40.9 s / 49.9 s | 101 ms / 216 ms / 239 ms | 85 ms / 571 ms / 573 ms |
| Lock wait p50 / p99 / max | 245 ms / 5.17 s / 5.46 s | 0 / 101 ms / 178 ms | 1 ms / 5.4 s / 5.4 s (\*) |
| Lock hold p99 / max | 1.12 s / 2.99 s | 16 ms / 90 ms | 18 ms / 82 ms |
| Holds > 250 ms | 469, all ledger writes | 0 | 0 |
| Requests in the ledger | 147 of 150 (5 sent unrecorded) | 150 of 150 | 150 of 150 |
| busy/locked/failed stderr lines | 144 (256 "database is locked") | 0 | 29 (throttle fallbacks, retries) |
| Thread-metadata writes failed | 30 | 0 | 10, at shutdown (\*\*) |
| Failed execs | 0 | 0 | 0 |
| `exec` duration p50 / max | 114 s / 168 s | 3.6 s / 4.0 s | 4.7 s / 19 s |

(\*) Waits over 1 s with the lock held are all the background ledger writer's retries
(`accounting_store.rs` open), not requests. (\*\*) A thread's put-off metadata is written once
more when its writer shuts down, waiting the normal 5 s; with the lock held through that, it is
logged and left to the rollout (the canonical record; listing reconciles it).

Cold hour, no warm-up (validation due), main: p99 gap 30.8 s, lock wait p99 5.24 s, 27 of 150
requests unrecorded (`run-before-n10.json`). On this branch the validating `exec` spends
28-29 s at exit, once per hour per home, while the others are not held up.

`cargo test -p codex-state ten_sessions_on_one_home_wait_briefly_and_lose_no_record` is the CI
version: ten runtimes (as ten processes have) on one home make requests (throttle check,
admission, metadata write, usage) while another connection holds the write lock for 3 s; it
asserts p99 < 1.5 s, max < 2.5 s, no failure, and every record in the ledger.

## Functional check: `glm_tmux_check.sh`

Eight interactive TUI sessions in tmux on one disposable home with a copy of the live DB,
`-m glm-5.3-flash -c model_provider="zai"`, developer accounting on,
`CORBANU_TEST_NO_NATIVE_KEYRING=1`; each session fetches the key from the vault inline (never
printed or exported). Each ran three shell tool calls and answered. All eight finished
(`evidence/glm-8-pane-s1.txt`); no busy, locked or accounting lines in any pane or in the home's
log DB; the ledger has 32 attempts with 32 observations across the 8 conversations and nothing left
pending (one session validated the ledger for the hour meanwhile).

An earlier run of the script closed the sessions before that validation finished: their 60
records stayed saved, and the next process on the home (one `exec`) claimed and wrote all of them.
In another earlier run one TUI exited at start, before its pane could be kept, with nothing in the
home's log DB; its other seven sessions finished cleanly. The script now keeps exited panes; no
later run lost a session.
