## 1. Review of `amendments3-v2.md`

| Case | Verdict | Correction |
|---|---|---|
| AGG-17 | agree | The pre-check is added correctly, and the rest of the row is unchanged. |
| AGG-18 | agree | — |
| AGG-23 | agree | — |
| AGG-24 | agree | — |
| AGG-33 | agree | — |
| AGG-R17 | agree | My corrections are applied. Step 1 no longer claims "no Grant once offered": the command also writes `.codex`, so that couldn't be pinned on `~/.zshrc`, and AGG-11 now covers it. "~/.zshrc unchanged" still protects step 1 if Grant once is offered and used. |
| AGG-R12 | agree | — |
| AGG-R13 | agree | — |
| AGG-32, Option B branch | change | "Neither … nor the inspector's launch checks" goes further than the decision. The question put to the owner was only about marking "at startup", and recommendation B is "no startup 'not clean' warning". Today's inspector behaviour isn't known. Replace with: "no startup warning marks the setup 'not clean' because of the readable extra config layer; the session starts at Aggressive and the inspector opens; if the inspector's launch checks still mark it 'not clean', record that as an advisory inconsistency, not a failure." |
| AGG-32, Option A branch and other answers | agree | — |
| AGG-02 / AGG-04 / AGG-37, pending (a) | agree | Both branches match the options as put to the owner. Under A, AGG-04's existing clause "restart-only list not shown as active" adjusts by itself. |
| AGG-11 | agree | Answer 8 confirms the case runs without `core.hooksPath` set. |
| AGG-27 | change | (1) Add #653 "Now (B)" to the Decision column, since it is the basis for the relaunch fallback. (2) Under "Now (B)", `/model` listing models but not connections is acceptable for this run. So say: "record the missing choice as a finding against #664 and the coordinator brief; it does not fail AGG-27." (3) Add an expectation following "picking one switches the session": "the independent evidence check shows `/messages` for the zai-anthropic turns and `/chat/completions` for the zai turns, at the recorded times". Also add to Actions: "record the wall-clock time of each pong". |
| AGG-22 | change | The pre-check has to run before step 1. Once the kill switch is on, turns are refused and the pre-check can't run. Add: "Do the pre-check before turning the kill switch on." The rest of the row is fine. |

## 2. Case changes from the coordinator's answers

- **Answer 7:** AGG-T05 is withdrawn. A home folder can't serve as the current folder in the test machine.
- **Answer 9:** AGG-T07 is revised. The executor records the time of each turn, and the independent evidence check reads the relay paths afterwards.
- **Answer 5:** AGG-T08 is revised. A launch with no connection named isn't possible, so the case observes the `/model` order and marking and the default a new pane offers.
- AGG-T01–T04 and T06 are unchanged.

```json
{"amendment_review":[
{"case":"AGG-17","verdict":"agree","correction":""},
{"case":"AGG-18","verdict":"agree","correction":""},
{"case":"AGG-23","verdict":"agree","correction":""},
{"case":"AGG-24","verdict":"agree","correction":""},
{"case":"AGG-33","verdict":"agree","correction":""},
{"case":"AGG-R17","verdict":"agree","correction":""},
{"case":"AGG-R12","verdict":"agree","correction":""},
{"case":"AGG-R13","verdict":"agree","correction":""},
{"case":"AGG-32 (Option B)","verdict":"change","correction":"Replace with: no startup warning marks the setup 'not clean' because of the readable extra config layer; the session starts at Aggressive and the inspector opens; if the inspector's launch checks still mark it 'not clean', record that as an advisory inconsistency, not a failure. The question and recommendation B concern only the startup marking/warning."},
{"case":"AGG-32 (Option A / other)","verdict":"agree","correction":""},
{"case":"AGG-02, AGG-04, AGG-37 (pending (a))","verdict":"agree","correction":""},
{"case":"AGG-11","verdict":"agree","correction":""},
{"case":"AGG-27","verdict":"change","correction":"Cite #653 'Now (B)' as the basis for the relaunch fallback. Say the missing /model choice is recorded as a finding against #664/the brief and does not fail AGG-27. Actions: record the wall-clock time of each pong. Expected added: the independent evidence check shows /messages for the zai-anthropic turns and /chat/completions for the zai turns at those times."},
{"case":"AGG-22","verdict":"change","correction":"Add: do the .husky pre-check before step 1 (kill switch on), since turns are refused afterwards."}],
"revised_cases":[
{"id":"AGG-T07","priority":"blocker","starting_conditions":"S1 on 'GLM 5.3 Flash · Z.AI (OpenAI-style)' (zai). The relay log is read afterwards by the independent evidence check (path /chat/completions = OpenAI-style, /messages = Claude-style).","actions":["Type /model, Enter","Down to 'GLM 5.3 Flash · Z.AI (Claude-style)', Enter; Enter on the default effort if asked","Send: Reply with exactly: pong; record the wall-clock time","Type /model, note which entry is marked (current), Esc","Type /security, note the active level, Esc","Type /model, choose 'GLM 5.3 Flash · Z.AI (OpenAI-style)', Enter; send: Reply with exactly: pong; record the wall-clock time"],"expected":["Step 1 lists GLM 5.3 Flash once per connection; OpenAI-style is listed first and marked (current)","After step 2 the footer names zai-anthropic; no restart, onboarding or API-key prompt","Both pongs complete with 'pong'; no broker or refusal error","Step 4 marks Claude-style as (current); Aggressive is still active","Evidence check: the request at the first recorded time used /messages, the one at the second /chat/completions"]},
{"id":"AGG-T08","priority":"advisory","starting_conditions":"S1; pane A switched in /model to 'GLM 5.3 Flash · Z.AI (Claude-style)'","actions":["Type /model; note the order and marking of each model offered on both connections; Esc","Type /panes, choose '+ Corbanu Terminal Pane'; at the model choice note which connection is offered first or highlighted for GLM 5.3 Flash; accept it without changing the connection; name the pane B","In pane B send: Reply with exactly: pong; record the wall-clock time"],"expected":["In /model, every model offered on both connections lists OpenAI-style first","The new pane offers OpenAI-style (zai) as its default for GLM 5.3 Flash; if it preselects Claude-style (inherited from pane A), record that as an advisory finding","The pong completes; the evidence check shows /chat/completions at the recorded time (or the path matching what the pane showed)"]}],
"withdrawn":["AGG-T05"]}
```