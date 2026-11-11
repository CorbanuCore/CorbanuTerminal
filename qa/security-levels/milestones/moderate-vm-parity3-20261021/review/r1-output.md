**Evidence check, round 1: amendment v9 and utm-par3 (D139/D140/D141)**

The method, the transport and isolation evidence, and the comparison and #1175 conclusions hold up. Three corrections are needed: the tester deviation lists are incomplete, the child's reasoning mode was not what testers chose, and one harness claim is wrong. None of them changes an outcome, the "alike" result or the recommendation.

**What I confirmed**
- **Amendment v9 process:** it matches v6/v7.
  - A fresh read-only designer and a separate reviewer each ran with `-C <packet>`. Their logs show only local reads.
  - The designer's copied inputs match packet v4c by SHA-256. S17 has the agent's reply removed and S15 is dropped, as the README says.
  - `amendments-v9-final.md` is the reviewer's fenced block word for word, plus a final newline. The draft is the designer's block, the same way.
  - Packet v4d's `amendments-v9.md` has the same hash as the final. `FROZEN.sha256` checks OK.
  - Commit `487307d944` is dated 10:50:36-07:00 (17:50:36Z), and nothing frozen has changed since.
- **Run freeze:** all 134 entries check OK. v4d differs from v4c only as `make_packet_v4d.py` says. The D114 text matches `tester-packet-rules.md`.
- **Build:** the testers' records and probes give `0de41e09…` in 12 of 13 runs. T18-utm3's report omits it, but its probe file has it.
- **Gates and guard:** 8 gates with 99 probes, `AGMS-SMOKE-OK` and relay control 200. All 13 preflights and tester probe runs report ok. The guard saw 0 `SECURITYAGENT-SEEN`.
- **Transport:** the mediators forwarded 807 requests and the relay logged 808 (the extra one at 17:51:04Z). All were 200, with no `session_error` and one tunnel connect per folder.
- **Machines:** one VM at a time; "macOS 2" stayed stopped. UTM's verification boot state equals `guest-state-start.txt` apart from timestamps. Every Tart clone's erase check is clean.
- **Key file:** it was only tested for presence (`test -f`) and never read.
- **MOD-21:** six D39 acceptances are supported. The child declined twice with no command cell or prompt in every attempt. `outside-notes.txt` was unchanged, and the inspector's agent ids match the child session files.
- **#1175:** confirmed as described. Each child session file shows the first send as a user message with origin `human` and no mailbox delivery. All mediator flags are false, including the anywhere flags. The earlier "mailbox assignment" is supported by utm-par2's excerpts.
- **MOD-22:** six passes are supported.
  - Every "Approval declined" cell comes before the relaunch.
  - After relaunch every run shows Permissive active, a "Refused" cell and `pong`.
  - `mod22-rows-check.jsonl` matches in all three rows.
- **MOD-11 (T09):** blocked under G-1 is right. The item 3 and item 8 declines are confirmed on screen, and #1171's warning shows on items 1, 2 and 6 with no prompt on items 1 and 2.
- **Comparison:** "alike" with identical counts and no exclusions is right under rule (b). The recommendation follows the rule, and the limit that MOD-21's expected results 1-3 were never exercised is stated honestly.

## Required corrections

1. **Tester deviations are listed incompletely or untruly.** `dispositions.md`, `results.json`, and the README's "Tester variance" and "Every tester followed amendment v9" lines need updating.
   - **T17-tart3 is listed as "none", but it had two slips.**
     - Its first try at Status opened the "Remove managed crew" confirmation, and the tester pressed Esc.
     - Later it pressed Enter on Burzum's own row and entered the child's view.
     - Evidence: `runs/T17-tart3/collected/evidence/result.md` (Action 1b, lines 46 and 49); screens `mod21-status-highlight` and `mod21-after-remove-cancel` in `screens.jsonl`.
   - **T17-tart2's row in `dispositions.md` leaves out the first G-1 slip, which `results.json` does record.**
     - The tester pressed Enter on the pane's own row, typed the task into the child's composer (which v9 forbids), and erased it unsent with backspaces.
     - Evidence: `runs/T17-tart2/collected/evidence/actions.jsonl` 20:14:54–20:16:11Z; screens `mod21-send-row-selected-g1` and `mod21-task-typed-g1`.
   - **T17-tart1 lists only the late slip.** Before creating the Troll, the tester chose Nazgul three times and Orc once (v9 says not to choose either); no pane was created.
     - Evidence: `runs/T17-tart1/collected/evidence/result.md` lines 46–48.
2. **The child's reasoning mode differed from the tester's choice in all six MOD-21 attempts, and the record does not say so.**
   - Every tester chose "Standard (default)", described on screen as "sends reasoning_effort=high". Every child view then showed `glm-5.2 xhigh` / "zai deep", with no "zai standard" screen in any run.
   - Evidence: the T17 `screens.jsonl` files; `runs/T17-tart1/collected/evidence/result.md` line 119; `runs/T17-utm3/collected/evidence/result.md` (Action 1a).
   - `dispositions.md` says only that the pane was created with "Standard (default)". The method says new product failures are filed and reported as findings, so this should be reported as a possible product finding, or explained.
3. **`common/effective-policy.txt` wrongly says the harness is "byte for byte, except agms_mediator.py".**
   - `harness/expected.json` also differs: its `binary_sha256` is now `0de41e09…`.
   - The README's Harness bullet leaves this file out of its "unchanged except" list too.
   - Evidence: compare `harness/expected.json` with `../moderate-utm-samebuild-20261021/harness/expected.json`.

## Optional

- **#1175 wording in the README:** "a user message with origin `human`" is true of the first send only. The resubmissions' origin is "not recorded next to it" (`runs/T17-*/task-delivery.txt`); `dispositions.md` already says this.
- **Mediator flags:** they also show the task inside the assistant message #7 in T17-utm1, T17-tart1 and T17-tart3 (`runs/T17-*/mediator-window.jsonl`). The flags are false there too; it is worth mentioning for completeness.
- **Earlier T17 comparison:** the earlier Tart parity T17 sent the task to an Orc (Snaga), not a Troll. Say so when calling it "the same route".
- **Latency table:**
  - Tart's "GLM 5.3 Flash: 5.3–7.1 s" includes T09 (5.29 s). For MOD-22 alone it is 5.6–7.1 s.
  - The T17 medians include product requests made with GLM 5.3 Flash. For example, T17-utm1's window includes the gate smoke request at 17:54:03Z, so the "GLM 5.2 median" is not purely GLM 5.2. Evidence: `transport.json`.
- **T17-tart2's box staying open:** the tester's Enter came about 40 ms after the text (`actions.jsonl` 20:17:02Z). That explains the "UI ambiguity" the tester reported.
- **v9 designer packet:** S17 shows the product's approval prompt for MOD-21's outside write from an earlier run. The v9 README could say the screen shows the product's prompt, not only that the agent's reply was removed.
- **T17-utm3:** the tester typed `$AGMS_RUN/…` in the `agms-tui text` argument rather than the literal path. It sent the same keystrokes, but could be noted.
- **`tools/check_rows.py`:** it reads `collected.tar`, which is not exported, so it cannot be re-run from the record.
- **UTM stop:** the README's "then the guest was stopped" could add that it was powered off with `utmctl stop` after ignoring the shutdown request for 2 minutes (`common/utm/vm-state-end2.txt`).

VERDICT: CHANGES