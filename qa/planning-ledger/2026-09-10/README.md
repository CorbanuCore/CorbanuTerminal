# September 10 ledger repair evidence

Routine planning repair based on main `0d1c3240d752335cc89faeb2149c5030063e5d69`,
branch `docs/repair-sprint-ledger-20260910`. The user requested fixing the
reported errors and merging again. The [identity map](identity-map.json) records
five P0 renames to resolve seven inherited checker errors and the same
feature-ID ambiguity on sibling records. See the
[human-readable reconciliation](../../../docs/sprints/identity-reconciliation-2026-09-10.md).

## Final-tree checks

All commands and outputs are recorded in [results.json](results.json).

| Check | Result |
| --- | --- |
| Global sprint validator, unchanged | PASS; zero errors, 112 current / 121 archived |
| Global plan validator, unchanged | PASS; two active plans, zero available slots |
| Fixed-baseline identity/integrity audit | PASS; 233 original sprints compared, five renumbered |
| Sprint checker regression suite | PASS; 22 tests, including three new collision/link/filename regressions |
| Plan checker regression suite | PASS; four tests |
| Portfolio validation with 52 expected drafts | PASS; 16 proposals / 52 unallocated drafts |
| Ruby YAML parser, MkDocs navigation | PASS |
| Staged whitespace check | PASS |

The initial optional Python YAML probe lacked PyYAML; the existing Ruby YAML
parser was used successfully without installing dependencies. This was an
environment limitation, not a documentation parse failure.

## Independent review

Command: `python3 /Users/Neo/.codex/skills/autoreview/scripts/autoreview --mode local --engine codex --codex-bin <private-wrapper> --model claude-fable-5-1-plan --thinking high --no-web-search --stream-engine-output --prompt-file qa/planning-ledger/2026-09-10/review-scope.md --output <private-review.txt> --json-output <private-review.json>`.

Run once through Corbanu in a task-owned tmux session, with launch text and
Enter sent separately. Requested Fable 5.1 high; private transport trace confirmed
provider model `claude-fable-5-1`. The helper exited 0 with `findings: []`.
No accepted/rejected actionable findings or review-triggered patch cycles.
The reviewer independently ran both global checks and the integrity audit,
checked P0 counts/backlinks and matched all 19 frozen file hashes.

[Review scope](review-scope.md) and [manifest](review-manifest.json) identify the
reviewed content; this receipt and results are post-review evidence only.
The full private run is
`/Volumes/CorbanuDrive/Corbanu/.codex-work/ledger-review.NiqooR/`.
Review executable SHA-256:
`950d42af0ca423ecf24bdde0101363008406776e1f9753136e66fca088ffd7e7`.
This is a review engine, not a qualified product candidate; its source build
commit is not independently established here.

## Preserved boundaries

PF-27-S04 and PF-35-S01 retain their existing reservations. Claude-auth records
are byte-for-byte unchanged. Five P0 records retain all original metadata except
canonical identity/update dates and preserve checkbox evidence. Two historical
release files change link destinations only. PF-77-S01's natural production
observation stays unchecked and its draft status is unchanged.

No runtime/config/API changes, product activation, server deployment, Task Node
writes, additional agents, or human acceptance are claimed. True-TUI, live
repository qualification and benchmarks are not applicable to this docs-only
repair and were not rerun.

The fixed-baseline audit is one-time repair proof, not a permanent handoff gate:
future unrelated commits can intentionally make its scope assertion fail.
Use the global validators for ongoing work. The reviewer noted this distinction
as non-blocking; the reconciliation page already disclaims ongoing policy.
Old portfolio seven-error snapshot receipts remain historical and unchanged.
