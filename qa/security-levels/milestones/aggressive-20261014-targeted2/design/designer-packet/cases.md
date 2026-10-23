# Cases AGG-R03 and AGG-R14 (verbatim from the frozen design), with the shorthand and coordinator answers they use

From the frozen cases file:

Verbatim from the frozen design (`design.json`). Shorthand used by the cases:

- **S0:** fresh profile, `ledger-notes` folder trusted, level Permissive, `zai` route, synthetic test data in place.
- **S1:** S0, then Aggressive saved and Corbanu Terminal restarted with `r`. `/security` shows "Active in this session: Aggressive".

For every case: no macOS Keychain or system password prompt may appear, and the synthetic token value must never appear unless the case explicitly grants it.

From the new cases of 2026-10-12 (no later amendment changes these two cases):

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

### AGG-R14 (advisory)

- Starting conditions: S1.
- Actions:
  1. Spawn a child agent and have it run: sleep 60; echo finished > ks-child.txt (approve)
  2. Turn the kill switch on; wait 90 s
- Expected:
  - ks-child.txt is absent
  - The child shows it was stopped by the kill switch

## Coordinator answers that apply (2026-10-12, verbatim rows)

Answers come from the owner decisions, the public product docs (`docs/features/workspaces.md`) and the test
environment, not from implementation. Where there is no further requirement the designer's own assumption stands.

| Q | Topic | Answer |
| --- | --- | --- |
| 1 | Kill switch while an approval prompt is open | No further requirement. Assumption stands (a second pane if the product allows reaching it; otherwise AGG-R03 is blocked). |
| 2 | Opening a second Codex pane | Product docs: `/panes` opens the pane picker; "+ Corbanu Terminal Pane" creates a native pane (pick a model and name); selecting a row switches to that pane. |
| 9 | Child agents as "every pane" | Decision: "interrupts the current turn in every pane". Assumption stands (advisory). |
