# Opus 5.5 High vs Astra High — model evaluation, 2026-11-14

Status: **version 1 (progress)**. Track A is scored. Track B (paired implementation + blind grading) is running and will be added in a later commit.
The readable report with charts is `docs/management/astraVsOpus.html`.

Owner request (Travis, 2026-11-14): compare Claude Opus 5.5 High with Astra High (`gpt-6-astra`, reasoning high) on our real work, to decide when Astra may be used to increase throughput.

## Track A: review benchmark (8 known defects + 2 clean controls)

The answer key (`trackA/key.md`, sha256 in `trackA/key.sha256`) was written and frozen before any model ran. Each model reviewed each defective head read-only, in a clone with no later history and no network, using an identical neutral brief (`trackA/briefs/`).

| Case | PR | Class | Opus 5.5 High | Astra High |
|---|---|---|---|---|
| A01 | #1837 | security classifier | missed · REQ | **found** · REQ |
| A02 | #1962 | test infra (dependency) | missed · APPROVE ✗ | missed · APPROVE ✗ |
| A03 | #1055 | classifier instrumentation | missed · APPROVE ✗ | **found** · REQ |
| A04 | #2043 | sync conflict (security) | **found** · REQ | **found** · REQ |
| A05 | #2133 | security classifier | partly (1 of 2) · REQ | **found** (both) · REQ |
| A06 | #1171 | logic / message | partly · REQ | missed · REQ |
| A07 | #1175 | agent control (TUI) | **found** (+ more secondaries) · REQ | **found** · REQ |
| A08 | #1889 | Windows test quality | **found** · REQ | **found** · REQ |
| A09 | #1878 | control | APPROVE ✓ | APPROVE ✓ |
| A10 | #1944 | control | APPROVE ✓ | REQUEST_CHANGES ✗ (low-impact point) |

| Metric | Opus 5.5 High | Astra High |
|---|---|---|
| Known defects (full = 1, partial = ½) | 4.0 / 8 | 6.0 / 8 |
| Correct verdict | 8 / 10 | 8 / 10 |
| Real extra findings / uncertain / false | 16 / 7 / 0 | 8 / 6 / 1 (A10 verdict) |
| Wall time (10 reviews) | 27 min | 29 min |
| Tokens (10 reviews) | 969k | 877k |

Per-case reasoning: `trackA/scoring-notes.md`. Raw review outputs: `trackA/reviews/`.

Compliance: no arm ran a GitHub-write or push command (`compliance.txt`).

## Caveats (version 1)

- n is small (8 + 2, single run per model per case).
- The defects in A01, A03 and A05 were originally found by Astra High during the 2026-11-11 contributor integration; A04, A06, A07 and A08 by Opus. This can favour each model on "its" cases.
- Reviews were static (no builds/tests). A02 required knowledge of assert_cmd 2.2 that was not available offline.
