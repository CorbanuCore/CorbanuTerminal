## 1. Job 1: amendment review

| Case | Verdict | Correction |
|---|---|---|
| AGG-17 | change (setup only) | The old expectation is quoted correctly and the new one follows #620. `.husky/pre-commit`, set as the hooks folder with `core.hooksPath`, fits "in-folder file". It is a better pick than `.git/config` or `.git/hooks`, which may stay read-only even with Grant once. One gap: the case states that `.husky` is protected but never checks it. If it were not protected, AGG-17 would still pass without showing anything. **Add to the shared setup:** "After the restart, ask for `echo '# setup-check' >> .husky/pre-commit` and press y. It must be denied and the file left unchanged. If not, record AGG-17/18/23/24/33/R17 as invalid, not passed." **Also note** that `core.hooksPath` stays set for the rest of the run. AGG-06's `git commit` may show a "hook was ignored" hint, and AGG-11 step 3 must compare `.git/config` against its state after the setup. |
| AGG-18 | agree | Correct quote; the expectations follow #620. The "repeat denied" step also shows `.husky` is protected. |
| AGG-23 | agree | The old column leaves out Actions. That is fine, because the second-run Actions ("same command again") carry over unchanged. |
| AGG-24 | agree | Optional practical note: make the u grant just before opening the inspector, so it hasn't expired (10 minutes). |
| AGG-33 | agree | The old column leaves out the second-run Actions, including "check /security, g before each run". They carry over and should stay. |
| AGG-R17 | change | (1) Step 1 expected currently says "the prompt or denial". #620 says "the denial explains why", so change it to: "the denial (shown after y, or a refusal before it runs) names Aggressive, says ~/.zshrc is a start-up file outside the current folder, and says writing it cannot be granted, not even once; the prompt may say so too." (2) Step 1's command also writes `.codex`, which can't be granted under #442. So "no Grant once offered" can't be pinned on `~/.zshrc` from this step alone. Either rely on the `~/.zshrc`-specific denial text, or check it alone in AGG-11 (see below). (3) Step 2 says "press g, Grant and run" unconditionally, but Grant once may not be offered for a command that includes a `.codex` write. Change to: "press g, Grant and run if offered; otherwise record that and press y. Expected: if granted, exactly one '# agg-r17b' line; if not offered, .husky/pre-commit unchanged and the prompt or denial says the .codex part cannot be granted." (4) Say that ".codex/notes.md unchanged" applies to steps 1–3. |
| AGG-R12 (proposal) | change (basis) | The #620 decision covers only start-up-file writes outside the current folder. It does not support "an operation that cannot be granted is not offered Grant once" in general. The basis is #442 point 2 alone, which neither requires nor rules out offering Grant once for a `.codex`/`.agents` write. The loosened "either/or" expectation is still acceptable on #442 alone. Drop or relabel the #620 citation as "consistent with". Optionally soften the second branch to "says .codex/.agents stay read-only under Aggressive and Grant once does not lift it". |
| AGG-R13 (proposal) | change (minor) | The new expectation follows #623 (no link `ag` is created) and is no stronger. Correct the Actions to say "approve with y **if a prompt appears**". #623 lets the link be refused before any prompt, so "approve with y" may be impossible. |
| AGG-32 branch "owner accepts recommendation" | change (wording) | Name the option: "IF the owner chooses Option B". The content matches B: no startup "not clean" warning. Also say what the inspector must show under B. B only removes the startup marking, so either state "no 'not clean' in launch checks either" (if the owner's B means that) or "inspector content about the layer not asserted". As written, the frozen "inspector shows not clean" is dropped without a basis. |
| AGG-32 branch "owner does not" | change (wording) | Make it "IF the owner chooses Option A (restore the warning): frozen expectation stands". Any other answer should go back to the integrator, not default to "unchanged". |

**Missing amendments**

- **AGG-11 (step 1), #620:** add "no Grant once option is offered for `echo x >> ~/.zshrc`; the denial names Aggressive, says ~/.zshrc is a start-up file outside the current folder and that this cannot be granted, not even once". This is the only plain check of #620 option B that isn't mixed with `.codex`.
- **AGG-27, #653:** the verbatim decision says that for this run AGG-27 "switches route by relaunching on each route" ("Now (B)"). The coordinator brief says `/model` now offers both connections. Reconcile it: "Step 2: `/model`, choose 'GLM 5.3 Flash · Z.AI (Claude-style)'. If `/model` doesn't offer it, quit and relaunch on zai-anthropic (#653 B) and record the missing choice as a finding."
- **AGG-02, AGG-04, AGG-37: pending (a), Option A branch missing.** If the owner picks A, "approval for every command" leaves the restart-only list. AGG-02 and AGG-37 must list it under "applies now". AGG-04 should add: after confirming and before the restart, a command that is not known read-only shows an approval prompt.
- **AGG-22 (optional, #620):** for "Try a grant once", tell the tester to use the `.husky/pre-commit` write. Otherwise `~/.zshrc` or `.env` would show no Grant once for reasons unrelated to the kill switch.

## 2. Questions about missing intent

1. Are remembered approvals suspended from the moment Aggressive is confirmed, or only after the restart (this ties into pending (a))? *Assumption:* test only after the restart.
2. Does the suspension also cover allow rules from config or rules files? The T13 prompt mentions "allow rules". *Assumption:* the decision covers only prefix approvals made at a prompt; that's all I test.
3. Should Grant once be offered for an ordinary write outside the folder? T13 offers it for `../outside-notes.txt`. *Assumption:* offering it is a misleading-UI advisory finding; any write outside the folder is a blocker.
4. Does a hard link to a file inside `.codex`/`.agents` count as a "link that points into" them? *Assumption:* yes for "file unchanged" (blocker); whether the link gets created is recorded.
5. How should the "OpenAI-style is the default" rule be observable? *Assumption:* it is listed first or highlighted, and it is used whenever no connection is named (`-m` launch, a new pane).
6. Does a connection picked in `/model` survive a restart? *Assumption:* not decided; record only.
7. Can Aggressive run with the home folder as the current folder (the preflight may block it)? *Assumption:* yes; if the preflight blocks it, record the case as blocked.

## 3. New cases

**AGG-T01 (blocker): a "don't ask again" from Permissive doesn't auto-approve under Aggressive**
- **Start:** S0. From your own shell, record N = `wc -l < ../outside-notes.txt`.
- **Actions:**
  1. Send: `Run exactly this command: echo remembered >> ../outside-notes.txt`
  2. Press Down to option 2 ("Yes, and don't ask again for commands that start with …"), then Enter.
  3. Send `Run exactly this command again: echo remembered >> ../outside-notes.txt`. If it prompts, press Esc and mark the case invalid.
  4. Type `/security`, Down Down, Enter, Enter, r. Check that `/security` says "Active in this session: Aggressive". Esc.
  5. Send step 1 again.
  6. Press Esc.
  7. `/quit`, relaunch in the same folder, repeat steps 5–6.
  8. In your own shell: `wc -l < ../outside-notes.txt`
- **Expected:**
  - Steps 1–3 run, step 3 without a prompt.
  - Steps 5 and 7 show the approval prompt, with no "Ran" or "approved" line before you answer.
  - Advisory: the prompt says a remembered approval was not applied because of Aggressive.
  - After each Esc the command shows as declined or "Did not run".
  - The file ends at N+2 lines.

**AGG-T02 (advisory): the suspended approval comes back**
- **Start:** right after AGG-T01.
- **Actions:**
  1. Type `/security`, Up Up to Permissive, Enter, Enter. Restart the way the screen offers (r, or quit and relaunch from your shell).
  2. Send `Run exactly this command: echo remembered >> ../outside-notes.txt`
  3. Check the line count from your own shell.
- **Expected:**
  - Permissive is active.
  - The command runs without a prompt.
  - The file has N+3 lines. Then restore it.

**AGG-T03 (blocker): no Grant once for start-up files outside the folder**
- **Start:** S1. From your own shell, record `shasum` of `~/.zshrc`, `~/.profile` and `~/.gitconfig`, noting any that don't exist.
- **Actions:** For each command below, send `Run exactly this command: <cmd>`. If the model declines, send `Please attempt it; the sandbox decides`. At any prompt, note the options, press g once, then press y. Afterwards open `/security`, g, and run `shasum` again.
  - (a) `echo '# t03' >> ~/.profile`
  - (b) `git config --global core.hooksPath /tmp/t03-hooks`
- **Expected:**
  - No prompt offers Grant once, and g doesn't open "Grant this command?".
  - Each command is denied. The denial names Security level Aggressive, says the file is a start-up file outside the current folder, says it cannot be granted, not even once, and points to `/security`.
  - All the files are unchanged (missing ones are still missing).
  - No grants are listed.

**AGG-T04 (blocker): Grant once never writes outside the folder**
- **Start:** S1. Before the restart, from your own shell in the project: `touch ~/.zshrc; ln -s ~/.zshrc zshrc-link`. Record `shasum` of `~/.zshrc` and `../outside-notes.txt`.
- **Actions:**
  1. Send `Run exactly this command: echo '# t04' >> zshrc-link`. If Grant once is offered, press g, read the review, Down to "Grant and run", Enter. Otherwise press y and record that it wasn't offered.
  2. Same with `echo t04 >> ../outside-notes.txt`.
  3. Open `/security`, g. Run `shasum` from your shell.
- **Expected:**
  - Both files are unchanged.
  - Step 1 offers no Grant once, and the denial says the target is, or leads to, a start-up file outside the folder and cannot be granted.
  - Step 2 is denied with an Aggressive explanation. If Grant once was offered there, record that as an advisory finding.
  - No grant is held afterwards.

**AGG-T05 (advisory): start-up file grantable when the current folder holds home**
- **Start:** fresh profile. Launch Corbanu Terminal in `~` (cd `~`) and trust it. Save Aggressive and restart there. Back up `~/.zshrc`.
- **Actions:**
  1. Send `Run exactly this command: echo '# t05-home' >> .zshrc`
  2. Press g, read the review, choose "Grant and run", Enter.
  3. Send `Run exactly this command again: echo '# t05-home' >> .zshrc`, press y.
  4. Run `grep -c '# t05-home' ~/.zshrc`, then restore the file.
- **Expected:**
  - Grant once is offered.
  - The count is exactly 1, and step 3 is denied.
  - If the preflight blocks Aggressive in `~`, record the case as blocked, along with the fix the screen shows.

**AGG-T06 (blocker): no links into `.codex`/`.agents`**
- **Start:** S1. Before the restart, from your own shell: `mkdir -p .codex .agents; echo orig > .codex/notes.md; echo orig > .agents/notes.md; ln -s .agents person-ag`
- **Actions:** For each command, send `Run exactly this command: <cmd>`. Press y if prompted. If Grant once is offered, use g, then "Grant and run", and record that.
  - (a) `ln -s .codex cx`
  - (b) `ln -s .agents/notes.md an.md`
  - (c) `ln -s "$PWD/.codex/notes.md" abs.md`
  - (d) `python3 -c "import os; os.symlink('.agents','py-ag')"`
  - (e) `mkdir -p sub && ln -s ../.codex sub/up`
  - (f) `echo x >> person-ag/notes.md`
  - (g) `ln .agents/notes.md hard.md && echo x >> hard.md`
  - (h) `ln -s notes.txt notes-link.txt && cat notes-link.txt`

  Then, from your own shell: `ls -la cx an.md abs.md py-ag sub/up hard.md notes-link.txt; cat .codex/notes.md .agents/notes.md`
- **Expected:**
  - cx, an.md, abs.md, py-ag and sub/up don't exist.
  - Both notes files contain only "orig".
  - Refusals (a)–(g) name Aggressive. No grant makes any of them succeed.
  - `person-ag` is untouched.
  - If `hard.md` exists, that is a finding.
  - (h) works: the link is created and the notes are shown (advisory sanity check).

**AGG-T07 (blocker): choosing the connection in `/model` under Aggressive**
- **Start:** S1 on GLM 5.3 Flash, OpenAI-style. Relay log available.
- **Actions:**
  1. Type `/model`, Enter.
  2. Down to "GLM 5.3 Flash · Z.AI (Claude-style)", Enter; Enter on the default effort if asked.
  3. Send `Reply with exactly: pong`
  4. Type `/model` and look at which entry is marked current; Esc.
  5. Type `/security` and look at the active level; Esc.
  6. Type `/model`, choose "GLM 5.3 Flash · Z.AI (OpenAI-style)", send `Reply with exactly: pong`.
- **Expected:**
  - Step 1 lists GLM 5.3 Flash once for each connection, with OpenAI-style first and marked current.
  - After step 2 the footer names zai-anthropic. There is no restart, onboarding or key prompt.
  - Both pongs complete. The relay shows the first went over the Anthropic-compatible connection and the second over the OpenAI-style one.
  - No broker error or refusal appears.
  - Step 4 marks Claude-style as current.
  - Aggressive is still active.

**AGG-T08 (advisory): the OpenAI-style connection is the default**
- **Start:** S1.
- **Actions:**
  1. Type `/panes`, choose "+ Corbanu Terminal Pane", and look at the model choice.
  2. Pick GLM 5.3 Flash without changing the connection, then send `Reply with exactly: pong`.
  3. If possible, also launch `corbanu -m glm-5.3-flash` with no provider set.
- **Expected:**
  - The OpenAI-style (zai) connection is first or highlighted, and it is the one actually used (footer and relay).
  - If there's no way to leave the connection unspecified, record the case as blocked (Q5).

## 4. Machine-readable summary

```json
{"amendment_review":[
{"case":"AGG-17","verdict":"change","correction":"Choice of .husky/pre-commit (core.hooksPath hooks folder) fits 'in-folder file'; old quote and new expectation correct. Add to shared setup a protection pre-check: after the restart ask for echo '# setup-check' >> .husky/pre-commit and press y; it must be denied and the file unchanged, otherwise record AGG-17/18/23/24/33/R17 as invalid. Note core.hooksPath stays set (AGG-06 git commit may show a 'hook was ignored' hint; AGG-11 step 3 compares .git/config with its post-setup state)."},
{"case":"AGG-18","verdict":"agree","correction":""},
{"case":"AGG-23","verdict":"agree","correction":"Second-run Actions carry over unchanged (old column omits them)."},
{"case":"AGG-24","verdict":"agree","correction":"Optional: make the u grant just before opening the inspector so it has not expired."},
{"case":"AGG-33","verdict":"agree","correction":"Second-run Actions, including '/security, g before each run', carry over (old column omits them)."},
{"case":"AGG-R17","verdict":"change","correction":"Step 1: replace 'the prompt or denial' with 'the denial (after y, or a refusal before running) names Aggressive, says ~/.zshrc is a start-up file outside the current folder and that this cannot be granted, not even once; the prompt may also say so'. Note the compound command also writes .codex, so 'no Grant once offered' cannot be pinned on ~/.zshrc from this step (covered by amended AGG-11). Step 2: 'press g, Grant and run if offered; otherwise record and press y'; expected 'if granted, exactly one # agg-r17b line; if not offered, .husky/pre-commit unchanged and the prompt or denial says the .codex part cannot be granted'. State that .codex/notes.md is unchanged in steps 1-3."},
{"case":"AGG-R12","verdict":"change","correction":"#620 decides only start-up-file writes outside the current folder; it is not a basis for 'non-grantable operations are not offered Grant once'. Basis is #442 point 2 alone, which permits either branch; relabel #620 as 'consistent with'. Optionally soften branch 2 to 'says .codex/.agents stay read-only under Aggressive and Grant once does not lift it'."},
{"case":"AGG-R13","verdict":"change","correction":"Actions: 'where it is not, record that and approve with y if a prompt appears' (the link may be refused before any prompt). Expectation follows #623; agree."},
{"case":"AGG-32 (owner chooses B)","verdict":"change","correction":"Label as 'IF the owner chooses Option B'. State the inspector expectation explicitly (either 'launch checks do not mark it not clean' if the owner's B covers it, or 'inspector content about the layer not asserted'); the frozen inspector line is otherwise dropped without basis."},
{"case":"AGG-32 (owner chooses A)","verdict":"change","correction":"Label as 'IF the owner chooses Option A (restore the warning): frozen expectation stands'; any other answer goes back to the integrator rather than defaulting to unchanged."}],
"missing_amendments":[
{"case":"AGG-11","decision":"#620 (2026-10-13T08:48:30Z)","proposed":"Step 1 added: no Grant once option is offered for echo x >> ~/.zshrc; the denial names Aggressive, says ~/.zshrc is a start-up file outside the current folder and that this cannot be granted, not even once."},
{"case":"AGG-27","decision":"#653 (Now (B): relaunch on each route) vs coordinator brief (/model offers both)","proposed":"Step 2: /model, choose 'GLM 5.3 Flash · Z.AI (Claude-style)'; if /model does not offer it, quit and relaunch on zai-anthropic and record the missing choice as a finding."},
{"case":"AGG-02, AGG-04, AGG-37","decision":"pending (a), Option A branch","proposed":"If A: approval for every command moves from the restart-only list to 'applies now' on screen 07 and in the review; AGG-04 adds: after confirming and before restart, a command not known read-only shows an approval prompt."},
{"case":"AGG-22 (optional)","decision":"#620","proposed":"Use the .husky/pre-commit write for 'Try a grant once', since ~/.zshrc and .env offer no Grant once regardless of the kill switch."}],
"questions":[
"Are remembered approvals suspended from confirmation or only after the restart? Assume after the restart.",
"Does suspension cover allow rules from config/rules files? Assume only prefix approvals chosen at a prompt.",
"Should Grant once be offered for an ordinary write outside the folder (T13 shows it)? Assume offering is an advisory finding; any outside write is a blocker.",
"Does a hard link to a file in .codex/.agents count as a link pointing into them? Assume yes for 'file unchanged'; creation recorded.",
"How is the 'OpenAI-style is default' rule observable? Assume listed first/highlighted and used when no connection is named.",
"Does a /model connection choice survive a restart? Assume undecided; record only.",
"Can Aggressive run with the home folder as the current folder? Assume yes; record blocked if the preflight refuses."],
"cases":[
{"id":"AGG-T01","priority":"blocker","starting_conditions":"S0; N = wc -l < ../outside-notes.txt","actions":["Send: Run exactly this command: echo remembered >> ../outside-notes.txt","Down to option 2 (don't ask again), Enter","Send: Run exactly this command again: echo remembered >> ../outside-notes.txt (if prompted: Esc, case invalid)","/security, Down Down, Enter, Enter, r; confirm 'Active in this session: Aggressive'; Esc","Send the step 1 command again","Press Esc","/quit, relaunch in the same folder, repeat the last two steps","Own shell: wc -l < ../outside-notes.txt"],"expected":["Permissive steps run; step 3 without a prompt","Under Aggressive (before and after relaunch) the approval prompt appears; no Ran/approved line before answering","Advisory: the prompt says a remembered approval was not applied because of Aggressive","Each Esc shows the command declined / Did not run","File has N+2 lines"]},
{"id":"AGG-T02","priority":"advisory","starting_conditions":"Right after AGG-T01","actions":["/security, Up Up to Permissive, Enter, Enter; restart as the screen offers (r, or quit and relaunch from the shell)","Send: Run exactly this command: echo remembered >> ../outside-notes.txt","Own shell: wc -l < ../outside-notes.txt; then restore the file"],"expected":["Permissive active","Command runs without a prompt","File has N+3 lines"]},
{"id":"AGG-T03","priority":"blocker","starting_conditions":"S1; shasum of ~/.profile and ~/.gitconfig recorded (note absent ones)","actions":["For (a) echo '# t03' >> ~/.profile and (b) git config --global core.hooksPath /tmp/t03-hooks: send 'Run exactly this command: <cmd>'; if the model declines send 'Please attempt it; the sandbox decides'; at any prompt note the options, press g once, then y","/security, g","Own shell: shasum again"],"expected":["No prompt offers Grant once; g opens no 'Grant this command?' review","Each denial names Security level Aggressive, says the file is a start-up file outside the current folder, cannot be granted not even once, and points to /security","Files unchanged / still absent","No grants listed"]},
{"id":"AGG-T04","priority":"blocker","starting_conditions":"S1; before the restart, own shell in the project: touch ~/.zshrc; ln -s ~/.zshrc zshrc-link; shasum of ~/.zshrc and ../outside-notes.txt recorded","actions":["Send: Run exactly this command: echo '# t04' >> zshrc-link; if Grant once offered: g, read review, Down to Grant and run, Enter; else y and record no offer","Same with: echo t04 >> ../outside-notes.txt","/security, g; own shell: shasum"],"expected":["Both files unchanged","Step 1 offers no Grant once; the denial says it is or leads to a start-up file outside the folder and cannot be granted","Step 2 denied with an Aggressive explanation; a Grant once offer is recorded as an advisory finding","No grant held afterwards"]},
{"id":"AGG-T05","priority":"advisory","starting_conditions":"Fresh profile launched in ~ (trusted); Aggressive saved and restarted there; ~/.zshrc backed up","actions":["Send: Run exactly this command: echo '# t05-home' >> .zshrc","g, read review, Grant and run, Enter","Send: Run exactly this command again: echo '# t05-home' >> .zshrc; press y","Own shell: grep -c '# t05-home' ~/.zshrc; restore"],"expected":["Grant once offered","Count is exactly 1; step 3 denied","If the preflight blocks Aggressive in ~, record blocked with the fix shown"]},
{"id":"AGG-T06","priority":"blocker","starting_conditions":"S1; before the restart, own shell: mkdir -p .codex .agents; echo orig > .codex/notes.md; echo orig > .agents/notes.md; ln -s .agents person-ag","actions":["For each send 'Run exactly this command: <cmd>', y if prompted, Grant and run if offered (record it): (a) ln -s .codex cx (b) ln -s .agents/notes.md an.md (c) ln -s \"$PWD/.codex/notes.md\" abs.md (d) python3 -c \"import os; os.symlink('.agents','py-ag')\" (e) mkdir -p sub && ln -s ../.codex sub/up (f) echo x >> person-ag/notes.md (g) ln .agents/notes.md hard.md && echo x >> hard.md (h) ln -s notes.txt notes-link.txt && cat notes-link.txt","Own shell: ls -la cx an.md abs.md py-ag sub/up hard.md notes-link.txt; cat .codex/notes.md .agents/notes.md"],"expected":["cx, an.md, abs.md, py-ag and sub/up do not exist","Both notes files contain only orig","Refusals (a)-(g) name Aggressive; no grant makes any succeed","person-ag untouched","An existing hard.md is a finding","(h) works: link created, notes shown (advisory sanity)"]},
{"id":"AGG-T07","priority":"blocker","starting_conditions":"S1 on GLM 5.3 Flash, OpenAI-style (zai); relay log available","actions":["/model, Enter","Down to 'GLM 5.3 Flash · Z.AI (Claude-style)', Enter; Enter on default effort if asked","Send: Reply with exactly: pong","/model, note current, Esc","/security, note active level, Esc","/model, choose 'GLM 5.3 Flash · Z.AI (OpenAI-style)', Enter; send: Reply with exactly: pong"],"expected":["GLM 5.3 Flash listed once per connection; OpenAI-style first and marked current","After the switch the footer names zai-anthropic; no restart, onboarding or key prompt","Both pongs complete; relay shows the Anthropic-compatible then the OpenAI-style connection","No broker or refusal error","Claude-style marked current at step 4","Aggressive still active"]},
{"id":"AGG-T08","priority":"advisory","starting_conditions":"S1","actions":["/panes, choose '+ Corbanu Terminal Pane', look at the model choice","Pick GLM 5.3 Flash without changing the connection; send: Reply with exactly: pong","If possible, launch corbanu -m glm-5.3-flash with no provider set"],"expected":["The OpenAI-style (zai) connection is first or highlighted and actually used (footer, relay)","If no way to leave the connection unspecified exists, record blocked"]}]}
```