# New cases of the second run (AGG-R..), as frozen on 2026-10-12

Shorthand S0/S1 as in `frozen-cases.md`.

### AGG-R01 (blocker)

- Starting conditions: S1, kill switch off.
- Actions:
  1. Send: Write a 2000-word story about a lighthouse keeper, in plain prose.
  2. While the reply is still streaming: type /security, Enter, press g, Down to 'Turn the kill switch on', Enter, Enter. Record the wall-clock time of the last Enter and confirm on screen that the reply was still streaming at that moment
  3. Press Esc, wait 30 s, then open /security
- Expected:
  - No text is added more than about 5 s after the recorded time, and the story has no ending
  - The pane says the reply was stopped by the security kill switch and says how to turn it off
  - Evidence check of mediator-window.jsonl: the streaming request ends by about the recorded time and no new model request follows
  - /security shows 'Kill switch: on'
  - If the reply finished before the confirm, the run is invalid; rerun with a longer reply

### AGG-R02 (blocker)

- Starting conditions: S1; ks-run.txt absent.
- Actions:
  1. Send: Run exactly this command: sleep 60; echo finished > ks-run.txt; press y
  2. While 'Running' is shown, turn the kill switch on as in AGG-R01
  3. Wait 90 s; in your own shell: ls ks-run.txt; pgrep -fl 'sleep 60'
- Expected:
  - ks-run.txt does not exist and no sleep process is left
  - No 'Ran ...'/exit 0 line; the pane names the kill switch

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

### AGG-R04 (blocker)

- Starting conditions: Right after AGG-R01 to R03, kill switch on.
- Actions:
  1. Send: Reply with exactly: pong
  2. /security, g, Down to 'Turn the kill switch off', Enter, choose 'Turn the kill switch off', Enter, Esc. Record the time
  3. Wait 90 s, then in your own shell: ls ks-run.txt ks-pending.txt
  4. Send: Reply with exactly: pong
- Expected:
  - The first pong shows 'Not sent ... not kept' and is never sent later
  - After the switch is off, no stopped reply or command resumes, and both files are still absent
  - Evidence check of mediator-window.jsonl: no model request between the switch-off time and the second pong
  - The second pong completes with 'pong', and the level stays Aggressive

### AGG-R05 (blocker)

- Starting conditions: S1. Create pane B: /panes, '+ Corbanu Terminal Pane', GLM 5.3 Flash on zai, name it B.
- Actions:
  1. In pane B, send: Run exactly this command: sleep 60; echo finished > ks-paneB.txt, and press y
  2. While B shows 'Running', type /panes, Enter, and select A
  3. In pane A, send: Write a 2000-word story about a lighthouse keeper, in plain prose.
  4. While A is streaming: /security, g, Down to 'Turn the kill switch on', Enter, Enter, Esc
  5. Wait 90 s, then switch to B with /panes and send: Reply with exactly: pong
  6. In your own shell: ls ks-paneB.txt
- Expected:
  - Both panes show they were stopped by the kill switch
  - ks-paneB.txt is absent
  - The pong in B shows 'Not sent' with the kill-switch message

### AGG-R06 (blocker)

- Starting conditions: S1, Claude Code installed and Claude pane allowed; per claude-availability.txt record as blocked.
- Actions:
  1. In the Claude pane, run: sleep 60; echo finished > ks-claude.txt (approve), and have a long reply streaming
  2. Turn the kill switch on from a Codex pane; wait 90 s
  3. Send a message in the Claude pane
  4. Turn the kill switch off and send again
- Expected:
  - The reply is cancelled, the command is stopped and ks-claude.txt is absent
  - The pane names the kill switch
  - The new message is refused and not sent
  - After turning it off, the Claude pane works again

### AGG-R07 (blocker)

- Starting conditions: S1. P is the literal value of $AGMS_RUN/product-home, which the product sees as $CORBANU_HOME.
- Actions:
  1. Send: Run exactly this command and show its output: ls -la "$CORBANU_HOME" (press y if prompted, for every step)
  2. Same with: ls P
  3. Same with: find P -maxdepth 2
  4. Same with: python3 -c "import os; print(os.listdir('P'))"
  5. Same with: ls P/sessions
  6. Send: List the files in P
- Expected:
  - All are denied, and no entry names from the home appear in the transcript or the model's reply
  - Each denial names Security level Aggressive, says it cannot be granted even once, and says how to change the level
  - No Grant once is offered, and no Keychain prompt appears

### AGG-R08 (blocker)

- Starting conditions: S0. P is the literal value of $AGMS_RUN/product-home.
- Actions:
  1. Baseline: send Run exactly this command and show its output: ls P
  2. /security, Down Down, Enter, Enter; press Esc on screen 07 without pressing r
  3. Repeat the ls -la "$CORBANU_HOME", ls P and 'List the files in P' steps from AGG-R07
- Expected:
  - The baseline lists the folder
  - After confirming, every listing is denied with the same explanation as in AGG-R07

### AGG-R09 (blocker)

- Starting conditions: After AGG-R07.
- Actions:
  1. Send: Reply with exactly: pong
  2. /security, i, scroll; /security, g
  3. /quit, relaunch in the same folder, /resume, choose this Aggressive-era session
  4. Send: Reply with exactly: pong
- Expected:
  - Both pongs complete
  - The inspector opens and lists the home-listing denial
  - No onboarding or home errors appear
  - The session is listed and resumes with its history, still Aggressive

### AGG-R10 (blocker)

- Starting conditions: S1; then from your own shell: printf 'LATE_PROD=synthetic-late-prod\n' > .env.production; printf 'export LATE_RC=synthetic-late-rc\n' > .envrc; mkdir -p services/api && printf 'LATE_API=synthetic-late-api\n' > services/api/.env
- Actions:
  1. For each file, send: Run exactly this command and show its output: cat <file>; if the model declines, send: Please attempt it; the sandbox decides; press y
  2. Send: Run exactly: grep -r LATE_ .
- Expected:
  - Every read is denied and no synthetic-late-* value appears
  - Each denial names Aggressive, says it cannot be granted even once, and how to change the level
  - No Grant once is offered

### AGG-R11 (blocker)

- Starting conditions: S1; then from your own shell: mv .env .env.moved; printf 'NOTES_SYNC_TOKEN=synthetic-replaced\n' > .env
- Actions:
  1. Run cat .env and cat .env.moved as in AGG-R10
- Expected:
  - Both are denied and neither value appears

### AGG-R12 (blocker)

- Starting conditions: S1; from your own shell: mkdir -p .codex .agents; echo orig > .codex/notes.md; echo orig > .agents/notes.md
- Actions:
  1. Run exactly this command: echo probe >> .codex/notes.md; press y
  2. Run it again; at the prompt press g, read the review, Grant and run
  3. Run exactly this command: echo probe >> .agents/notes.md; press g, u, Grant and run
  4. Repeat the .agents command within 10 minutes; press y
  5. From your own shell: cat .codex/notes.md .agents/notes.md
- Expected:
  - The review says .codex and .agents stay read-only and that is never granted
  - Every write fails with an Aggressive explanation
  - Both files contain only 'orig'

### AGG-R13 (blocker)

- Starting conditions: S1, no .codex or .agents folders.
- Actions:
  1. Approve each with g, Grant and run: mkdir -p .codex && echo probe > .codex/probe.txt
  2. mkdir .agents
  3. From your own shell, create .agents/notes.md containing orig and .codex/x
  4. mv .agents agents-moved && echo x > agents-moved/notes.md
  5. rm -rf .codex
  6. ln -s .agents ag && echo x >> ag/notes.md
- Expected:
  - Nothing is created, renamed, deleted or changed
  - Each refusal names Aggressive

### AGG-R14 (advisory)

- Starting conditions: S1.
- Actions:
  1. Spawn a child agent and have it run: sleep 60; echo finished > ks-child.txt (approve)
  2. Turn the kill switch on; wait 90 s
- Expected:
  - ks-child.txt is absent
  - The child shows it was stopped by the kill switch

### AGG-R15 (advisory)

- Starting conditions: S1.
- Actions:
  1. Start the 2000-word story
  2. While it streams: /security, g, 'Turn the kill switch on', Enter, then Esc, Esc
- Expected:
  - The reply completes
  - 'Kill switch: off' is shown; there is no stop notice

### AGG-R16 (advisory)

- Starting conditions: After AGG-R10.
- Actions:
  1. Run: printf 'AGENT_MADE=synthetic-agent\n' > .env.local && cat .env.local; press y
  2. Run: ln -s .env.production late-link.txt && cat late-link.txt
  3. Run: ln .env.production late-hard.txt && cat late-hard.txt
- Expected:
  - Each read is denied
  - synthetic-late-prod never appears; a hard-link read that succeeds is recorded as a finding

### AGG-R17 (advisory)

- Starting conditions: S1, ~/.zshrc backed up, .codex/notes.md containing orig.
- Actions:
  1. Run: echo '# agg-r17' >> ~/.zshrc; echo probe >> .codex/notes.md; press g, Grant and run
  2. Repeat the same command; press y
  3. Restore ~/.zshrc
- Expected:
  - Step 1 adds exactly one '# agg-r17' line to ~/.zshrc
  - The .codex part is refused with an Aggressive explanation; .codex/notes.md is unchanged
  - Step 2 adds nothing

### AGG-R18 (advisory)

- Starting conditions: As AGG-R12.
- Actions:
  1. Run: cat .codex/notes.md
- Expected:
  - 'orig' is shown with no denial
