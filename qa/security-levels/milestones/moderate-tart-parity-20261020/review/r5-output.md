**VERDICT: CHANGES.** Two required corrections remain, both in `README.md`. Every disposition, parity class, attribution, total and parity-rule statement is right, and so is the Recommendation's factual basis.

## Round 4's corrections

| # | Correction | Status |
|---|---|---|
| 1 | MOD-21 reason and delivery path | Applied, but the README's #1146 summary was not updated (required correction 2 below). I checked the new wording against T17 `actions.jsonl` (16:27:30–54Z through Send task, 16:28:09–27Z in Snaga's view), the screens (49/50: the wrapper and the secret-handling policy; 51: `source=child_agent`; 53: `source=unknown`), the withheld hash `3e90b10e…bdc298a117.jsonl`, and the post-cap replay's `result.md`: no wrapper reported; item 1 refused citing Moderate, items 2–3 as secrets. |
| 2 | "All 34 baselines are older" | Applied. `git merge-base --is-ancestor` confirms `df3d8c5d1c`, `4ef3be8467` and `8bf182de43` are all ancestors of `8c73c8ea80`. |
| 3 | MOD-11 tie-break `reproduces` field | Applied. It now says "UTM for items 1-2; the Tart outcome for item 8 and the final check". |
| 4 | Round 3's optional items | Applied. The C-c session loss is cited, and the hashes for MOD-07 (`7e993fb5…`, `withheld-files.txt:261`) and MOD-11 (T09, the replay and the tie-break) match their files. The MOD-34 "inconclusive" note matches `T26 result.md:71`. |
| Opt. | MOD-22 case-text gap, counter-examples, explicit rule 3, item 1 citing Moderate, `denied_path` | All applied. The relay log has 1,746 `forwarded/200`, 1 `started` and 1 `denied_path` at 11:07:30Z. The shakedown mediator starts at 11:09:55Z. |

## End-to-end check

- **Totals:** 23 passed / 2 accepted / 8 blocked / 1 failed. Blockers are 18/1/6/1, advisory 5/1/2, parity 25 matched / 9 differs. `results.json`, `tools/tart-dispositions.json`, `dispositions.md` and `parity-table.md` agree on every disposition, summary, class and attribution.
- **Hashes:** all result and isolation hashes match their files. `FROZEN.sha256` lists 206 files and all verify.
- **Isolation numbers:**
  - 34 clones;
  - 1,741 forwarded requests, 0 non-200, 0 `session_error`;
  - one tunnel connect per clone and 0 ssh exits;
  - 29,051 guard checks and 0 `SECURITYAGENT-SEEN`.
- **MOD-11:** UTM G09's model declined item 8 with no tool call (`G09 result.md:35`). The tie-break on Tart escalated items 1–2 with Moderate named (`:42`, `:47`), and item 8 and the final check passed (`:105-106`). So both classifications in the README follow the tie-break rule.
- **Parity rule:** rule 2 is "not met", and rule 3 is stated explicitly. Both match `design/parity-method.md`.

## Required corrections

1. **The Named-limits sentence "MOD-22 and MOD-26 were not replayed" is false for MOD-26.** MOD-26 has a post-cap replay, `T21-replay-postcap` (executor blocked), listed in `dispositions.md`, `results.json` and the parity table. Suggested wording: "MOD-22 was not replayed; MOD-26 and MOD-29 had only the post-cap replay."
2. **The README's "New issues" entry for #1146 still says delivery "as `corbanu_untrusted_data`, so the child refuses it".**
   - The README's own MOD-21 paragraph says the opposite: "Whether the labelling caused the decline is not shown."
   - The scope is also too narrow. The resubmission typed into Snaga's pane view was reported wrapped too (`source=unknown`, T17 screen `53-resubmission-after-wait`).
   - Suggested wording: "…delivers the person's own task to the child as `corbanu_untrusted_data` (T17: also when typed into the child's pane view opened from Spawn status); whether the label caused the child's refusal is not shown."
   - The ledger says #1146 was corrected by a comment. I can't read that comment from here.

## Optional corrections

- **The "executor verdicts corrected" list** includes MOD-33 (failed → passed under D41) but leaves out the other owner-decision mappings: MOD-09 (blocked → accepted under D39) and MOD-28 (passed → accepted under D40). Either list them or call MOD-33 an owner-decision mapping rather than a correction.
- **MOD-34's "(not judged by either record)"** is inexact. UTM G25 wrote "passed, with wording ambiguity noted" (`moderate-20261014/runs/G25/.../result.md:63`). Better: "the case only asks to record it". Also, Tart's inspector listed only "Model key broker", while UTM listed "Model key broker, Credential broker". No expected result judges this; it is worth a note.
- **The `denied_path` attribution** ("the integrator's own check") is plausible: it came 4 s after the relay started, right after a `/models` 200. But nothing in the record names who made the request.
- **Review-ledger round 4** says everything was "all applied". That should be qualified by required correction 2.

VERDICT: CHANGES
EXIT:0
