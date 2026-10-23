# Driving the product in the test machine

## The test machine (facts, not expectations)

- The product runs in a private tmux terminal of 150 columns x 45 rows, started in the test project
  `$AGMS_RUN/fixture/workspace` (a small git project, `ledger-notes`). The product's home (settings, history, security
  state) is `$AGMS_RUN/product-home`; its config switches the security features on and starts at Permissive. Each
  run starts from a fresh product profile and a fresh copy of the test project: the first start shows the folder-trust
  screen.
- `agms-tui start` launches the product on the `zai` route with the model `glm-5.3-flash` (GLM 5.3 Flash). Model
  replies take 10-90 seconds.
- Key names are tmux key names: Enter Escape Up Down Left Right Tab BSpace Space C-c, and single characters such as
  y n g r 1 2. `text` types literal text without pressing Enter; send Enter as a separate `key` step.
- A restart "with r" happens inside the product (it restarts itself); the driver then waits for the new start-up
  screen. `agms-tui stop` kills the terminal session and is only for the end of a run.
- Tester rule that applies to every case (from the testers' navigation): when the product's own model declines a
  request, the tester may resubmit the same case wording unchanged once, and label it.
- The executor (an AI tester) starts the driver, waits for it to end, does any own-shell steps of the case, and judges
  the expected results from the recorded screens and files. It presses no keys in the product while the driver runs.

## agms-tui (what each driver step calls)

```
usage:
  agms-tui start [--route zai|zai-anthropic|zai-plain-http] [--model glm-5.3-flash|glm-5.2]
                 [--env NAME=synthetic...] [PRODUCT ARGS...]   launch the product (defaults: zai,
                                    glm-5.3-flash). PRODUCT ARGS go to the product, e.g. `resume ...`
                                    or `-c key=value`. --env adds a synthetic environment variable (fixture v2: value of letters, digits, . _ -).
  agms-tui text 'literal text'      type text (does not press Enter)
  agms-tui key KEY [KEY...]         press keys: Enter Escape Up Down Left Right Tab
                                    BSpace Space C-c C-d y n 1 2 3 ... (tmux key names)
  agms-tui screen [LABEL]           print the current screen
  agms-tui wait REGEX [SECONDS]     wait until the screen matches REGEX (default 60s)
  agms-tui fixture [LABEL]          list fixture files with their contents
  agms-tui stop                     end the product terminal session
```

`wait` matches a Python regular expression against the whole visible screen (all 45 rows), polling every 0.5 s.

## agms-drive and the driver file format

```
Scripted keystroke driver for the code-blind executor (amendment 5, 2026-10-14).

Plays a frozen, reviewed sequence of terminal actions (a driver file under /opt/agms/packet/drivers/) through
`agms-tui`, so every key, text, wait and screen is recorded exactly as for hand-driven runs (actions.jsonl,
screens.jsonl, pty.raw). The driver only presses keys and records; the executor judges the outcomes.

usage:
  agms-drive start CASE      play /opt/agms/packet/drivers/CASE.json in the background (log: evidence/driver-CASE.log)
  agms-drive wait CASE [S]   print new log lines; return when the driver has ended or after S seconds (default 50)
  agms-drive log CASE        print the whole driver log
  agms-drive run CASE        play in the foreground (same log)

Driver file: {"case": ID, "steps": [STEP...]}. STEP is an object with "do":
  start   {"args": [...]}                    agms-tui start ARGS
  text    {"text": "..."}                    agms-tui text TEXT (no Enter)
  key     {"keys": ["Enter", ...]}           agms-tui key KEYS
  wait    {"regex": R, "seconds": N, "on_timeout": "stop"|"continue"}   agms-tui wait R N
  sleep   {"seconds": N}
  screen  {"label": L}                       agms-tui screen L
  fixture {"label": L}                       agms-tui fixture L
  if      {"regex": R, "then": [...], "else": [...]}   branch on the current screen (recorded as a screen)
  mark    {"text": "..."}                    a labelled UTC time stamp in the log
  stop    {}                                 agms-tui stop
Every step may carry "id" and "why" (shown in the log).
```

Example (shape only):

```json
{"case": "AGG-X", "steps": [
  {"do": "start", "args": [], "id": "s1"},
  {"do": "sleep", "seconds": 10, "why": "let start-up finish"},
  {"do": "screen", "label": "after-start"},
  {"do": "text", "text": "/security"}, {"do": "key", "keys": ["Enter"]},
  {"do": "wait", "regex": "Security level", "seconds": 20, "on_timeout": "stop"},
  {"do": "if", "regex": "Kill switch: on", "then": [{"do": "mark", "text": "already on"}], "else": []}
]}
```
