# Review ledger (Moderate second targeted re-run, qual-mod3)

Budget: 5 review/evidence rounds in total (brief: "one evidence-check budget of 5 rounds", counted as in qual-mod2:
design, design review and evidence checks). Extensions only via qual-mod3.request.md.

| # | Round | Agent | Started | Verdict |
| --- | --- | --- | --- | --- |
| 1 | Amendment v4 design (fresh read-only designer) | Opus 5.5 High, `corbanu exec -s read-only` | 2026-10-20T03:30:42Z (ended 03:34:22Z) | DESIGN: COMPLETE |
| 2 | Amendment v4 review (fresh read-only reviewer) | Opus 5.5 High, `corbanu exec -s read-only` | 2026-10-20T03:35:52Z (ended 03:38:53Z) | VERDICT: CHANGES (its final v4 frozen word for word; not re-reviewed) |
| 3 | Evidence check r1 (independent read-only) | Opus 5.5 High, `corbanu exec -s read-only`, on a copy of the record (check-r1) | 2026-10-20T04:51:23Z (ended 04:59:17Z) | VERDICT: fail (3 corrections and 2 recommendations, applied) |
| 4 | Evidence check r2 (independent read-only, fresh) | Opus 5.5 High, `corbanu exec -s read-only`, on a copy of the record (check-r2) | 2026-10-20T05:02:07Z (ended 05:11:46Z) | VERDICT: fail (3 corrections and 3 recommendations, applied) |
| 5 | Evidence check r3 (independent read-only, fresh; last in budget) | Opus 5.5 High, `corbanu exec -s read-only`, on a copy of the record (check-r3) | 2026-10-20T05:16:07Z (ended 05:24:48Z) | VERDICT: fail (1 correction and 5 recommendations; 1 correction and 4 recommendations applied, the fifth already disclosed; budget spent, not re-checked) |

A first designer launch at 2026-10-20T03:26:19Z (background job) died before sending any request (empty events file, no output); it is not counted as a round.

All three evidence checks found every disposition and the 34-case totals supported; each failed the record on written
claims, corrected after it. The budget is spent; the r3 corrections are applied but not re-checked.
