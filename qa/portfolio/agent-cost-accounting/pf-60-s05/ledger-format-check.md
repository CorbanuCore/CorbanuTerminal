# PF-60-S05 AC6: ledger written before 2026-09-21, and the ledger format

*2026-10-09. Review Major 3 ([body review](../pf-60-s03/body-review-20261008/review.md)).*

## Reproduced first

- **Fixture.** A one-off test against `codex-state` at `3b6c338f8f` (`5bae03414e^`, the last commit before price records
  gained a basis) wrote a ledger with three attempts: priced and complete, priced with incomplete usage, and unpriced.
  The generator is kept beside this note as `pre-5bae-fixture-generator.rs.txt`; it ran with
  `ACCT_FIXTURE_OUT=<dir> cargo test -p codex-state --test acct_fixture_gen` in a detached worktree at that commit.
  The accounting objects were dumped with `sqlite3 <db> ".dump '%accounting%'"` into
  `codex-rs/state/tests/fixtures/accounting/pre-5bae-ledger.sql`.
- **Main fails on it.** `ledger_written_before_plan_basis_validates_and_reads` (state/tests/accounting_store.rs), run on
  main `8ec1aee7ab` before the fix: `Error: noncanonical snapshot`. The snapshot written without `basis` and
  `plan_burn_millis` no longer re-serialized to its stored bytes, so the whole ledger stopped validating. Major 3 is
  real.

## Real old ledgers

- **Searched:** every `state_*.sqlite` under `/Volumes/CorbanuDrive/Corbanu` (`.codex-work`, `worktrees`, `qa`,
  `tmp`), `~/.codex`, `~/.corbanu`, `~/.pfterminal`, `/tmp` and the macOS temp folders on the Mac, and every
  `*state_5.sqlite` under the RTX box's home.
- **No ledger from before 2026-09-21 exists.** Developer-accounting test homes are temporary and were deleted. The two
  oldest real ledgers are on the RTX box: `corbanu-rtx/home` (177 attempts, 2026-09-23 to 2026-10-03) and
  `corbanu-rtx/fresh` (11 attempts, 2026-09-23).
- **Both validate and read on the new build.** Copies were checked with the ignored helper
  `real_ledger_validates_and_reads` (`ACCT_REAL_LEDGER=<copy> cargo test -p codex-state --test accounting_store
  real_ledger -- --ignored`): 101 and 6 thread-days read before and after a validating write-open with identical
  totals and requests, and the ledgers stayed in format 1.

## The format

- **Format = applied accounting migrations.** Format 1 is the original schema. Format 2 (migration
  `0002_ledger_format.sql`) lets a price record state a `Local` or `Undeclared` basis or a basis from the user's config,
  and a compact day count that work.
- **Older forms are read as written.** Both stored forms of snapshots and estimates are accepted: today's, and the
  pre-09-21 form, only for records whose omitted fields hold their defaults.
- **Upgrade only when needed.** A new ledger starts in format 1 and is upgraded in the same transaction that first
  writes a record format 1 cannot express (admission, observation or late import). An older developer build sharing
  the state DB keeps working until then. After it, that older build has no format gate: it fails to open the ledger
  and, since #299, sends its requests unrecorded with a gap warning on every turn. Release builds contain no
  accounting and are unaffected. A developer ledger opened by the first, eager-upgrade commit of this slice
  (`03bf8e6705`, never merged) is already format 2.
- **Newer format refused by name.** A ledger with a migration this build doesn't know is refused with
  `NewerLedgerFormat` for reads and writes and left untouched. Core turns collection off on it with one warning
  (slice 3).
- **Tests:** the fixture validates, reads, keeps its format on a write-open, takes a new usage report (mixing legacy
  and current records), upgrades when an undeclared-basis record is written, and reads identically after retention
  compacts the day 100 days later; a newer format is refused for read and write with the ledger unchanged; a ledger
  claiming format 1 while holding format 2's table is refused.
