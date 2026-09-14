# Supervised functional execution and recovery rehearsal — September 14

PF-80-S01, **Internal delivery control — TO BUILD**, “Use sequential sprints per
initiative.” Fable takeover manager executed the frozen DEC cases through the
private code-blind harness and rehearsed stalled/crashed-worker recovery with real
TMUX workers. Authority: Travis 2026-09-14 (supervised qualification; unattended
recurrence approved pending qualification; `--yolo` for non-test workers).

## Harness lineage (private repo `.codex-work/functional-execution.ikteKU`)

`db4211f` (DEC-001 only) → `7ce49a7` provenance receipts (review P2) → `ebe0bd7`
(corrective review clean) → `4713c31` fresh attested fixture from a detached
candidate worktree at `c77123a7e9` → `00ef50f` multi-case + attested same-browser
phases (review correct; DEC-021 P2 → unmapped) → `2761743`/`3ffeac0` manager fixes
found by execution (write-once receipt collisions → checkpoint files; control venv
interpreter) → `5328d68` Docker symlink-caching pointer fix and renderer cap
(review correct, P2 pids margin 116–123/128) → `66f9095` step-budget awareness,
prior-phase context, scroll repaint wait, harness hash pins (review correct, P3)
→ `f0badcd`/`714cb08` per-case budgets, then reverted above the pinned guest's
~20-action budget (recorded executor limit). Pre-container failures are retained
under `retained-failures/`.

## DEC results (candidate `c77123a7e9`, code-blind `gpt-6-astra` actor, pinned isolated transport)

| Case | Executor verdict | Attempt | Independent evidence review |
| --- | --- | --- | --- |
| DEC-001 | passed | `attempt-waid23v9` | supported with limits (review 01a09eab/evidence exec) |
| DEC-002 | passed | `attempt-13v51xqk` | supported |
| DEC-004 | passed | `attempt-109y9ih4` | supported with limits |
| DEC-005 | passed | `attempt-une1dlx8` | supported with limits |
| DEC-006 | passed | `attempt-0gjbrd8x` | supported |
| DEC-007 | passed | `attempt-4zgbl4i8` | supported |
| DEC-008 | passed | `attempt-xkyyg2g0` | supported |
| DEC-009 | passed | `attempt-ghjsdej5` | supported |
| DEC-010 | passed | `attempt-xn467o5k` | supported with limits |
| DEC-011 | passed | `attempt-drev8y5u` | pending (batch 3: DEC-011/012/019 not yet reviewed) |
| DEC-012 | passed | `attempt-yblstvdu` | pending (batch 3: DEC-011/012/019 not yet reviewed) |
| DEC-013 | passed | `attempt-g5cr9nka` | supported with limits |
| DEC-014 | passed | `attempt-y28ifm_9` | supported with limits |
| DEC-015 | passed | `attempt-awu39enk` | supported |
| DEC-016 | passed | `attempt-v7h4dcaz` | supported |
| DEC-017 | passed | `attempt-dlqq6pdt` | supported with limits |
| DEC-018 | passed | `attempt-7mki3iji` | supported |
| DEC-019 | passed | `attempt-r9_oo5uf` | pending (batch 3: DEC-011/012/019 not yet reviewed) |
| DEC-020 | blocked (executor budget; browser crash at step 28 under 36-step trial) | `attempt-0faqldw8` | pending (batch 3: DEC-011/012/019 not yet reviewed) |
| DEC-022 | blocked (executor budget) | `attempt-lx8oxcil` | pending (batch 3: DEC-011/012/019 not yet reviewed) |
| DEC-023 | blocked (executor budget) | `attempt-_h4512tb` | pending (batch 3: DEC-011/012/019 not yet reviewed) |
| DEC-024 | blocked (executor budget) | `attempt-swgaeonc` | pending (batch 3: DEC-011/012/019 not yet reviewed) |
| DEC-025 | failed (product finding) | `attempt-61f5jo95` | supported with limits (genuine product/UX finding, amplified by long fixture) |
| DEC-003 | unmapped | — | not executable at candidate c77123a7e9 (see cases.json unmapped_reason) |
| DEC-021 | unmapped | — | not executable at candidate c77123a7e9 (see cases.json unmapped_reason) |
| DEC-026 | unmapped | — | not executable at candidate c77123a7e9 (see cases.json unmapped_reason) |

Counts: 17 supported by independent evidence review (DEC-011/012/019 passes await
their batch review), 4 blocked by the executor's pinned action budget, 1 product
finding (DEC-025), 3 unmapped. Evidence reviews: batch 1 (12 cases), batch 2
(DEC-013/015/025) and the DEC-001 replay review are retained privately with SHA-256
in the coordinator events `evidence-review:dec-batch1`, `evidence-review:dec-batch2`
and `dec001-replay:attempt-waid23v9`.

Cross-cutting limits (from the reviews): verdicts may cite rendered text not shown
in frames (`image_observation: supplied_not_attested`); post-scroll repaint was
fixed only from `66f9095`; “same browser” rests on an identity tuple; the pinned
guest's action budget caps long/keyboard/phone cases; DEC-020 crashed the
sandboxed browser at step 28 in a 36-step trial (limits were not raised).

## Product finding retained

**DEC-025** (`attempt-61f5jo95`): expanding one long decision fills the 1440×900
viewport so the other open decisions cannot be tracked while inspecting it. The
independent reviewer judged this a genuine product/UX finding amplified by the
synthetic `long` fixture, not an actor error. Not fixed; recorded for PF-80 backlog.

## Restart / stall / crash rehearsal (real TMUX workers, coordinator revisions 535–542)

- **Complete handoff**: demonstrated repeatedly today (claim → tmux launch with
  private home/socket → exact allocation-digest ACK → START → RETURN → git
  verification → `returned`/`verify`), e.g. `responses-dispatch-impl-02`,
  `harness-navigation-fix-01`.
- **Crashed worker** (`rehearsal-crash-worker-01`, timeout 120 s): real worker
  ACKed, started, wrote its output, then the manager killed its tmux server
  (11:09:23Z). After the deadline `watchdog` reported `stall:…:0`; owner inspection
  found no tmux server/processes and no RETURN; `reconcile_dispatch` recorded
  `failed` with the inspection evidence and the partial output retained. No relaunch.
- **Uncertain dispatch** (`rehearsal-uncertain-dispatch-01`, timeout 60 s): claimed,
  then a simulated host crash before launch. `watchdog` moved it to
  `dispatch_uncertain`; inspection found no session/output; `reconcile_dispatch`
  recorded `failed`. No duplicate launch.
- **Long-lived receiver**: `slack-receiver-02` crossed its 14400 s threshold and was
  flagged; inspection showed it alive and idle, so it was kept (timeout is a
  threshold, not proof of death) and the inspection recorded as an event.
- **Manager restart**: every fresh Fable decision cycle today started from durable
  SQLite state only (13 cycles, three `briefing_size_hold` runs failed explicitly
  and were reconciled); the briefing compaction (`da456599d`, `82dda1ad2`) is what
  makes that sustainable.

Not yet qualified: an unattended owner (no tool-enabled human-started session).
That requires the owner daemon designed in `owner-daemon-design-01`; the paused
Desktop schedules alone cannot dispatch or reconcile workers.
