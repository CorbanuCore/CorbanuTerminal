# Why the accounting gate failed en masse on 20 September, and what it actually proves

Recorded during recovery from a host crash. Intermediate record, committed while
the comparison runs rather than after, because the crash already cost one run.

## The host, not the change

The machine rebooted at 13:59 local. Afterwards macOS re-indexed both volumes:
18 `mdworker` processes, `mds` at 65% CPU, `disk0` at 11,437 transfers per
second in 4 KiB units, load average near 8 with a single test process of mine
running. `mobileassetd`, `deleted` and `backupd-helper` were each burning 60-85%
CPU at the same time. Time Machine itself was not running.

The accounting integration tests carry internal deadlines of roughly 12-18
seconds. Under that storm they expire before the work completes, and the failure
prints `Error: deadline has elapsed` with no assertion — 86 occurrences in one
lane. A cold `cargo build` in a fresh target directory also stalled at 0% CPU for
twenty minutes before I killed it.

`/Volumes/CorbanuDrive/.metadata_never_index` now exists, which stops Spotlight
from indexing the work volume without needing privileges. Worker count fell from
18 to 11 and `disk0` from 11,437 to 2,808 transfers per second within two
minutes. Making it immediate needs `sudo mdutil -i off /Volumes/CorbanuDrive`.

## The comparison, which is the only thing that settles attribution

Identical lanes, same host, same warm target directory, same
`NEXTEST_TEST_THREADS=4`, candidate `2e6d47e17` against base `6322a6e7c`.

| lane | candidate | base |
| --- | --- | --- |
| `codex-core` accounting | 129 run, 81 passed, 48 failed | 124 run, 62 passed, **62 failed** |
| `codex-core` accounting, `developer-accounting` | 133 run, 64 passed, 69 failed | not run at base; the feature does not exist there |
| `codex-state` accounting | 166 run, 161 passed, 5 failed | running |
| `codex-tui` usage | 92 run, **92 passed** | not needed |
| `codex-tui` tokens | 66 run, 65 passed, 1 failed | running |

The base fails *more* core tests than the candidate. The core failures are
therefore environmental and carry no information about the change. They are not
evidence that the change is sound either — they are evidence of nothing, which is
why the state and tui failures below are the ones that matter.

## The two failures that are real

1. **`codex-state`, 5 failures, all assertions rather than deadlines.** The
   change makes a zero-count bucket price as zero only when a rate snapshot was
   actually selected, otherwise `MissingRate`. That is the intended and more
   honest semantics: an absent price must not render as a complete zero estimate.
   The goldens in `accounting_late_import_tests`, `accounting_latest_quote_tests`,
   `accounting_lifecycle_tests`, `accounting_retention_atomic_tests` and
   `state/tests/accounting_store.rs` still encode the old behaviour and were not
   updated. Each needs deciding on its merits, not blanket acceptance.

2. **`codex-tui`, `accounting_inspect_maintenance_with_healthy_raw_renders_lag`.**
   The page used to open with `Snapshot is not current; newer activity is
   unverified`. It now opens with an estimate block — `Estimated token cost:
   unknown`, `Known estimated token cost: $0.000000 + unknown costs`, `Full
   recorded estimate: unavailable (1 of 1 attempts incomplete)` — and the lag
   warning no longer leads. This is the shape this lane exists to prevent: a
   freshness warning displaced by a number. It must be resolved deliberately,
   either by restoring the warning's precedence or by proving the new order tells
   the operator the truth.

Nothing here is a receipt. The change stays unreceived until both are closed and
an independent review has looked at them.
