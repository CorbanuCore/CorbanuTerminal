I made no changes. I only read the packet. For Job 1, 13 of the 15 amendment rows need a correction: only AGG-16 and AGG-37 are fine as written. For Job 2, I propose 18 new cases, AGG-R01 to AGG-R18; 13 are blockers. AGG-R06 needs a Claude pane, so it will be recorded as blocked, since Claude Code isn't installed on the test machine.

## 1. Job 1: amendment review

| Case | Verdict | Correction |
|---|---|---|
| AGG-04 | change | The vault part is fine. The Corbanu-home read is fine, but cite the coordinator brief ("protected reads are never grantable") for "cannot be granted". No decision requires the **inspector** to show restart-only protections as "starting at the restart": only screen 07 must name them (#443, #505, brief). Restore the frozen clause: "Protections not yet active (the restart-only list) are not shown as active in the inspector." If the inspector lists vault refusal or Corbanu-home protection, they must not read as inactive. |
| AGG-09 | change | The row says "(unchanged)" but drops "('Operation not permitted')". Restore: "or the OS denies the command ('Operation not permitted')". Add from #442 19:55 (1): the denial text must not suggest only `.env` files found at start are protected. |
| AGG-10 | change | Limiting "cannot be granted, and how to change the level" to the Corbanu-home read is too narrow. Keychains are protected credential files, so the same text applies to `ls ~/Library/Keychains` (#442 08:17). It applies to `security find-generic-password` too if that is refused as a protected read. Cite #443 07:40 for home denial; #506 only covers listing. |
| AGG-15 | change | "PLAIN_VAR is absent" is right. "the /security review **and inspector** say so" is stronger than #440, which only fixes the wording (#458) of the issue's review text. Correct to: "the /security review says variables outside the allowlist are removed too". |
| AGG-16 | agree | — |
| AGG-17 | change | The new target is fine. Add from #442 19:55 (2), "Grant review … text to be updated": the review says the project's `.codex`/`.agents` stay read-only and protected reads stay denied, even with this grant. |
| AGG-18 | change | #442 says to exercise grants "on those (e.g. protected writes)". A `notes.py add` runs the same with or without a grant, so "under the grant" can't be observed. Use a protected write, e.g. `echo '# grant-once-1' >> ~/.zshrc`, its repeat, and `echo '# grant-once-2' >> ~/.zshrc`. The frozen "repeat and different command are denied" then stays unchanged: `~/.zshrc` gains exactly one line, and the old "reads .env" bullet becomes "the first run writes once". Keep the `cat .env` bullet. |
| AGG-22 | change | (a) The row says the question is refused "after the restart" but adds no action; add "after step 4, ask a normal question again". (b) Since #445 refuses turns, step 2 can't produce a new approval prompt. Open the approval before step 1; expected: the pending command never runs (#507-4) and no grant can be made. (c) Add from #507-3: the kill-switch text says Claude panes stop too and the current turn is interrupted in every pane. |
| AGG-23 | change | Same issue as AGG-18. Use a protected write (`echo '# revoke-probe' >> ~/.zshrc`) so the frozen "The command is denied again" stays unchanged. |
| AGG-24 | change | Also add from #442 19:55: the inspector says `.codex`/`.agents` are always read-only (even with Grant once), and credential-file protection includes files created later. |
| AGG-26 | change | Make it conditional. #436 and the brief only remove "restart now" **when a key was handed to the broker**. Add a precondition: record whether the zai key came from the environment or vault and was handed to the broker. Otherwise "restart now" may be offered. The other bullets are fine. |
| AGG-33 | change | (a) Use a protected write (`echo '# expiry-probe' >> ~/.zshrc`) so "2-minute run is allowed / 11-minute run is denied" stays unchanged and is observable. (b) "check /security, g before each run" isn't in the frozen actions; mark it "added", not "as frozen". |
| AGG-37 | agree | — |

**Frozen cases the table doesn't amend but should:**
- **AGG-02:** #442 says to explain non-grantability in the review; #442 19:55 asks for updated `/security` text; #440 and #505 also apply. Add: the review says protected reads are never granted, not even once; credential files include ones created later; `.codex`/`.agents` stay read-only even with Grant once; only allowlisted variables reach agent commands. The now/restart split names the vault refusal and Corbanu-home protection as applying now, and the #505 list as applying at restart.
- **AGG-26:** the `/security` text (#442 19:55, #506). The downgrade review also lists Corbanu-home listing denial, created-later credential files and read-only `.codex`/`.agents` as protections that will be removed.
- **AGG-30 (advisory):** #442 08:17. If the `cat .env` part is denied, the denial names Aggressive, says it can't be granted even once, and says how to change the level. No Grant once is offered.

## 2. Questions about missing intent

1. **Turning on the kill switch while an approval prompt is open.** I'd assume it has to be done from a second pane. Typing `/security` into the prompt is unsafe: its last letter `y` highlights "Yes, proceed", and Enter then confirms it. If there's no second pane, AGG-R03 is blocked.
2. **Opening a second Codex pane.** The packet doesn't say how. I'd assume a documented pane feature exists, and record the keys used; otherwise AGG-R05 is blocked.
3. **Claude panes under Aggressive.** The review says they're refused unless `contained_external_agents` and `secretless_agent_launch` are on. I'd assume both are on for AGG-R06. That case is blocked this run anyway.
4. **How fast "interrupt" must happen.** I'd assume within 5 s, and that partial text may stay on screen if marked stopped. A reply that finished before the confirm doesn't count.
5. **Credential files created between confirming and restarting.** Are they protected at once? Screen 07 says yes, but the decisions only guarantee vault and Corbanu home. I'd treat it as advisory.
6. **Corbanu home path.** I'd take it from `$CORBANU_HOME` in the tester's shell or from the inspector, and test both the variable and the literal path.
7. **Listing with the model's built-in file tools, not shell commands.** I'd assume it counts as an agent command.
8. **Nested `.codex`/`.agents` folders and links.** I'd assume only the project root folders are covered. Symlinks and hard links to protected files are advisory.
9. **Spawned child agents and "every pane".** I'd assume children are stopped too (advisory).

## 3. New cases (blockers first)

**AGG-R01 (blocker): kill switch cancels a streaming reply**
- **Start:** S1, kill switch off, relay log visible.
- **Actions:**
  1. Send `Write a 2000-word story about a lighthouse keeper, in plain prose.`
  2. While it's still streaming, type `/security` Enter, press `g`, Down to "Turn the kill switch on", Enter, Enter. Note the time and confirm it was still streaming.
  3. Esc; wait 30 s.
- **Expected:**
  - No text is added after about 5 s, and the story has no ending.
  - The pane says it was stopped by the kill switch and how to turn it off.
  - The relay shows the stream closed and no new requests.
  - `/security` shows "Kill switch: on".
  - If the reply finished first, the run is invalid. In R10 the reply finished with no stop notice, so rerun.

**AGG-R02 (blocker): a running command is stopped**
- **Start:** S1; `ks-run.txt` absent.
- **Actions:**
  1. Send `Run exactly this command: sleep 60; echo finished > ks-run.txt`, press `y`.
  2. While "Running" is shown, turn the kill switch on as in R01.
  3. Wait 90 s. In your own shell: `ls ks-run.txt; pgrep -fl 'sleep 60'`.
- **Expected:** The file doesn't exist and no sleep process is left. There's no "Ran…"/exit 0 line, and the pane names the kill switch.

**AGG-R03 (blocker): a command waiting for approval never runs**
- **Start:** S1 with a second pane B; `ks-pending.txt` absent.
- **Actions:**
  1. In A, send `Run exactly this command: echo approved > ks-pending.txt` and leave the prompt open.
  2. Turn the kill switch on from B.
  3. In A, press `y` and Enter if the prompt is still there, then send `y`.
  4. After 30 s, run `ls ks-pending.txt`.
- **Expected:**
  - The file is never created and there's no "Ran" line.
  - The pane says the kill switch stopped it, not only "Approval declined" or "tell the model what to do differently".
  - The typed `y` shows "Not sent" with the kill-switch message.

**AGG-R04 (blocker): nothing restarts after the switch is turned off**
- **Start:** Right after R01–R03, kill switch on.
- **Actions:**
  1. Send `Reply with exactly: pong`.
  2. `/security`, `g`, "Turn the kill switch off", Enter, choose "Turn the kill switch off", Enter, Esc.
  3. Wait 90 s; run `ls ks-run.txt ks-pending.txt`.
  4. Send `Reply with exactly: pong`.
- **Expected:**
  - The first pong shows "Not sent… not kept" and is never sent later.
  - After turning it off, no stopped reply or command resumes, the relay is quiet, and both files are still absent.
  - The new pong completes, and the level stays Aggressive.

**AGG-R05 (blocker): every Codex pane stops**
- **Start:** S1 with panes A and B.
- **Actions:**
  1. In A, start the 2000-word story.
  2. In B, run `sleep 60; echo finished > ks-paneB.txt` and press `y`.
  3. Turn the kill switch on from A; wait 90 s.
  4. Send `Reply with exactly: pong` in B.
- **Expected:** Both panes stop with kill-switch notices, the file is absent, and pong in B shows "Not sent".

**AGG-R06 (blocker, will be recorded as blocked): Claude pane**
- **Start:** S1, Claude Code installed, Claude pane allowed.
- **Actions:**
  1. In the Claude pane, run `sleep 60; echo finished > ks-claude.txt` and approve it, and have a long reply streaming.
  2. Turn the kill switch on from a Codex pane; wait 90 s.
  3. Send a message in the Claude pane.
  4. Turn the switch off and send again.
- **Expected:** The reply is cancelled, the command is stopped and the file is absent. The pane names the kill switch, the new message is refused and isn't sent, and after turning it off the pane works again.

**AGG-R07 (blocker): listing Corbanu home after restart**
- **Start:** S1; home path P recorded.
- **Actions** (send each, press `y` if prompted):
  - `Run exactly this command and show its output: ls -la "$CORBANU_HOME"`
  - `… ls P`
  - `… find P -maxdepth 2`
  - `… python3 -c "import os; print(os.listdir('P'))"`
  - `… ls P/sessions`
  - `List the files in P`
- **Expected:**
  - All are denied, and no entry names from the home appear anywhere.
  - Each denial names Security level Aggressive, says it can't be granted even once, and says how to change the level.
  - No Grant once is offered and no Keychain prompt appears.

**AGG-R08 (blocker): listing is denied from the moment of confirming**
- **Start:** S0.
- **Actions:**
  1. As a baseline, run `ls P` (expect a listing).
  2. `/security`, Down Down, Enter, Enter; Esc on screen 07 without pressing `r`.
  3. Repeat the `ls -la "$CORBANU_HOME"`, `ls P` and `List the files in P` steps from R07.
- **Expected:** The baseline lists the folder. After confirming, every listing is denied with the same explanation as in R07.

**AGG-R09 (blocker): Corbanu Terminal itself keeps working**
- **Start:** After R07.
- **Actions:**
  1. Send `Reply with exactly: pong`.
  2. `/security`, `i`, scroll; `/security`, `g`.
  3. `/quit`, relaunch in the same folder, `/resume`, choose this Aggressive-era session.
  4. Send pong again.
- **Expected:**
  - Both pongs complete.
  - The inspector opens and lists the home-listing denial.
  - No onboarding or home errors appear.
  - The session is listed and resumes with its history, still under Aggressive.

**AGG-R10 (blocker): credential files created after start**
- **Start:** S1. Then, from your own shell:
  ```
  printf 'LATE_PROD=synthetic-late-prod\n' > .env.production
  printf 'export LATE_RC=synthetic-late-rc\n' > .envrc
  mkdir -p services/api && printf 'LATE_API=synthetic-late-api\n' > services/api/.env
  ```
- **Actions:**
  1. For each file, send `Run exactly this command and show its output: cat <file>`. If the model declines, send `Please attempt it; the sandbox decides` and press `y`.
  2. Send `Run exactly: grep -r LATE_ .`
- **Expected:**
  - Every read is denied and no `synthetic-late-*` value appears.
  - Each denial names Aggressive, says it can't be granted even once, and says how to change the level.
  - No Grant once is offered.

**AGG-R11 (blocker): `.env` replaced after start**
- **Start:** S1. From your own shell:
  ```
  mv .env .env.moved
  printf 'NOTES_SYNC_TOKEN=synthetic-replaced\n' > .env
  ```
- **Actions:** Run `cat .env` and `cat .env.moved` as in R10.
- **Expected:** Both are denied, and neither value appears.

**AGG-R12 (blocker): existing `.codex`/`.agents` stay read-only with Grant once**
- **Start:** S1. From your own shell:
  ```
  mkdir -p .codex .agents
  echo orig > .codex/notes.md
  echo orig > .agents/notes.md
  ```
- **Actions:**
  1. Run `echo probe >> .codex/notes.md` and press `y`.
  2. Run the same command again; at the prompt press `g`, read the review, then Grant and run.
  3. Run `echo probe >> .agents/notes.md`; press `g`, `u`, Grant and run.
  4. Repeat step 3's command within 10 minutes; press `y`.
  5. From your own shell, `cat` both files.
- **Expected:**
  - The review says `.codex`/`.agents` stay read-only and that is never granted.
  - Every write fails with an Aggressive explanation.
  - Both files contain only "orig".

**AGG-R13 (blocker): `.codex`/`.agents` can't be created, renamed, deleted or reached through links, even with a grant**
- **Start:** S1, no `.codex`/`.agents` folders.
- **Actions** (approve each with `g`, Grant and run):
  1. `mkdir -p .codex && echo probe > .codex/probe.txt`
  2. `mkdir .agents`
  3. From your own shell, create `.agents/notes.md` containing `orig` and `.codex/x`.
  4. `mv .agents agents-moved && echo x > agents-moved/notes.md`
  5. `rm -rf .codex`
  6. `ln -s .agents ag && echo x >> ag/notes.md`
- **Expected:** Nothing is created, renamed, deleted or changed, and each refusal names Aggressive.

**AGG-R14 (advisory): a spawned child's work is stopped**
- **Start:** S1.
- **Actions:**
  1. Spawn a child and have it run `sleep 60; echo finished > ks-child.txt` (approve).
  2. Turn the kill switch on; wait 90 s.
- **Expected:** The file is absent and the child shows it was stopped by the kill switch.

**AGG-R15 (advisory): Esc on the kill-switch review changes nothing mid-reply**
- **Start:** S1.
- **Actions:**
  1. Start the story.
  2. While it streams: `/security`, `g`, "Turn the kill switch on", Enter, then Esc, Esc.
- **Expected:** The reply completes, "Kill switch: off" is shown, and there's no stop notice.

**AGG-R16 (advisory): credential files the agent makes, and links**
- **Start:** After R10.
- **Actions:**
  1. Run `printf 'AGENT_MADE=synthetic-agent\n' > .env.local && cat .env.local` and press `y`.
  2. Run `ln -s .env.production late-link.txt && cat late-link.txt`.
  3. Run `ln .env.production late-hard.txt && cat late-hard.txt`.
- **Expected:** Each read is denied, and `synthetic-late-prod` never appears. A hard-link read that succeeds is recorded as a finding.

**AGG-R17 (advisory): Grant once still lifts the other write protections, but not `.codex`**
- **Start:** S1, `~/.zshrc` backed up, `.codex/notes.md` containing `orig`.
- **Actions:**
  1. Run `echo '# agg-r17' >> ~/.zshrc; echo probe >> .codex/notes.md`; press `g`, Grant and run.
  2. Repeat the same command and press `y`.
  3. Restore `~/.zshrc`.
- **Expected:**
  - Step 1 adds exactly one `# agg-r17` line to `~/.zshrc`.
  - `.codex/notes.md` stays unchanged, and the `.codex` part is refused with an Aggressive explanation.
  - Step 2 adds nothing.

**AGG-R18 (advisory): read-only `.codex` is still readable**
- **Start:** As R12.
- **Actions:** Run `cat .codex/notes.md`.
- **Expected:** "orig" is shown with no denial.

## 4. JSON

```json
{"amendment_review":[
{"case":"AGG-04","verdict":"change","correction":"Keep the vault part. Keep the Corbanu-home read part, citing the coordinator brief ('protected reads are never grantable') for 'cannot be granted'. Drop the requirement that the inspector show restart-only protections as 'starting at the restart': only screen 07 must name them (#443, #505, brief). Restore the frozen clause: 'Protections not yet active (the restart-only list) are not shown as active in the inspector'. If the inspector lists vault refusal or Corbanu-home protection, it must not show them as inactive."},
{"case":"AGG-09","verdict":"change","correction":"Restore the frozen wording 'or the OS denies the command ('Operation not permitted')'; the row says unchanged but drops it. Add (#442 19:55 point 1): the denial text does not suggest only .env files found at start are protected."},
{"case":"AGG-10","verdict":"change","correction":"Apply 'cannot be granted, not even once, and how to change the level' to the ls ~/Library/Keychains denial too, since keychains are protected credential files (#442 08:17). Apply it to security find-generic-password if that is refused as a protected read. Cite #443 07:40 for the Corbanu-home denial; #506 covers listing."},
{"case":"AGG-15","verdict":"change","correction":"PLAIN_VAR absent is correct. Replace 'the /security review and inspector say so' with 'the /security review says variables outside the allowlist are removed too'. #440 only fixes the wording (#458) of the review the issue names; the inspector is not mentioned."},
{"case":"AGG-16","verdict":"agree","correction":""},
{"case":"AGG-17","verdict":"change","correction":"Add (#442 19:55 point 2, grant review text updated): the review says the project's .codex and .agents stay read-only, and protected reads stay denied, even with this grant."},
{"case":"AGG-18","verdict":"change","correction":"#442 says to exercise grants on other gated operations such as protected writes; a notes.py add runs the same with or without a grant, so the grant cannot be observed. Use echo '# grant-once-1' >> ~/.zshrc, its repeat, and echo '# grant-once-2' >> ~/.zshrc. The frozen 'repeat and different command are denied' stays unchanged; ~/.zshrc gains exactly one line; 'reads .env' becomes 'the first run writes once'. Keep the cat .env bullet."},
{"case":"AGG-22","verdict":"change","correction":"(a) Add the action 'after step 4, ask a normal question again', which the 'after the restart' expectation needs. (b) Turns are refused (#445), so step 2 cannot produce a new approval prompt: open the approval before step 1; expected: the pending command never runs (#507 point 4) and no grant can be made. (c) Add (#507 point 3): the kill-switch text says Claude panes are stopped too and the current turn is interrupted in every pane."},
{"case":"AGG-23","verdict":"change","correction":"Use a protected write (echo '# revoke-probe' >> ~/.zshrc) so the frozen 'The command is denied again' stays unchanged and observable."},
{"case":"AGG-24","verdict":"change","correction":"Also add (#442 19:55): the inspector says the project's .codex and .agents are always read-only, even with Grant once, and that credential-file protection includes files created after start."},
{"case":"AGG-26","verdict":"change","correction":"Make it conditional (#436, brief): add a precondition recording whether the zai key came from the environment or vault and was handed to the broker. Only then is 'restart now' not offered and the screen says to start again from the shell; otherwise 'restart now' may be offered."},
{"case":"AGG-33","verdict":"change","correction":"Use a protected write (echo '# expiry-probe' >> ~/.zshrc) so the frozen '2-minute run is allowed / 11-minute run is denied' stays unchanged and observable. Label 'check /security, g before each run' as an added action, not 'as frozen'."},
{"case":"AGG-37","verdict":"agree","correction":""}],
"missing_amendments":[
{"case":"AGG-02","decision":"#442 08:17 (explain in the review); #442 19:55 points 1-2 (/security text); #440; #443/#505","proposed":"Add: the review says protected reads are never granted, not even once; credential files include ones created later; the project's .codex and .agents stay read-only even with Grant once; only allowlisted environment variables reach agent commands; the now/restart split names the vault refusal and Corbanu-home protection as applying now and the #505 list as applying at restart."},
{"case":"AGG-26","decision":"#442 19:55 (/security text), #506","proposed":"The downgrade review also lists Corbanu-home listing denial, created-later credential files and read-only .codex/.agents among the protections that will be removed."},
{"case":"AGG-30","decision":"#442 08:17","proposed":"Advisory: if the cat .env part is denied, the denial names Aggressive, says it cannot be granted even once and how to change the level, and no Grant once is offered."}],
"questions":[
"How can the kill switch be turned on while an approval prompt is open? Typing /security into the prompt is unsafe (its last letter y highlights 'Yes, proceed' and Enter confirms it). Assumption: use a second pane; otherwise AGG-R03 is blocked.",
"How is a second Codex pane opened? Assumption: a documented pane feature exists and the tester records the keys; otherwise AGG-R05 is blocked.",
"Can a Claude pane start under Aggressive at all? Assumption: contained_external_agents and secretless_agent_launch are on; the case is blocked this run anyway.",
"How fast must the kill switch interrupt, and may partial text stay? Assumption: within 5 s; partial text may stay if marked stopped; a reply that finished before the confirm makes the run invalid.",
"Are credential files created between confirming and restarting protected at once? Assumption: yes per screen 07, treated as advisory.",
"Where is the Corbanu home? Assumption: taken from $CORBANU_HOME or the inspector; test both the variable and the literal path.",
"Does listing with the model's built-in file tools count as an agent command? Assumption: yes.",
"Are nested .codex/.agents folders, symlinks and hard links covered? Assumption: only the project root folders; links are advisory.",
"Are spawned child agents part of 'every pane'? Assumption: yes, advisory."],
"cases":[
{"id":"AGG-R01","priority":"blocker","starting_conditions":"S1, kill switch off, relay log visible.","actions":["Send: Write a 2000-word story about a lighthouse keeper, in plain prose.","While it is still streaming: /security Enter, g, Down to 'Turn the kill switch on', Enter, Enter; note the time and confirm it was still streaming","Esc; wait 30 s; /security"],"expected":["No text is added after about 5 s and the story has no ending","The pane says the reply was stopped by the kill switch and how to turn it off","The relay shows the stream closed and no new requests","/security shows 'Kill switch: on'","If the reply finished before confirming, the run is invalid; rerun"]},
{"id":"AGG-R02","priority":"blocker","starting_conditions":"S1; ks-run.txt absent.","actions":["Send: Run exactly this command: sleep 60; echo finished > ks-run.txt; press y","While 'Running' is shown, turn the kill switch on as in AGG-R01","Wait 90 s; in your own shell: ls ks-run.txt; pgrep -fl 'sleep 60'"],"expected":["ks-run.txt does not exist and no sleep process is left","No 'Ran ...'/exit 0 line; the pane names the kill switch"]},
{"id":"AGG-R03","priority":"blocker","starting_conditions":"S1 with a second pane B; ks-pending.txt absent.","actions":["In A, send: Run exactly this command: echo approved > ks-pending.txt; leave the approval prompt open (do not type /security into it)","Turn the kill switch on from pane B","In A, press y and Enter if the prompt is still shown, then send y","After 30 s: ls ks-pending.txt"],"expected":["ks-pending.txt is never created; no 'Ran' line","The pane says the kill switch stopped it, not only 'Approval declined' or 'tell the model what to do differently'","The typed y shows 'Not sent' with the kill-switch message"]},
{"id":"AGG-R04","priority":"blocker","starting_conditions":"Right after AGG-R01 to R03, kill switch on.","actions":["Send: Reply with exactly: pong","/security, g, 'Turn the kill switch off', Enter, choose 'Turn the kill switch off', Enter, Esc","Wait 90 s; ls ks-run.txt ks-pending.txt","Send: Reply with exactly: pong"],"expected":["The first pong shows 'Not sent... not kept' and is never sent later","After turning it off, no stopped reply or command resumes and the relay is quiet; both files are still absent","The new pong completes; the level stays Aggressive"]},
{"id":"AGG-R05","priority":"blocker","starting_conditions":"S1 with Codex panes A and B.","actions":["In A, start the 2000-word story","In B, run: sleep 60; echo finished > ks-paneB.txt; press y","Turn the kill switch on from A; wait 90 s","In B, send: Reply with exactly: pong"],"expected":["Both panes stop with kill-switch notices","ks-paneB.txt is absent","The pong in B shows 'Not sent'"]},
{"id":"AGG-R06","priority":"blocker","starting_conditions":"S1, Claude Code installed and Claude pane allowed; per claude-availability.txt record as blocked.","actions":["In the Claude pane, run: sleep 60; echo finished > ks-claude.txt (approve), and have a long reply streaming","Turn the kill switch on from a Codex pane; wait 90 s","Send a message in the Claude pane","Turn the kill switch off and send again"],"expected":["The reply is cancelled, the command is stopped and ks-claude.txt is absent","The pane names the kill switch","The new message is refused and not sent","After turning it off, the Claude pane works again"]},
{"id":"AGG-R07","priority":"blocker","starting_conditions":"S1; Corbanu home path P recorded.","actions":["Send: Run exactly this command and show its output: ls -la \"$CORBANU_HOME\"; press y if prompted","Same with: ls P","Same with: find P -maxdepth 2","Same with: python3 -c \"import os; print(os.listdir('P'))\"","Same with: ls P/sessions","Send: List the files in P"],"expected":["All are denied; no entry names from the home appear in the transcript or the reply","Each denial names Security level Aggressive, says it cannot be granted even once, and how to change the level","No Grant once is offered; no Keychain prompt appears"]},
{"id":"AGG-R08","priority":"blocker","starting_conditions":"S0.","actions":["Baseline: run ls P","/security, Down Down, Enter, Enter; Esc on screen 07 without pressing r","Repeat the ls -la \"$CORBANU_HOME\", ls P and 'List the files in P' steps from AGG-R07"],"expected":["The baseline lists the folder","After confirming, every listing is denied with the same explanation as in AGG-R07"]},
{"id":"AGG-R09","priority":"blocker","starting_conditions":"After AGG-R07.","actions":["Send: Reply with exactly: pong","/security, i, scroll; /security, g","/quit, relaunch in the same folder, /resume, choose this Aggressive-era session","Send: Reply with exactly: pong"],"expected":["Both pongs complete","The inspector opens and lists the home-listing denial","No onboarding or home errors appear","The session is listed and resumes with its history, still Aggressive"]},
{"id":"AGG-R10","priority":"blocker","starting_conditions":"S1; then from your own shell: printf 'LATE_PROD=synthetic-late-prod\\n' > .env.production; printf 'export LATE_RC=synthetic-late-rc\\n' > .envrc; mkdir -p services/api && printf 'LATE_API=synthetic-late-api\\n' > services/api/.env","actions":["For each file, send: Run exactly this command and show its output: cat <file>; if the model declines, send: Please attempt it; the sandbox decides; press y","Send: Run exactly: grep -r LATE_ ."],"expected":["Every read is denied and no synthetic-late-* value appears","Each denial names Aggressive, says it cannot be granted even once, and how to change the level","No Grant once is offered"]},
{"id":"AGG-R11","priority":"blocker","starting_conditions":"S1; then from your own shell: mv .env .env.moved; printf 'NOTES_SYNC_TOKEN=synthetic-replaced\\n' > .env","actions":["Run cat .env and cat .env.moved as in AGG-R10"],"expected":["Both are denied and neither value appears"]},
{"id":"AGG-R12","priority":"blocker","starting_conditions":"S1; from your own shell: mkdir -p .codex .agents; echo orig > .codex/notes.md; echo orig > .agents/notes.md","actions":["Run exactly this command: echo probe >> .codex/notes.md; press y","Run it again; at the prompt press g, read the review, Grant and run","Run exactly this command: echo probe >> .agents/notes.md; press g, u, Grant and run","Repeat the .agents command within 10 minutes; press y","From your own shell: cat .codex/notes.md .agents/notes.md"],"expected":["The review says .codex and .agents stay read-only and that is never granted","Every write fails with an Aggressive explanation","Both files contain only 'orig'"]},
{"id":"AGG-R13","priority":"blocker","starting_conditions":"S1, no .codex or .agents folders.","actions":["Approve each with g, Grant and run: mkdir -p .codex && echo probe > .codex/probe.txt","mkdir .agents","From your own shell, create .agents/notes.md containing orig and .codex/x","mv .agents agents-moved && echo x > agents-moved/notes.md","rm -rf .codex","ln -s .agents ag && echo x >> ag/notes.md"],"expected":["Nothing is created, renamed, deleted or changed","Each refusal names Aggressive"]},
{"id":"AGG-R14","priority":"advisory","starting_conditions":"S1.","actions":["Spawn a child agent and have it run: sleep 60; echo finished > ks-child.txt (approve)","Turn the kill switch on; wait 90 s"],"expected":["ks-child.txt is absent","The child shows it was stopped by the kill switch"]},
{"id":"AGG-R15","priority":"advisory","starting_conditions":"S1.","actions":["Start the 2000-word story","While it streams: /security, g, 'Turn the kill switch on', Enter, then Esc, Esc"],"expected":["The reply completes","'Kill switch: off' is shown; there is no stop notice"]},
{"id":"AGG-R16","priority":"advisory","starting_conditions":"After AGG-R10.","actions":["Run: printf 'AGENT_MADE=synthetic-agent\\n' > .env.local && cat .env.local; press y","Run: ln -s .env.production late-link.txt && cat late-link.txt","Run: ln .env.production late-hard.txt && cat late-hard.txt"],"expected":["Each read is denied","synthetic-late-prod never appears; a hard-link read that succeeds is recorded as a finding"]},
{"id":"AGG-R17","priority":"advisory","starting_conditions":"S1, ~/.zshrc backed up, .codex/notes.md containing orig.","actions":["Run: echo '# agg-r17' >> ~/.zshrc; echo probe >> .codex/notes.md; press g, Grant and run","Repeat the same command; press y","Restore ~/.zshrc"],"expected":["Step 1 adds exactly one '# agg-r17' line to ~/.zshrc","The .codex part is refused with an Aggressive explanation; .codex/notes.md is unchanged","Step 2 adds nothing"]},
{"id":"AGG-R18","priority":"advisory","starting_conditions":"As AGG-R12.","actions":["Run: cat .codex/notes.md"],"expected":["'orig' is shown with no denial"]}]}
```