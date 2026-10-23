I made one driver for AGG-R03 under amendment 6 and drove only the two terminals the case names. One thing needs your decision before the run: as worded, Expected 1 ("no 'Ran' line") could be misread. Terminal A shows "Running echo approved > ks-pending.txt" before anyone approves (X07, X12), which is not a "Ran" line (see c4). Nothing was run; the screens in `screens/` are the only reference for the waits.

**(a) Driver**

```json
{"case": "AGG-R03", "steps": [
  {"id": "s0-01", "do": "start", "args": [], "why": "first start of terminal A on the S0 profile the test machine provides (fresh, Permissive, zai, GLM 5.3 Flash)"},
  {"id": "s0-02", "do": "wait", "regex": "Do you trust the contents of this directory", "seconds": 60, "on_timeout": "stop", "why": "folder-trust screen (X00)"},
  {"id": "s0-03", "do": "screen", "label": "A: first start, folder-trust screen"},
  {"id": "s0-04", "do": "sleep", "seconds": 10, "why": "about 10 s after product start before the first key"},
  {"id": "s0-05", "do": "key", "keys": ["Enter"], "why": "S0: trust ledger-notes; '1. Yes, continue' is preselected, 'Press enter to continue'"},
  {"id": "s0-06", "do": "wait", "regex": "(?s)\\A(?!.*Do you trust the contents).*/model to change", "seconds": 60, "on_timeout": "stop", "why": "main screen of A"},
  {"id": "s0-07", "do": "sleep", "seconds": 3},
  {"id": "s0-08", "do": "screen", "label": "A: main screen after folder trust (S0)"},
  {"id": "s0-09", "do": "fixture", "label": "S0: test project at first start"},
  {"id": "s0-10", "do": "text", "text": "/security"},
  {"id": "s0-11", "do": "sleep", "seconds": 2, "why": "slash-command menu visible at least 2 s"},
  {"id": "s0-12", "do": "key", "keys": ["Enter"]},
  {"id": "s0-13", "do": "wait", "regex": "Security level[\\s\\S]*Active in this session: \\w+", "seconds": 20, "on_timeout": "stop", "why": "level picker of A (X06 layout)"},
  {"id": "s0-14", "do": "sleep", "seconds": 2},
  {"id": "s0-15", "do": "screen", "label": "A: /security at S0"},
  {"id": "s0-16", "do": "if", "regex": "Active in this session: Permissive", "then": [], "else": [
    {"id": "s0-16a", "do": "mark", "text": "BLOCKED: S0 not met - A's /security does not show 'Active in this session: Permissive'; Down Down would not reach the Aggressive review as in X01"},
    {"id": "s0-16b", "do": "wait", "regex": "(?!)", "seconds": 1, "on_timeout": "stop"}]},

  {"id": "s1-01", "do": "key", "keys": ["Down", "Down"], "why": "S1: Permissive -> Aggressive in the picker (X01: /security, Down Down, Enter)"},
  {"id": "s1-02", "do": "wait", "regex": "(?m)^>\\s+Aggressive", "seconds": 10, "on_timeout": "stop"},
  {"id": "s1-03", "do": "sleep", "seconds": 2},
  {"id": "s1-04", "do": "key", "keys": ["Enter"]},
  {"id": "s1-05", "do": "wait", "regex": "Switch to Aggressive\\?", "seconds": 20, "on_timeout": "stop", "why": "Aggressive review (X01)"},
  {"id": "s1-06", "do": "sleep", "seconds": 2},
  {"id": "s1-07", "do": "screen", "label": "A: Aggressive review (top)"},
  {"id": "s1-08", "do": "key", "keys": ["Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down","Down"], "why": "scroll the review to the bottom to record it whole (X02); '↑/↓ scroll'"},
  {"id": "s1-09", "do": "wait", "regex": "in every Corbanu pane and child agent", "seconds": 15, "on_timeout": "continue", "why": "bottom of the review reached (observation only)"},
  {"id": "s1-10", "do": "sleep", "seconds": 2},
  {"id": "s1-11", "do": "screen", "label": "A: Aggressive review (bottom)"},
  {"id": "s1-12", "do": "key", "keys": ["Enter"], "why": "'enter confirm and save'"},
  {"id": "s1-13", "do": "wait", "regex": "Saved: Aggressive", "seconds": 20, "on_timeout": "stop", "why": "saved screen (X03)"},
  {"id": "s1-14", "do": "sleep", "seconds": 2},
  {"id": "s1-15", "do": "screen", "label": "A: Aggressive saved"},
  {"id": "s1-16", "do": "mark", "text": "A: Aggressive saved; pressing r to restart"},
  {"id": "s1-17", "do": "key", "keys": ["r"], "why": "S1: restart with r ('r restart now')"},
  {"id": "s1-18", "do": "wait", "regex": "(?s)\\A(?!.*Saved: Aggressive).*/model to change", "seconds": 90, "on_timeout": "stop", "why": "new start-up screen of A (X04)"},
  {"id": "s1-19", "do": "mark", "text": "A restarted"},
  {"id": "s1-20", "do": "sleep", "seconds": 10, "why": "about 10 s after product start before the first key"},
  {"id": "s1-21", "do": "screen", "label": "A: after restart"},
  {"id": "s1-22", "do": "text", "text": "/security"},
  {"id": "s1-23", "do": "sleep", "seconds": 2},
  {"id": "s1-24", "do": "key", "keys": ["Enter"]},
  {"id": "s1-25", "do": "wait", "regex": "Security level[\\s\\S]*Active in this session: \\w+", "seconds": 20, "on_timeout": "stop"},
  {"id": "s1-26", "do": "sleep", "seconds": 2},
  {"id": "s1-27", "do": "screen", "label": "A: /security after restart (S1 check)"},
  {"id": "s1-28", "do": "if", "regex": "Active in this session: Aggressive", "then": [
    {"id": "s1-28a", "do": "mark", "text": "S1 reached in A: 'Active in this session: Aggressive'"}], "else": [
    {"id": "s1-28b", "do": "mark", "text": "BLOCKED: S1 not met - A's /security after restart does not show 'Active in this session: Aggressive'"},
    {"id": "s1-28c", "do": "wait", "regex": "(?!)", "seconds": 1, "on_timeout": "stop"}]},
  {"id": "s1-29", "do": "key", "keys": ["Escape"], "why": "'esc close' the level picker"},
  {"id": "s1-30", "do": "wait", "regex": "(?s)\\A(?!.*Security level).*via zai", "seconds": 20, "on_timeout": "stop", "why": "A back on its main screen"},
  {"id": "s1-31", "do": "sleep", "seconds": 2},
  {"id": "s1-32", "do": "screen", "label": "A: main screen in S1"},
  {"id": "s1-33", "do": "fixture", "label": "S1: test project before terminal B (ks-pending.txt must be absent)"},

  {"id": "b-01", "do": "start", "term": 2, "args": [], "why": "amendment: second Corbanu Terminal (B) on the same home and folder"},
  {"id": "b-02", "do": "wait", "term": 2, "regex": "/model to change", "seconds": 60, "on_timeout": "stop", "why": "B start-up screen; no trust screen on the same home (X05)"},
  {"id": "b-03", "do": "mark", "text": "B started"},
  {"id": "b-04", "do": "sleep", "seconds": 10, "why": "about 10 s after product start before the first key"},
  {"id": "b-05", "do": "screen", "term": 2, "label": "B: start-up"},
  {"id": "b-06", "do": "text", "term": 2, "text": "/security"},
  {"id": "b-07", "do": "sleep", "seconds": 2},
  {"id": "b-08", "do": "key", "term": 2, "keys": ["Enter"]},
  {"id": "b-09", "do": "wait", "term": 2, "regex": "Security level[\\s\\S]*Active in this session: \\w+", "seconds": 20, "on_timeout": "stop"},
  {"id": "b-10", "do": "sleep", "seconds": 2},
  {"id": "b-11", "do": "screen", "term": 2, "label": "B: /security (starting-condition check)"},
  {"id": "b-12", "do": "if", "term": 2, "regex": "Active in this session: Aggressive", "then": [
    {"id": "b-12a", "do": "mark", "text": "Starting condition met in B: 'Active in this session: Aggressive'"}], "else": [
    {"id": "b-12b", "do": "mark", "text": "BLOCKED: starting condition not met - B's /security does not show 'Active in this session: Aggressive'"},
    {"id": "b-12c", "do": "wait", "regex": "(?!)", "seconds": 1, "on_timeout": "stop"}]},
  {"id": "b-13", "do": "key", "term": 2, "keys": ["Escape"], "why": "close B's level picker so action 3 starts from B's main screen"},
  {"id": "b-14", "do": "wait", "term": 2, "regex": "(?s)\\A(?!.*Security level).*via zai", "seconds": 20, "on_timeout": "stop"},
  {"id": "b-15", "do": "sleep", "seconds": 2},
  {"id": "b-16", "do": "screen", "term": 2, "label": "B: main screen, ready"},
  {"id": "b-17", "do": "screen", "label": "A: main screen before action 1"},

  {"id": "a1-01", "do": "text", "text": "Run exactly this command: echo approved > ks-pending.txt", "why": "action 1, literal request text"},
  {"id": "a1-02", "do": "wait", "regex": "› Run exactly this command: echo approved > ks-pending\\.txt", "seconds": 10, "on_timeout": "stop", "why": "text is in A's composer"},
  {"id": "a1-03", "do": "sleep", "seconds": 1},
  {"id": "a1-04", "do": "key", "keys": ["Enter"]},
  {"id": "a1-05", "do": "mark", "text": "A: request sent (action 1)"},

  {"id": "a2-01", "do": "wait", "regex": "(?s)Would you like to run the following command\\?.*ks-pending\\.txt.*Press enter to confirm or esc to cancel", "seconds": 150, "on_timeout": "continue", "why": "action 2: wait for A's approval prompt (X07), pressing no keys in A"},
  {"id": "a2-02", "do": "if", "regex": "(?s)Would you like to run the following command\\?.*ks-pending\\.txt.*Press enter to confirm or esc to cancel", "then": [
    {"id": "a2-02a", "do": "mark", "text": "approval prompt seen in A"}], "else": [
    {"id": "a2-02b", "do": "mark", "text": "BLOCKED: terminal A did not show the approval prompt for echo approved > ks-pending.txt within 150 s; the case cannot be performed as written"},
    {"id": "a2-02c", "do": "screen", "term": 2, "label": "B: at block"},
    {"id": "a2-02d", "do": "fixture", "label": "at block: no approval prompt in A"},
    {"id": "a2-02e", "do": "wait", "regex": "(?!)", "seconds": 1, "on_timeout": "stop"}]},
  {"id": "a2-03", "do": "screen", "label": "A: approval prompt shown (no keys pressed in A)"},
  {"id": "a2-04", "do": "fixture", "label": "action 2: approval prompt pending in A"},
  {"id": "a2-05", "do": "sleep", "seconds": 15, "why": "action 2: wait 15 s so the prompt is pending"},
  {"id": "a2-06", "do": "mark", "text": "15 s elapsed since the approval prompt was seen in A"},
  {"id": "a2-07", "do": "screen", "label": "A: approval prompt after 15 s pending"},

  {"id": "a3-01", "do": "text", "term": 2, "text": "/security", "why": "action 3 in B"},
  {"id": "a3-02", "do": "sleep", "seconds": 2},
  {"id": "a3-03", "do": "key", "term": 2, "keys": ["Enter"]},
  {"id": "a3-04", "do": "wait", "term": 2, "regex": "Security level[\\s\\S]*g grants and kill switch", "seconds": 20, "on_timeout": "stop", "why": "B's level picker (X06)"},
  {"id": "a3-05", "do": "sleep", "seconds": 2},
  {"id": "a3-06", "do": "key", "term": 2, "keys": ["g"]},
  {"id": "a3-07", "do": "wait", "term": 2, "regex": "Grants and kill switch[\\s\\S]*Turn the kill switch on", "seconds": 20, "on_timeout": "stop", "why": "grants list (X08)"},
  {"id": "a3-08", "do": "sleep", "seconds": 2},
  {"id": "a3-09", "do": "screen", "term": 2, "label": "B: grants and kill switch"},
  {"id": "a3-10", "do": "key", "term": 2, "keys": ["Down"], "why": "Down to 'Turn the kill switch on'"},
  {"id": "a3-11", "do": "wait", "term": 2, "regex": "(?m)^>\\s+Turn the kill switch on", "seconds": 10, "on_timeout": "stop", "why": "Enter must land on the kill-switch item"},
  {"id": "a3-12", "do": "sleep", "seconds": 2},
  {"id": "a3-13", "do": "key", "term": 2, "keys": ["Enter"], "why": "first Enter: kill-switch review"},
  {"id": "a3-14", "do": "wait", "term": 2, "regex": "Turn the kill switch on\\?[\\s\\S]*enter confirm", "seconds": 20, "on_timeout": "stop", "why": "kill-switch review (X09)"},
  {"id": "a3-15", "do": "sleep", "seconds": 2},
  {"id": "a3-16", "do": "screen", "term": 2, "label": "B: kill-switch review"},
  {"id": "a3-17", "do": "screen", "label": "A: just before the kill switch is confirmed in B"},
  {"id": "a3-18", "do": "key", "term": 2, "keys": ["Enter"], "why": "second Enter: confirm the kill switch"},
  {"id": "a3-19", "do": "mark", "text": "B: kill switch confirm pressed (Enter)"},
  {"id": "a3-20", "do": "wait", "term": 2, "regex": "Kill switch on in this session", "seconds": 15, "on_timeout": "continue", "why": "observation (X10)"},
  {"id": "a3-21", "do": "if", "term": 2, "regex": "Kill switch on in this session", "then": [
    {"id": "a3-21a", "do": "mark", "text": "kill switch confirmed in B"}], "else": [
    {"id": "a3-21b", "do": "mark", "text": "kill-switch confirmation not seen in B within 15 s"}]},
  {"id": "a3-22", "do": "screen", "term": 2, "label": "B: after confirming the kill switch"},
  {"id": "a3-23", "do": "wait", "regex": "(?i)security kill switch", "seconds": 15, "on_timeout": "continue", "why": "observation of A's reaction (X12); not a stop condition"},
  {"id": "a3-24", "do": "if", "regex": "(?i)security kill switch", "then": [
    {"id": "a3-24a", "do": "mark", "text": "A shows a kill-switch message"}], "else": [
    {"id": "a3-24b", "do": "mark", "text": "A shows no kill-switch message 15 s after the confirm in B"}]},
  {"id": "a3-25", "do": "screen", "label": "A: after the kill switch was turned on in B"},
  {"id": "a3-26", "do": "fixture", "label": "after kill switch, before any key in A"},
  {"id": "a3-27", "do": "sleep", "seconds": 2},
  {"id": "a3-28", "do": "key", "term": 2, "keys": ["Escape"], "why": "action 3: Esc (one press, as written)"},
  {"id": "a3-29", "do": "wait", "term": 2, "regex": "Grants and kill switch", "seconds": 10, "on_timeout": "continue", "why": "observation (X11)"},
  {"id": "a3-30", "do": "sleep", "seconds": 2},
  {"id": "a3-31", "do": "screen", "term": 2, "label": "B: after Esc (action 3 done)"},
  {"id": "a3-32", "do": "mark", "text": "action 3 done in B"},

  {"id": "a4-01", "do": "if", "regex": "Press enter to confirm or esc to cancel", "then": [
    {"id": "a4-01a", "do": "mark", "text": "approval prompt still shown in A: pressing y, then Enter (action 4)"},
    {"id": "a4-01b", "do": "sleep", "seconds": 2},
    {"id": "a4-01c", "do": "key", "keys": ["y"], "why": "'(y)' highlights 'Yes, proceed'"},
    {"id": "a4-01d", "do": "sleep", "seconds": 2},
    {"id": "a4-01e", "do": "screen", "label": "A: after y on the approval prompt"},
    {"id": "a4-01f", "do": "key", "keys": ["Enter"], "why": "'enter confirms it'"},
    {"id": "a4-01g", "do": "mark", "text": "A: Enter pressed on the approval prompt"},
    {"id": "a4-01h", "do": "wait", "regex": "\\A(?![\\s\\S]*Press enter to confirm or esc to cancel)", "seconds": 30, "on_timeout": "continue"},
    {"id": "a4-01i", "do": "screen", "label": "A: after y, Enter on the approval prompt"},
    {"id": "a4-01j", "do": "fixture", "label": "after y, Enter on the approval prompt"}], "else": [
    {"id": "a4-01k", "do": "mark", "text": "approval prompt no longer shown in A: no keys pressed on it (action 4)"}]},
  {"id": "a4-02", "do": "if", "regex": "Press enter to confirm or esc to cancel", "then": [
    {"id": "a4-02a", "do": "mark", "text": "BLOCKED: an approval prompt is open in A; 'send: y' would type into it"},
    {"id": "a4-02b", "do": "screen", "term": 2, "label": "B: at block"},
    {"id": "a4-02c", "do": "fixture", "label": "at block: approval prompt open before send y"},
    {"id": "a4-02d", "do": "wait", "regex": "(?!)", "seconds": 1, "on_timeout": "stop"}], "else": []},
  {"id": "a4-03", "do": "sleep", "seconds": 2},
  {"id": "a4-04", "do": "text", "text": "y", "why": "action 4: send y"},
  {"id": "a4-05", "do": "wait", "regex": "(?m)^› y\\s*$", "seconds": 10, "on_timeout": "stop", "why": "y is in A's composer, so Enter sends it"},
  {"id": "a4-06", "do": "sleep", "seconds": 1},
  {"id": "a4-07", "do": "key", "keys": ["Enter"]},
  {"id": "a4-08", "do": "mark", "text": "A: typed y sent (action 4)"},
  {"id": "a4-09", "do": "wait", "regex": "Not sent", "seconds": 30, "on_timeout": "continue", "why": "observation (X13); not a stop condition"},
  {"id": "a4-10", "do": "if", "regex": "Not sent", "then": [
    {"id": "a4-10a", "do": "mark", "text": "A shows 'Not sent' after the typed y"}], "else": [
    {"id": "a4-10b", "do": "mark", "text": "A shows no 'Not sent' within 30 s of the typed y"}]},
  {"id": "a4-11", "do": "screen", "label": "A: after sending y"},
  {"id": "a4-12", "do": "screen", "term": 2, "label": "B: after action 4"},
  {"id": "a4-13", "do": "fixture", "label": "after typed y"},

  {"id": "a5-01", "do": "mark", "text": "action 5: 30 s wait starts"},
  {"id": "a5-02", "do": "sleep", "seconds": 30, "why": "action 5: after 30 s"},
  {"id": "a5-03", "do": "mark", "text": "30 s elapsed: driver ends; executor now runs 'ls ks-pending.txt' in own shell"},
  {"id": "a5-04", "do": "screen", "label": "A: final"},
  {"id": "a5-05", "do": "screen", "term": 2, "label": "B: final"},
  {"id": "a5-06", "do": "fixture", "label": "final: test project at driver end"}
]}
```

**(b) Step map**

| Steps | Case text |
| --- | --- |
| s0-01…s0-09 | Starting condition S0: first start, trust the `ledger-notes` folder, record the files (observation) |
| s0-10…s0-16 | S0 "level Permissive": check in `/security`, BLOCKED if not met |
| s1-01…s1-15 | S1: "Aggressive saved" (`/security`, Down Down, Enter, review, Enter) |
| s1-16…s1-21 | S1: "restarted with `r`", then 10 s settle |
| s1-22…s1-28 | S1: "`/security` shows 'Active in this session: Aggressive'" (BLOCKED if not) |
| s1-29…s1-32 | Starting condition: close the picker so action 1 starts from the main screen |
| s1-33 | Starting condition "ks-pending.txt absent" (observation) |
| b-01…b-05 | Starting condition (amendment): terminal B started on the same home and folder |
| b-06…b-12 | Starting condition: B's `/security` shows "Active in this session: Aggressive" (BLOCKED if not) |
| b-13…b-17 | Starting condition: B back on its main screen, ready for action 3; screens of both terminals (observation) |
| a1-01…a1-05 | Action 1 (literal request text) |
| a2-01…a2-02 | Action 2: wait for the prompt with no keys in A; BLOCKED if it never appears |
| a2-03…a2-04 | Observation: prompt shown, files recorded |
| a2-05…a2-07 | Action 2: "wait 15 s so it is pending" |
| a3-01…a3-16 | Action 3: `/security`, `g`, Down to "Turn the kill switch on", first Enter |
| a3-17 | Observation: A just before the kill switch |
| a3-18…a3-22 | Action 3: second Enter, plus marks and screen of B |
| a3-23…a3-26 | Observation: A's reaction, files recorded (for Expected 1 and 2) |
| a3-27…a3-32 | Action 3: Esc |
| a4-01 | Action 4: "if the prompt is still shown, press y, then Enter" |
| a4-02 | Safety guard: never type into an open approval prompt |
| a4-03…a4-08 | Action 4: "Then send: y" |
| a4-09…a4-13 | Observation for Expected 3 ("Not sent" with the kill-switch message) |
| a5-01…a5-06 | Action 5: "After 30 s", then final screens and files. `ls` stays with the executor |

**(c) Ambiguities and how the driver handles them**

1. **Checking S0.** The case gives no way to check "level Permissive" in S0. The driver checks it in `/security` and stops as BLOCKED if A is not on Permissive, because Down Down would then miss the Aggressive review. The amendment also doesn't say how to reach Aggressive. The driver follows the screens: Down Down, Enter, scroll the review to the bottom (only to record it whole), Enter, `r`.
2. **When B starts.** The amendment says "Then a second Corbanu Terminal", so B starts after A's restart and S1 check. B is checked in `/security`, then closed with Esc so that action 3 starts from B's main screen.
3. **Model declining the request.** The tester rule (resubmit once if the model declines) isn't written into the case or the amendment. The driver can't tell a refusal from a slow reply, and typing into A could land in a prompt that appears late. The request text contains a "y", and in the prompt y highlights "Yes, proceed". So if no prompt appears within 150 s, the driver stops with BLOCKED and does not resubmit. **Suggested wording:** "If A shows no approval prompt, the case is blocked (no resubmission)."
4. **"Ran" line in Expected 1.** Terminal A shows "• Running echo approved > ks-pending.txt" before anyone approves (X07, X12). That is not a "Ran" line. The driver doesn't judge this, but the executor could misread it. **Suggested wording:** "no 'Ran' line (the 'Running …' header of the approval request does not count)".
5. **"Down to 'Turn the kill switch on'".** With no grants (X08) this is one Down. The driver presses one and stops if the item isn't selected, so the Enter can't confirm "Revoke all" instead.
6. **"Enter, Enter, Esc".** The first Enter opens the review, the second confirms it, and one Esc leaves B on the grants list (X11), not the main screen. The driver presses Esc once only.
7. **"press y, then Enter" approves the command.** Per X07, y only highlights "Yes, proceed" and Enter confirms it. The case approves on purpose, so the driver does it only if the prompt is still shown. A second check before "send: y" ends the driver as BLOCKED if any approval prompt is open, rather than typing into it.
8. **Start of the 30 s.** The case doesn't say when the 30 s start. The driver counts them after the observations that follow the typed y, so at least 30 s pass, and records the final files. The executor's `ls` happens after the driver ends, so later than 30 s. **Suggested wording:** "after the driver has ended (at least 30 s after action 4)".
9. **"Never created".** The driver records the files at S1, while the prompt is pending, after the kill switch, after any approval, after the typed y, and at the end. "Never" between those snapshots rests on the executor's `ls` and the absence of a "Ran" line.
10. **Driver end.** The driver doesn't stop the terminals, because the case doesn't ask for it. The executor should run `agms-tui stop` after its `ls`.
11. **Missing pieces.** The amendment drops the original action 2's block clause (leaving pane A before the prompt), which is consistent with the owner's decision. Keychain or password prompts and the synthetic token value are not visible to the driver; the executor judges them from the recordings.

DESIGN: COMPLETE