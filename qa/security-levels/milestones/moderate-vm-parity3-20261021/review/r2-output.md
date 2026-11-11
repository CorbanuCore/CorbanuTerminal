**Evidence check, round 2: amendment v9 and utm-par3 (D139/D140/D141)**

All three required corrections from round 1 are fixed correctly. I re-checked both records in full and found no new required correction. The method, every disposition, the "alike" result, the switch recommendation and the #1175 conclusion all follow from the evidence. I have four optional suggestions, listed at the end.

**Round 1 corrections**
1. **Tester deviations:** fixed. `dispositions.md`, `results.json` and the README's "Tester variance" bullet now say the following, and each point matches the evidence:
   - **T17-tart1:** chose Nazgul three times and Orc once before creating the Troll. Matches `result.md` lines 46–48.
   - **T17-tart2:** pressed Enter on the pane's own row and typed the task into the child's composer, then erased it unsent. `actions.jsonl` shows one Down at 20:14:54Z, and screen `mod21-send-row-selected-g1` shows the pane's own row highlighted, followed by 206 backspaces. On the retry, the resubmission came out as the task plus `\n/spawn`. That is 213 characters: the 206-character task plus 7, matching `task-delivery.txt` and `mod21-after-send-g1`.
   - **T17-tart3:** opened the "Remove managed crew" confirmation and cancelled it, and once opened the child's view by mistake. Both are confirmed in `screens.jsonl` and `actions.jsonl`.
   - The line "Every tester followed amendment v9" is gone.
2. **#1266:** the finding text is right in `dispositions.md`, `results.json` (per run and in `findings`) and the README. Issue #1266 exists, is open and describes the same thing.
   - All six testers moved from "Deep (current)" to "Standard (default)". For example, T17-tart3's `actions.jsonl` shows Up then Enter at 20:44:42Z, and its screen `mod21-child-reasoning-choice` shows Standard highlighted.
   - Every `mod21-child-view` shows `xhigh` and `zai deep`, and no screen of the child's view shows "zai standard".
   - The record says honestly that it is not known whether the choice is ignored or only labelled wrongly.
3. **Harness differences:** fixed. `effective-policy.txt` and the README now name `expected.json` as well as `agms_mediator.py`.
   - Every other guest file matches utm-par2's byte for byte. The mediator diff adds only `task_flags` and its call.
   - The driver differences are limited to paths, the lock holder, the VM name and the shared `h/` and stage.

**Whole-record re-check (all confirmed)**
- **Freezes:**
  - `FROZEN.sha256` checks 134 of 134 entries OK, and amendment v9's freeze checks 0 failures.
  - The freeze commit `487307d944` is dated 17:50:36Z, and nothing frozen has changed since it.
  - v4d differs from v4c only as `make_packet_v4d.py` says: it adds `amendments-v9.md`, which is byte-identical to `amendments-v9-final.md`.
- **Amendment v9 process:**
  - Both final texts are word for word the agents' fenced blocks: `amendments-v9-final.md` is the reviewer's, and `amendments-v9-draft.md` is the designer's.
  - The designer and the reviewer ran only local read commands, and neither made an MCP or web call. That is the same set-up as v6 and v7.
  - The README now describes S17 correctly.
- **Transport:**
  - The mediators forwarded 807 requests: 388 on UTM and 419 on Tart. The relay logged 808, the extra one being a GET at 17:51:04Z.
  - Every request returned 200, with no `session_error` and one tunnel connect per folder.
- **#1175:** confirmed in all six attempts.
  - In the mediator flags, every task-bearing message is a user message (or assistant #7), and all four flags are false.
  - In each child session file, the first send is a user message with origin `human`, and there is no `inter_agent_communication` record.
  - The comparison with the Tart parity run's T17 (sent to Snaga [orc], delivered as a mailbox assignment) is supported by utm-par2's dispositions, lines 55–58.
- **MOD-21:** all six D39 acceptances are supported. Text searches of the six T17 `screens.jsonl` files found no approval prompt, no "Approval declined" and no "Refused". The few "Running" matches in T17-tart1 are inside the child's reply text, not command cells.
- **MOD-22:** six passes. Every row in `mod22-rows-check.jsonl` matches.
- **MOD-11 (T09):** blocked under G-1 is right.
  - Items 3 and 8 were each declined twice by the model, with no tool call.
  - Items 1 and 2 show #1171's warning without a prompt.
  - "Item 8 passed on `8c73c8ea80`" is supported by utm-par2.
- **Machines and gates:**
  - 8 gates passed, all 13 preflights reported ok, and the guard saw 0 `SECURITYAGENT-SEEN`.
  - One VM ran at a time: UTM stopped at 19:31:49Z and Tart started at 19:32:00Z. "macOS 2" stayed stopped throughout.
  - UTM's verification boot shows the same state listing as `guest-state-start.txt`, and every Tart clone's erase check is clean.
- **Numbers in the README:** the latency figures, the memory figures (44–50% free), the times and the macOS and tool versions all match their sources.

## Required corrections

None.

## Optional

- **T17-tart3:** the "Remove managed crew" confirmation opened twice, at 20:44:56Z and again at 20:45:21Z, each cancelled with Esc. "The first try at Status … opened its confirmation" undercounts. Evidence: `runs/T17-tart3/collected/evidence/actions.jsonl` 20:44:55–20:45:29Z, and both `mod21-status-before-send` screens show "Remove the complete managed crew?".
- **MOD-22 deviation lists:** T18-utm3 also typed MOD-07 step 1 while the Saved screen was still open. Its `result.md` line 57 records this; T18-tart3 lists the same slip but T18-utm3 does not. T18-utm1's post-switch step-1 prompt (Esc) is covered only by the general "Noted" bullet, not in its own row.
- **Latency row:** the "median" is the upper median (`sorted[n//2]`). Tart's T17 windows also include that clone's gate requests (a GLM 5.3 Flash smoke request and a GET). The GLM 5.2 requests themselves took 24–82 s on Tart and 34–160 s on UTM, so the two T17 figures are not like for like. Evidence: `runs/T17-*/mediator-window.jsonl`, `transport.json`.
- **Decision 2:** "the state check only tested that it exists" is true. The gate's probes also try to open the file as executor accounts (`harness/agms-probe` line 112, denied as expected), and `capture-policy.sh` lists its metadata. Neither reads the file, but this could be added for completeness.

VERDICT: APPROVE