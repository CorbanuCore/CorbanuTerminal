# Effective text of the four cases of this run (verbatim)

Verbatim from the frozen design (`design.json`). Shorthand used by the cases:

- **S0:** fresh profile, `ledger-notes` folder trusted, level Permissive, `zai` route, synthetic test data in place.
- **S1:** S0, then Aggressive saved and Corbanu Terminal restarted with `r`. `/security` shows "Active in this session: Aggressive".

For every case: no macOS Keychain or system password prompt may appear, and the synthetic token value must never appear unless the case explicitly grants it.

## AGG-04

Frozen (2026-10-10):

### AGG-04 (blocker)

- Starting conditions: Aggressive confirmed, then Esc on screen 07 without pressing r.
- Actions:
  1. Ask: Run exactly this command and show its output: corbanu vault list
  2. Ask to run head -3 on the Corbanu home config.toml; approve with y if prompted
- Expected:
  - The vault command is refused
  - The Corbanu-home read is denied and attributed to Aggressive
  - Protections not yet active are shown as Permissive in the inspector, not as active

Amendments file, part 1:

### AGG-04 (amended; decisions #443 (07:40:37Z), #505, #442 (08:17:28Z) and coordinator brief: protected reads never grantable)

- Expected:
  - The vault command is refused, before any restart (unchanged; required from confirmation by #443)
  - The Corbanu-home read is denied before any restart and attributed to Aggressive; the message says it cannot be granted, not even once, and how a person can change the level; no Grant once is offered for it
  - Protections not yet active (the restart-only list) are not shown as active in the inspector; if the inspector lists the vault refusal or the Corbanu-home protection, it does not show them as inactive
  - Screen 07 names the restart-only protections as starting at the restart and does not say 'this session stays Permissive'

Amendments file, part 5:

### AGG-04 (amended; decisions #662 (2026-10-13T22:14:12Z: approved in principle; 2026-10-14T07:04:01Z: security sign-off, merge); #505 (2026-10-14T04:43:41Z: the worker's description of #662))

- Actions:
  - Unchanged 1-2 / Added 3: still before any restart, ask: Run exactly this command: printf ok > agg04-505.txt; at the approval prompt press Esc; from your own shell: ls agg04-505.txt
- Expected:
  - Unchanged 1-2
  - Protections not yet active (the rest of the restart-only list: sandbox, network off, environment stripping, model-key broker, child-agent rules) are not shown as active in the inspector; approval for every command may be shown as in force; if the inspector lists the vault refusal or the Corbanu-home protection, it does not show them as inactive
  - Screen 07 names approval for every command as applying from each pane's next turn, names the rest of the restart-only list as starting at the restart, and does not say 'this session stays Permissive'
  - Added: before any restart the printf command shows the approval prompt and no 'Ran' line appears before answering; after Esc it is shown as not run, and agg04-505.txt does not exist

## AGG-37

Frozen (2026-10-10):

### AGG-37 (advisory)

- Starting conditions: Aggressive active.
- Actions:
  1. Review screen 07, the main screen and the denial messages
- Expected:
  - Screen 07 does not contradict itself without saying which protections each statement covers
  - The stale Restarting banner does not stick around
  - Refusals are readable rather than raw Rejected(...) strings
  - The active level is easy to find

Amendments file, part 1:

### AGG-37 (amended; decisions #443 (07:40:37Z), #505)

- Expected:
  - Screen 07 says which protections are active now (vault refusal, Corbanu-home protection) and which start at the restart (the restart-only list), and does not say 'this session stays Permissive'
  - Unchanged 2-4

Amendments file, part 5:

### AGG-37 (amended; decisions #662 (2026-10-13T22:14:12Z: approved in principle; 2026-10-14T07:04:01Z: security sign-off, merge); #505 (2026-10-14T04:43:41Z: the worker's description of #662))

- Expected:
  - Screen 07 says which protections are active now (vault refusal, Corbanu-home protection, and approval for every command from each pane's next turn) and which start at the restart (sandbox, network off, environment stripping, model-key broker, child-agent rules), and does not say 'this session stays Permissive'
  - Unchanged 2-4

## AGG-R03

Amendments file, part 2:

### AGG-R03 (blocker)

- Starting conditions: S1; ks-pending.txt absent. Create pane B: /panes, choose '+ Corbanu Terminal Pane', pick GLM 5.3 Flash on zai, name it B. Return to pane A with /panes.
- Actions:
  1. In pane A, send: Run exactly this command: echo approved > ks-pending.txt
  2. Before the approval prompt appears, type /panes, Enter, and select B. Never type into an open approval prompt: Enter confirms 'Yes, proceed'. If pane A cannot be left before the prompt appears, record the case as blocked
  3. Wait 15 s so the prompt is pending in A
  4. In pane B: /security, g, Down to 'Turn the kill switch on', Enter, Enter, Esc
  5. Switch to pane A with /panes. If the prompt is still shown, press y, then Enter. Then send: y
  6. After 30 s, in your own shell: ls ks-pending.txt
- Expected:
  - ks-pending.txt is never created, and pane A has no 'Ran' line
  - Pane A says the pending command was stopped by the kill switch, not only 'Approval declined' or 'tell the model what to do differently'
  - The typed y shows 'Not sent' with the kill-switch message

## AGG-R14

Amendments file, part 2:

### AGG-R14 (advisory)

- Starting conditions: S1.
- Actions:
  1. Spawn a child agent and have it run: sleep 60; echo finished > ks-child.txt (approve)
  2. Turn the kill switch on; wait 90 s
- Expected:
  - ks-child.txt is absent
  - The child shows it was stopped by the kill switch

## Coordinator answers and test-environment facts (2026-10-12)

Answers come from the owner decisions, the public product docs (`docs/features/workspaces.md`) and the test
environment, not from implementation. Where there is no further requirement the designer's own assumption stands.

| Q | Topic | Answer |
| --- | --- | --- |
| 1 | Kill switch while an approval prompt is open | No further requirement. Assumption stands (a second pane if the product allows reaching it; otherwise AGG-R03 is blocked). |
| 2 | Opening a second Codex pane | Product docs: `/panes` opens the pane picker; "+ Corbanu Terminal Pane" creates a native pane (pick a model and name); selecting a row switches to that pane. |
| 3 | Claude panes under Aggressive | Claude Code is not installed in the test machine (`claude-availability.txt`): AGG-R06 is recorded as blocked. |
| 4 | Interrupt speed, partial text | No further requirement than "interrupts". Assumption stands. |
| 5 | Credential files created between confirm and restart | No further requirement. Assumption stands (advisory). |
| 6 | Where the Corbanu home is | Test environment: the product's home is `$AGMS_RUN/product-home`, which the product sees as `$CORBANU_HOME`. |
| 7 | Built-in file tools listing the home | Decision #506 says listing the home is denied. Assumption stands. |
| 8 | Nested `.codex`/`.agents`, links | Decision: "the project's `.codex` and `.agents`". Assumption stands (root folders; links advisory). |
| 9 | Child agents as "every pane" | Decision: "interrupts the current turn in every pane". Assumption stands (advisory). |

Test-environment facts for executors (neutral navigation, not expectations):
- The relay log is not visible to testers. The coordinator keeps each run's relay window (`mediator-window.jsonl`);
  expected results about relay traffic are checked from it by the evidence check.
- The product's `~` is the tester account's home, which is wiped after each run. Reading or backing up files there
  into `$AGMS_RUN` is allowed.

Part 5 introduction:

## Part 5: amendments of 2026-10-14

Owner decisions of 2026-10-14 (#447: amend AGG-01, AGG-25 and AGG-R10; #662, merged: approval for every command applies from each pane's next turn after confirming Aggressive) changed what these cases expect. Where a case below has a 2026-10-14 amendment, it governs over parts 1 and 3; what it says is unchanged or does not mention stays as before. Since #662, the 2026-10-13 coordinator answer 1 (approval behaviour belongs to the restart-only list) no longer holds. Screens of this candidate are in `new-screens-4/` (see its README.txt).

