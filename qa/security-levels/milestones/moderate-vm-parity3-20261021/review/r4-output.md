I found no required corrections. The round 3 optional point is applied correctly and matches the evidence, and nothing else in the records changed.

**The round 3 point (the latency note in the README's Environment table): applied, and it holds up.**
- **The edit:** `git diff HEAD~1 -- README.md` changes one line, the "Product request latency" row header. The old note was "Tart windows include the clone's gate requests". It now reads "the first window on each machine, every Tart clone's and UTM's T17-utm1, includes gate requests". The figures in that row didn't change.
- **T17-utm1 does include gate requests:** `runs/T17-utm1/mediator-window.jsonl` has four product-role entries:
  - a GLM 5.3 Flash POST at 10:54:03-0700 (5.01 s);
  - a GET at 10:54:04-0700 (0.35 s);
  - two GLM 5.2 POSTs (50.424 s and 65.865 s).
  
  The tool takes `secs[len(secs)//2]` (`tools/transport_check.py`), which gives 50.424. That matches `transport.json` for T17-utm1: 4 product requests, median 50.424. Without the gate requests the value would be 65.865, as round 3 said.
- **It is only UTM's first window:** T17-utm2 (3 product requests) and T17-utm3 (2) contain only GLM 5.2 POSTs. T18-utm1 contains only GLM 5.3 Flash POSTs, with no gate GET. So the "first window" wording is accurate.
- **The ranges still hold:** UTM T17 is still 36–160 s. The low end is 36.234 (T17-utm2) and the high end is 160.266 (T17-utm3's upper median).

**Nothing else changed.**
- `git diff HEAD~1 --stat` touches only `README.md` (1 line) and `review-ledger.md` (the new row 3), plus the review files (`r3-*` and `r4-brief.md`).
- Row 3 of `review-ledger.md` describes the change accurately.
- Nothing in `../moderate-amendment-v9-20261021/` changed.
- The only untracked file is `review/r4-start.txt`.

## Required corrections

None.

## Optional

None.

VERDICT: APPROVE