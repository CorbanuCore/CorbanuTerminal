Screens of the candidate under test (Corbanu Terminal 0.1.48, origin/main 4c9b0a0cf0), captured by the integrator on the
host Mac on 2026-10-15 between 07:49Z and 07:52Z in a disposable profile and test project, GLM 5.3 Flash on zai. Folder
paths are shortened to /Users/tester/run. Not a test-case run, not a test result. Two Corbanu Terminals on the same
product home and folder: "A" is the first (agms-tui), "B" the second (agms-tui --term 2).
- X00: A's first start (folder-trust screen).
- X01, X02: A, /security, Down Down, Enter: the Aggressive review (top part, then after scrolling to the bottom).
- X03: A, after Enter on the review: the saved screen (r restarts).
- X04: A after r (restart): main screen.
- X05: B started (no trust screen: the folder is already trusted on this home).
- X06: B, /security: the level picker.
- X07: A, after sending "Run exactly this command: echo approved > ks-pending.txt": the approval prompt.
- X08: B, g in /security: grants and kill switch. X09: Down, Enter: the kill-switch review. X10: Enter: kill switch on.
- X11: B, Esc: back to the grants list.
- X12: A, after the kill switch was turned on in B (its prompt is gone).
- X13: A, after typing y and Enter.
- X14: B after more Esc presses (main screen).
