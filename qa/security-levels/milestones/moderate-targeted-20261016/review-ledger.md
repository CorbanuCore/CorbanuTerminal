# Review ledger (Moderate targeted re-run, qual-mod2)

Budget: 5 review/evidence rounds in total (brief). Extensions only via qual-mod2.request.md.

| # | Round | Agent | Started | Verdict |
| --- | --- | --- | --- | --- |
| 1 | Amendment v3 design (fresh read-only designer) | Opus 5.5 High, `corbanu exec -s read-only` | 2026-10-16T10:06:56Z (ended 10:14:44Z) | DESIGN: COMPLETE |
| 2 | Amendment v3 review (fresh read-only reviewer) | Opus 5.5 High, `corbanu exec -s read-only` | 2026-10-16T10:16:43Z (ended 10:21:24Z) | VERDICT: CHANGES (its final v3 frozen word for word; its Q09 note change applied; not re-reviewed) |
| 3 | Evidence check r1 (independent read-only) | Opus 5.5 High, `corbanu exec -s read-only`, on a copy of the record (check-r1) | 2026-10-16T13:19:20Z (ended 13:30:11Z) | VERDICT: fail (5 corrections, applied) |
| 4 | Evidence check r2 (independent read-only, fresh) | Opus 5.5 High, `corbanu exec -s read-only`, on a copy of the record (check-r2) | 2026-10-16T13:36:45Z (ended 13:45:43Z) | VERDICT: fail (5 corrections and 4 advisory items, applied) |
| 5 | Evidence check r3 (independent read-only, fresh; last in budget) | Opus 5.5 High, `corbanu exec -s read-only`, on a copy of the record (check-r3) | 2026-10-16T13:48:03Z: attempt 1 aborted at 13:55:16Z by a provider 429 (no verdict); attempt 2 2026-10-19T18:36:43Z (ended 18:49:43Z) | VERDICT: fail (4 corrections and advisory notes, applied; budget spent, not re-checked) |

Round 5's first attempt ended with no verdict when the provider returned 429 and set a ~20 h cooldown; it was retried
once, unchanged, on the same copy of the record (check-r3, commit `1a2fef0fe7`) after the cooldown, as the same round.
Its events are kept as `review/attempt1-r3-events.jsonl.gz`. The budget is spent; the r3 corrections are applied but
not re-checked.
