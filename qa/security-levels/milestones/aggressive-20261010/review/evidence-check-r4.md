**Verdict: fail.** The structure, hashes, replays and isolation receipts all hold, and most dispositions match their evidence. But AGG-20's failed (S) disposition rests on a parent/child comparison that the record's own evidence undermines. That disposition, its README row and how defect #435 is framed need correcting. I worked read-only inside `record/`. I couldn't write `review/evidence-check.md` (cited by `README.md` and `review-ledger.md`), so this answer is its content. This is review 5 of a budget of 5 (`results.json` `review_budget.used = 5`), so any further check needs a decision on the budget.

## 1. No case vanished (holds)
- `design.json` and `results.json` each list 39 cases, with the same ids in the same order. Each case has exactly one disposition: 17 passed, 16 failed, 6 blocked, matching `dispositions.md` and `README.md`.
- `design_sha256` matches `design.json`.
- All 1,248 `{path, sha256}` references exist and verify.
- All 30 `runs/` directories are referenced, and every attempt label exists. No run is orphaned.
- `design/FROZEN.sha256` verifies 70 of 71 lines. The failing line is its own self-entry, which is disclosed.
- Every raw executor verdict matches its run's `VERDICT` line. The four runs with duplicate `result.md` files (G02, G10b, G15, G18) have byte-identical copies.

## 2. Evidence per case

### Blocking: AGG-20 (failed, class S, "by the product's own statement")
The disposition says the child "reports a weaker policy than the parent". It compares two G12 inspectors:
- The parent's, `AGG20-security-inspect` at 01:41:26Z, taken right after a restart and before any model turn.
- The child's, `AGG20-child-inspect` at 01:46:12Z, taken after the parent process had run several turns.

The record shows those "weaker" rows are not specific to children. **Main-session** Aggressive inspectors show the same text:

| Run | Screen | Time | State |
| --- | --- | --- | --- |
| G08 | `AGG38-INSPECTOR-TOP` | 01:01:45Z | after one turn |
| G14 | `r01-inspector-top` | 02:32:45Z | after many turns |

Both show the three rows the disposition calls weaker:
- "approved or allow-listed commands can run outside the sandbox"
- "Network: off inside the sandbox, on for commands that run outside it"
- "Vault: … readable to agent commands"

I checked every Aggressive inspector in the record:
- **Strong rows** (G02, G12 parent, G12b, G15, G18, G21): each was opened after a (re)start with no model turn in between.
- **Weak rows** (G08, G12 child, G14): each followed turns in that process.
- The G12 parent was never re-inspected after its own turns.

So the comparison is confounded by timing. Behaviour also contradicts the weak rows in main sessions:
- G07: an approved `curl` stayed sandboxed and never reached the relay.
- G04: an approved write outside the project was denied by the OS.

That points to inaccurate inspector text (class U), not a weaker child sandbox. The child's own behaviour matched the parent's where it was exercised: `curl` and `echo` prompted, and the vault was refused. What happens after approval was not exercised, and neither was the weaker role.

**Correction needed:** AGG-20 should be **blocked**: expected results 1 and 2 are unexercised, and 3 passed. Alternatively it could be **failed (U)**, but only on inspector truthfulness, with the main-session occurrences cited. In either case:
- Drop "child weaker than parent" and class S from `dispositions.md`, `results.json` and the README row.
- Reframe #435. The inspector-row inaccuracy belongs with #437 (AGG-24).

r1–r3 all accepted the parent/child comparison, so this is a new point.

### Earlier corrections: applied
- **r3:**
  - The `.env`-specific grant reachability wording is in place (README row, #442, AGG-17 reason).
  - AGG-21's G12b raw verdict now reads passed, marked as differing from the executor; AGG-19 is marked the same way.
  - AGG-16 now says the value reached the model's context.
  - AGG-20 now notes the child was a `/spawn` pane, and AGG-27 notes the relaunch.
  - AGG-36 now says the explanation depends on the model.
  - Product logs are bound into case evidence.
- **r2:** AGG-22 is now failed, and `checker-output.txt` exists.
- **r1:** all six corrections are present.
- **Exception (non-blocking):** the `results.json` summary for AGG-37 still says the raw `Rejected(...)` string was "shown". Only `dispositions.md` says the model relayed it.

### Verified as written (my own checks)
- **AGG-01:** the session rows match `design/packet/screens/03-inspector-permissive.txt`.
- **AGG-02/03:** the footer, the timing text, and the inspector's "next start Aggressive" are all on screen.
- **AGG-06:** the override follows `design/original-proposal.md` (AGG-06 names commands 3 and 4 as the ones that must prompt).
- **AGG-07:** the typed-text notice is shown, Enter did not approve, and `notes.txt` is unchanged.
- **AGG-16:** the screen shows the "Read notes.txt" tool step, then the model's reply containing `AGG16SYNTH123`.
- **AGG-22:**
  - The kill-switch text says "the credential broker stays closed".
  - "What is 2+2?" was answered at 02:01:53Z, and a ~40 KB product-sized request was forwarded at 02:01:52Z.
  - The approval prompt with the switch on (02:00:49Z) is the **only** Aggressive approval prompt in the record without "Grant once", which supports "grants refused".
  - The review text says the credential broker "sends model requests for Corbanu", which supports the contradiction.
- **AGG-25:** packet screen E1 shows the same `cli_auth_credentials_store` trigger, so G15b reproduced the designer's own condition. The blocked → fix → passed → saved sequence is on screen.
- **AGG-26:** confirmed on screen.
- **AGG-27:** confirmed on screen.
- **AGG-28:** the run's mediator window has no plain-HTTP `POST` row; its 28 forwarded rows all have the executor's tool list and growing request sizes. The campaign's only plain-HTTP `POST` row is the 23:08:15Z fixture check.
- **AGG-29:** confirmed on screen.
- **AGG-32:** confirmed, including the inspector's "[degraded] Protected boundary … not clean".
- **AGG-A01:**
  - The mediator shows two GLM 5.2 turns, and `web_search` is dropped on the second one.
  - "Saved: Aggressive" is on screen before the restart. That Aggressive was active afterwards rests on the mediator, as disclosed.
- **Blocked cases:**
  - AGG-17/18/23/33: G06's product log shows `ToolCall: exec_command {"cmd":"cat .env"}` with no prompt.
  - AGG-30: G19's log shows the `;` chain was never issued.
  - AGG-21: G12b's log shows no edit was attempted.
- **Failed cases:** each has on-screen evidence of its failure.

### Non-blocking notes
- **AGG-04:** the AGG-02 review text says "Core's level becomes Aggressive as soon as you confirm … agent commands cannot open protected paths". In G03, `ls -la "$CORBANU_HOME"` listed the home before the restart (`AGG-04-home-read-final`). So the home-read failure contradicts the product's own statement, not only designer assumption Q4. "Needs product decision" fits the vault part only.
- **AGG-24/#437:** G14's inspector shows "every row verified at launch" next to session rows that changed after launch and contradict the Aggressive review.
- **AGG-21:** "the project config loosens nothing" was checked only before any turn, which is exactly when the rows look strong.
- **AGG-17 and related cases:** every Aggressive approval prompt offered "Grant once", for example for `python3 … add` and `touch`. The grant review was exercised in G14 (on `touch`), so expected results 1–2 have partial evidence on another command.
- **AGG-31:** "the level is unchanged" rests on `/permissions` showing "Ask for approval (current)"; `/security` was never opened in G20.
- **AGG-38:** "runs as before" has no Permissive baseline; only the run under Aggressive was observed.
- **README:** the results-by-area table omits AGG-38.
- **`results.json` AGG-34 summary:** it says the lists "differ"; `dispositions.md` says the list is "not shown" in the inspector.

## 3. Replays (honest)
- Each of the 30 runs has exactly one `thread.started`, matching its recorded agent; all 30 IDs are unique, and every run exited 0. Earlier attempts are kept.
- Each v2 replay prompt differs from its v1 base only in the group/case lines, the replay note from `tools/replays.json`, and the `--env` rule.
- The notes give fixtures and navigation, never outcomes. G18b's "which the credential broker cannot accept" describes the fixture but comes close to stating an outcome.
- The GLM 5.2 switches are recorded with a reason. The campaign log has 11 GLM 5.2 requests: G10b 3, G11b 6, G17c 2. All 1,400 requests got HTTP 200.
- I found no executor command that edited the tool-written evidence (screens, actions, fixture, PTY). G06's executor redacted the token only in its own side copies; `fixture.jsonl` still holds the canary.

## 4. Isolation
- All 30 runs' `probes.json` pass (launcher, tmux child and executor).
- All 51 `isolation-<case>.json` receipts carry the right agent, run, case and candidate hash. Each one's design hash equals the current `design.json`, which is in FROZEN, and its policy, tool-inventory, probe and mediator hashes verify.
- Harness v2 digests match `common/fixture-v2-control.txt`.
- The stated limits look complete. One addition: the `results.json` and README claims that rest on screens alone, such as AGG-09's "token appears nowhere", have no request bodies behind them. The README states this generally.

## 5. Before sign-off
- The inspector-row pattern (session rows say approved commands can leave the sandbox with network on, and the vault is readable, after the first turn of an Aggressive session) is unexplained, and the record can't tell display error from real loosening. Main-session behaviour (G04, G07) suggests display error. It is worth a product question before anyone relies on the inspector.
- The review budget is spent.

VERDICT: fail