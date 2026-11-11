The record still needs changes. Two corrections are required: round 1's #3 was not fixed correctly, and one of the round-1 edits introduced a new error.

**Round-1 corrections that hold:**
- **#1, recommendation:** the README and Decision 1 now say "not yet" under the frozen rule, say what would settle it, and offer switching now as a departure for the owner. This matches `design/parity-method-utm.md`.
- **#2, MOD-22:** the README says Tart T18 was not replayed and the difference repeated on UTM.
- **#4, tester-only note:** it is disclosed in dispositions, the README and `results.json`. It matches `runs/T17-utm` at 08:40:54Z.
- **#5, #1139:** only item 2 is claimed to reproduce. Item 1's prompt is now described.
- **#6, MOD-26:** "withheld … in at least one step of every attempt" matches Tart's dispositions.
- **#7, transport header:** it now says "except T09-cur".
- **#8, harness:** the 22 files in `harness/` are byte-identical to Tart's, and Tart has only `seal-guest.sh` and `dns-refuse.py` beyond them. `harness-tart/seal-assets.sha256` matches Tart's copies and the current `scripts/qa/tart/` files, which were last changed in `a941bfb9eb`.
- **Other checks:**
  - Both freeze checks pass.
  - Since `04837d4f1a`, the only change to any frozen tree is `harness-tart/seal-assets.sha256`, which was added after the run and is disclosed.
  - Packet v4u differs from v4p only by the three-line bullet in all 26 prompts.
  - `package-files.txt` is identical to the Tart run's.
  - Every session-file SHA-256 in the excerpts appears in its run's `withheld-files.txt` (Tart ones under `../moderate-tart-parity-20261020/runs/T17*/`), and no session file is left out.
  - The UTM T17 route is supported. In screen `T24-send-task-highlight` the highlight is on "• Burzum [troll]", not on "Send task to Burzum"; Enter opened Burzum's pane, where the task was typed.
  - The numbers I re-checked hold: guard counts 4,993 / 1,202 / 852 with 0 `SECURITYAGENT-SEEN` seen; mediator requests 300 + 74 + 47 = 421; the relay's 423 includes 5 rate-limit (429) responses; latencies; memory 72–93% free; times; and the erase/restore outputs.

## Required corrections

1. **#1146: the session files show the route, not the label (round-1 #3 not resolved).**
   - **What the files show:** `tools/excerpt_tasks.py` finds `corbanu_untrusted_data` in no model input in any session file, including tool and agent outputs. That includes:
     - Tart T17 Snaga, file `3e90b10e…`. This is the session whose mailbox delivery the child reported as "wrapped in corbanu_untrusted_data (source=child_agent)" (Tart `runs/T17` screen `51-resubmission-text-entered`). Tart's own disposition rests on that report.
     - The T17-cur parent, file `ac28d6ca…`. Its `<subagent_notification>` input is agent output, which the product's own `/security` text says "reaches the model labelled untrusted" under Moderate.
   - **Why that matters:** the files do not record the label at all. So they cannot refute the child's report that the typed resubmission was "again wrapped (source=unknown)" (screen `53-resubmission-after-wait`). The record already concedes this in its own limit: "The session file records a mailbox delivery, not the wrapped text the model was shown."
   - **What is not supported:** the README's "the `corbanu_untrusted_data` label … follows the route … (product session files …)" and "… is not borne out by its session file". The same goes for dispositions' MOD-21 Tart paragraph, "What this shows about #1146", and the "no model input contains `corbanu_untrusted_data`" lines for UTM T17 and T17-cur, which read as evidence.
   - **Fix:** say the files show the route and its recorded origin (mailbox assignment versus a user message with origin human). Say they record no wrapper even where the child reported one, so whether the label follows the route is not shown. Present the Tart child's typed-resubmission report as a contrary case that is left open, not as one that was disproved.

2. **The round-1 edit to `common/effective-policy.txt` broke the hashes in all 9 isolation records.**
   - Every `runs/*/isolation-*.json` gives `common/effective-policy.txt` as `51bf826a…`. That was its hash at `f470787768`. The file is now `3e25511e…`.
   - Either regenerate the isolation records (and update the `results.json` hashes for them), or keep the change and add a disclosed erratum mapping the old hash to the new one. In both cases, record that the change was made after the run.

## Optional

- **Session-file check for D95:** the part 2 method has the integrator check "whether the child's first task input is wrapped" in the session files (`design/rerun-method-d94d95.md`). Correction 1 shows those files cannot show that. Decision 2's v8 re-run should name another way to confirm it, such as the child's own words or a product-side record.
- **`results.json`, part 2 MOD-11 note:**
  - It still says "each retried with the identical request". Dispositions were softened to "equal `request_bytes`".
  - It says ER 5 was "not exercised", while dispositions say "not observed".
- **The rate limits (429s):** the 429s at 10:00:59Z and 10:01:00Z were each followed by another 429; each run of same-size requests ended in a 200. One of the five, at 10:01:29Z, was the tester's request, not the product's. The wording should say this.
- **Seal assets:** `tmux` and `libevent_core-2.1.7.dylib` were also copied from host paths at run time, outside both freezes (`seal-assets.sha256`, `tart-run.sh` lines 61–62), but the README and policy name only two files.
  - The `tmux` hash matches Tart's `provision.out`.
  - The `libevent` hash does not appear in the Tart record.
- **Transport windows:** they overlap (start − 120 s), so the UTM per-window counts add up to 313 against 300 mediator requests. This is worth a line in `transport.json`'s note.

VERDICT: CHANGES
