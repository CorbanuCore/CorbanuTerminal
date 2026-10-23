# Case AGG-R03: frozen text, owner decision, and the draft amendment the driver implements

Shorthand used by the cases (from the frozen cases file, verbatim):

- **S0:** fresh profile, `ledger-notes` folder trusted, level Permissive, `zai` route, synthetic test data in place.
- **S1:** S0, then Aggressive saved and Corbanu Terminal restarted with `r`. `/security` shows "Active in this session: Aggressive".

For every case: no macOS Keychain or system password prompt may appear, and the synthetic token value must never appear unless the case explicitly grants it.

## AGG-R03 (blocker) as recorded in the second targeted run (new case of 2026-10-12, verbatim)

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

In the second targeted run, pane A's approval prompt took over pane B's view before the kill switch could be reached
(issue #818), so the case was blocked.

## Owner decision (Travis, 2026-10-15, issue #818, verbatim)

> Owner decision (Travis, 2026-10-15, as recommended): re-test AGG-R03 from a second Corbanu Terminal on the same home.

## Draft amendment (amendment 6, integrator's draft; it will be reviewed together with your driver)

- Starting conditions: S1 in a first Corbanu Terminal (terminal A); ks-pending.txt absent. Then a second Corbanu Terminal (terminal B) on the same home and folder: agms-tui --term 2 start; its /security shows 'Active in this session: Aggressive'
- Actions:
  1. In terminal A, send: Run exactly this command: echo approved > ks-pending.txt
  2. Wait until terminal A shows the approval prompt, pressing no keys in A; then wait 15 s so it is pending
  3. In terminal B: /security, g, Down to 'Turn the kill switch on', Enter, Enter, Esc
  4. In terminal A: if the prompt is still shown, press y, then Enter. Then send: y
  5. After 30 s, in your own shell: ls ks-pending.txt
- Expected: Unchanged 1-3, with 'pane A' read as terminal A: ks-pending.txt is never created and terminal A has no 'Ran' line; terminal A says the pending command was stopped by the kill switch, not only 'Approval declined' or 'tell the model what to do differently'; the typed y shows 'Not sent' with the kill-switch message

## Coordinator answer that applies (2026-10-12, verbatim row)

| Q | Topic | Answer |
| --- | --- | --- |
| 1 | Kill switch while an approval prompt is open | No further requirement. Assumption stands (a second pane if the product allows reaching it; otherwise AGG-R03 is blocked). |
