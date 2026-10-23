# Review ledger: PF-60-S10

Budget: 5 rounds in total for design and evidence checks (pf60-s10 brief; wave2-common REVIEW BUDGET rule: an extension
needs Travis's or the coordinator's authorization through `pf60-s10.request.md`; none self-authorized).

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 1 | Code-blind design (fresh designer) from the intent packet | `01a1428c-9592-7652-8620-53768ab043ba` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, packet directory only | 34 cases, 15 questions ([design/designer-output.md](design/designer-output.md)) |
| 2 | Independent design review against intent, owner decisions and harness facts | `01a14298-8fd0-7381-aa6f-d4a5ee26429d` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, review packet only | **CHANGES**, 22 amendments (3 new cases), applied as written, not re-reviewed ([design/review-output.md](design/review-output.md)) |

Budget at the freeze: 2 of 5 used; 3 remain for independent evidence checks.

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 3 | Independent evidence check r1 (fresh session) | `01a143c5-c348-7703-97a2-584f6e0a1850` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, record copy only (`git archive` of commit 7f8b517b57) | **fail**: totals, freeze, packets, replays, receipts and isolation confirmed; corrections: ACC-15's reason (the `side:` line is in the raw terminal output), ACC-18 has two residuals (one left by ACC-14's deviation), ACC-36's unmet ALPHA step, amended priorities (ACC-17, 34, 35), make_packet.py drift; plus gaps (provisioning runs, 443 rule scope, request count, spend scope, record key-scan output, sprint list). All corrected ([r1](review/evidence-check-r1.md), [addendum](common/effective-policy-addendum.txt)) |
| 4 | Independent evidence check r2 (fresh session) | `01a143d0-4bd6-7832-a64d-4628800df189` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit 13bd70ed7b) | **fail**: every r1 correction confirmed; isolation, freeze and totals confirmed; corrections: ACC-11, ACC-14 and ACC-18 claimed unchanged/unique Request IDs no screen shows (and ACC-18's per-conversation sums only for ALPHA) — now blocked by the same rule as ACC-15 (32/1/4 -> 29/1/7); ACC-17's "listed once" rests on aggregates (stated); ACC-05 and ACC-15 wording softened; README limits (ID gap, harness change during F, the guest's Z.AI key file); key-scan count re-run. Mechanical change after this check's snapshot: large JSONL gzipped and agms-probe stored as .txt for CI ([r2](review/evidence-check-r2.md)) |
| 5 | Independent evidence check r3 (fresh session), last within budget | `01a143db-ee78-7c93-95d6-e7807ca20735` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit 2cbc512f7a) | **fail**, on written statements only: every disposition, the 29/1/7 totals, the freeze, the replays, receipts and every r2 correction confirmed. Corrections: harness fix 1 was installed before F's first product start (not "06:2xZ"), and its check ran on lane agmx0 concurrently with F; packet v2 was installed while D ran; failed gate attempts 1 and 2 each sent a real Z.AI smoke turn; the README's S08 bullet was misleading; the key-scan count included 16 git-ignored files. All corrected afterwards ([addendum](common/effective-policy-addendum.txt), README, [keyscan-record.txt](keyscan-record.txt)), **not independently verified** ([r3](review/evidence-check-r3.md)) |

Budget: 5 of 5 used. No extension requested: the last check found statement corrections, no disposition change and no
major defect (binding rule of 2026-10-14). The last independent verdict is **fail**; its corrections are applied but
not re-checked.
