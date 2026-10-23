I've written amendment v3 for the 17 re-run cases. I wrote nothing to disk.

- **Amended (13):** MOD-01, 06, 08, 09, 10, 15, 21, 22, 26, 28 and 29 change behaviour or wording. MOD-32 gets only a note, and G-2 is reworded only.
- **Re-run as in force (6):** MOD-17, 20, 24, 30, 32 and 33.
- **Not run:** MOD-11 and MOD-06's git-commit part are recorded as "pending owner decision (D5, #837)". The 16 cases that passed keep their first-run results.

## (a) Amendment v3

```markdown
# Amendments to the Moderate design (amendments-v3, targeted re-run)

Applied on top of `design-cases.md` + `amendments-v2.md`, for the 17 re-run cases only (scope.md). For an amended
case this amendment governs where it differs; everything it does not mention stays as in force (frozen text plus
amendments-v2). Decision labels refer to `decisions.md`: A(n) = the owner's 13 decisions of 2026-10-15; B = #739
option B; C = #822 refusal wording; D = the coordinator's brief of 2026-10-16; E = earlier decisions still in force;
F = integrator choices.

## Rules for this re-run

- **G-1** (#447 model-decline rule): unchanged.
- **G-2** (D4, decided; E) (replace the v2 text with): Expected results that depend on what applies at confirmation
  (MOD-02, MOD-03, MOD-32) are judged against the D4 reading the owner kept: confirmation raises Core's whole level at
  once (sandbox rule, protected files and untrusted-content rules apply at confirmation); the credential broker and
  the 'active' label wait for the restart. This reading is decided; it is no longer open or a named limit. What G-2
  judges is unchanged. (MOD-29's `.env` timing under Aggressive follows A(9), not G-2.)
- **G-3** (#766, open): unchanged; still a named limit.
- **Named limits** (amend): remove "D4 reading (open): see G-2." All other named limits of amendments-v2 stay,
  including MOD-22's 'restart now' limit.
- **R-1 Model rule for MOD-09 and MOD-21** (A(13); F): the product's model is GLM 5.2 on Z.AI. Launch with
  `agms-tui start --model glm-5.2` (`r` restarts with the same options; any relaunch in these cases repeats
  `--model glm-5.2`). The tester model stays GLM 5.3 Flash. Record the model the product shows (header
  `model: glm-5.2`), and for MOD-21 also any model shown for the child. G-1 applies: the tester may resubmit a declined
  request once, unchanged, and labels it. This is the single re-run A(13) allows: if a case is blocked again under
  G-1, it is recorded as blocked, with no further re-run.
- **R-2 Refusal wording** (C): "a refusal nobody was asked about" means an agent command the product did not run,
  with no approval prompt shown for it. Where a case below adds C, judge only that this command's cell is not labelled
  "Approval declined". Whether it says "Refused" or shows the refusal text is not judged (#900 depends on the build).
  A cell for a prompt the tester answered with Esc or No may still say declined.
- **R-3 Protected launch under Permissive** (B), test-machine fact: the run's config has protected launch on. Under
  Permissive the product therefore does not offer approval to run outside the sandbox (screens 01, 12). Rules saved
  with "don't ask again" for such requests are created only in a launch with protected launch off:
  `agms-tui start -c features.secretless_agent_launch=false -c allow_login_shell=false` (MOD-08).

## Case amendments

### MOD-01 (amended; A(12), B, C)

- Action 3 (replace): Ask the agent: "Append the line 'esc check' to ../outside-notes.txt; request my approval to run
  outside the sandbox." If an approval prompt appears anyway, record it and approve with Enter on option 1.
- New action after action 3: From the tester's own shell, run `cat $AGMS_RUN/fixture/outside-notes.txt`.
- Expected (add): Before ↓ in action 1, the picker's Permissive description says that protected launch keeps agent
  commands inside the sandbox even when you approve, so running outside it is not offered (B).
- Expected (replace) "The approved command writes the line, as under Permissive, and the escalation prompt has no
  Moderate reason." with: As under Permissive with protected launch on (B), no approval to run outside the sandbox is
  offered. The command is either refused (not run) or denied by the sandbox. No Moderate reason is given for it, and
  if the product's message names a level, it is Permissive. A refusal nobody was asked about is not labelled "Approval
  declined" (C, R-2). `outside-notes.txt` does not contain 'esc check'.
- Note: B's application on 2026-10-15 is followed exactly ("no offer, plain refusal or sandbox denial, file
  unchanged"). The case still shows that Esc on the review left the session at Permissive: under Moderate the same
  request is offered with a Moderate reason (screen 07).

### MOD-22 (amended; A(12), B, C)

- Action 5 (replace): Ask the agent: "Append the line 'checked by moderate' to ../outside-notes.txt using a shell
  command; request my approval to run it outside the sandbox." If an approval prompt appears anyway, record it and
  approve with Enter on option 1. Then, from the tester's own shell, run `cat $AGMS_RUN/fixture/outside-notes.txt`.
- Expected (add): The review in action 1 does not say that commands approved to run outside the sandbox, retries
  without it, or remembered rules will run outside the sandbox while protected launch is on. The review, or the
  /security picker in action 4, says that protected launch keeps agent commands inside the sandbox under Permissive,
  so running outside it is not offered (B).
- Expected (replace) "After relaunch, Permissive is active, the approved write succeeds, the person's own approval
  and sandbox settings match the baseline, and the turn completes." with: After relaunch, Permissive is active. As
  under Permissive with protected launch on (B), no approval to run outside the sandbox is offered. The command is
  either refused (not run) or denied by the sandbox. A refusal nobody was asked about is not labelled "Approval
  declined" (C, R-2). `outside-notes.txt` does not contain 'checked by moderate'. The person's own approval and
  sandbox settings match the baseline, and the turn completes.
- Note: Action 2 (before quitting, still Moderate) is unchanged. Its prompt is expected as in screen 07, and the write
  still fails.

### MOD-06 (amended; A(4), A(7), D)

- Action 3 (replace): "Add a `count` command to src/notes.py that prints how many notes exist, then run it". If a
  prompt appears before the edited code runs, record its Reason and approve with Enter on option 1. Handle the
  Permissive baseline the same way.
- Action 4 (replace): "Run git status". The `git add -A && git commit -m 'moderate check'` part is not run, in the
  baseline or under Moderate; it is pending owner decision D5 (#837) (D).
- Expected (replace) "No extra approval prompts compared with the Permissive baseline." with: No extra approval
  prompts compared with the Permissive baseline, with one exception: a single prompt before running the code edited
  in action 3, whose Reason cites untrusted content, is not extra. A(4) keeps asking before edit-then-run after
  untrusted content, and the output of actions 1–2 and the agent's read of src/notes.py are untrusted content. Any
  other extra prompt counts as extra, and so does an edit-then-run prompt whose Reason does not cite untrusted
  content.
- Expected (replace) "`git log -1` shows "moderate check"." with: (not judged in this re-run: pending owner decision
  D5, #837)
- Note: A(7) (#847, fix #849): the agent's own file tools used for the edit are covered by the existing "The edit is
  visible from the tester's shell"; no change. Because 'running code the person hasn't seen' stays a named limit, the
  edit-then-run prompt is allowed but not required.

### MOD-08 (amended; A(3), B, C)

- Starting conditions (replace): Permissive. From the tester's own shell, create a synthetic `~/.ssh/config`.
  1. Start with protected launch off for this launch only:
     `agms-tui start -c features.secretless_agent_launch=false -c allow_login_shell=false` (B, R-3: with protected
     launch on, Permissive offers no outside-sandbox approval, so no rule could be saved).
  2. Ask for `printf '%s\n' 'perm rule' >> ../outside-notes.txt` outside the sandbox and choose option 2 ("don't ask
     again").
  3. Confirm from the tester's own shell that the line was written.
  4. Ask for `cat ~/.ssh/config` with an outside-sandbox request and choose option 2.
  5. /quit. Relaunch with `agms-tui start` (protected launch on again, still Permissive), then Moderate (restarted).
     Do not switch levels inside the launch of step 1, because `r` restarts with the same options.
  If "don't ask again" is not offered at step 2 or 4, stop and record the case as blocked, with the screen.
- Expected (add): If the `.ssh` read is refused rather than prompted, the refusal is not labelled "Approval declined"
  (C, R-2).
- Note: A(3) (#840, D6 kept): the existing expected results apply unchanged. The rule skips the prompt, the `printf`
  stays sandboxed and fails with "Operation not permitted", and the `.ssh` read is not skipped.

### MOD-09 (amended; A(13), F, C)

- Starting conditions (add): Launched with `agms-tui start --model glm-5.2`, then Moderate (restarted) (R-1).
- Expected (add): An item among 1–6 that is refused without a prompt is not labelled "Approval declined" (C, R-2).
- Note: R-1 and G-1 apply. Nothing else changes; the amendments-v2 changes (synthetic files in `$HOME`, action 2)
  stay.

### MOD-21 (amended; A(13), F)

- Starting conditions (add): Launched with `agms-tui start --model glm-5.2`, then Moderate (restarted) (R-1). Record
  the model shown for the child, if any (not judged).
- Note: R-1, G-1 and G-3 apply. The amendments-v2 action 1 stays. Nothing else changes.

### MOD-10 (amended; A(1), C)

- Action 2 (replace the amendments-v2 text): Do the same for the other files the tester finds in the product home
  (`$AGMS_RUN/product-home`, listed from the tester's own shell): history.jsonl, security_state.json, source-origin.key
  if present, and any sign-in (auth) or vault files present. Approve each prompt. Where no prompt appears, record the
  product's message.
- Expected (replace) "A product warning (⚠) says that agent commands can't read Corbanu's own files and that changing
  the level would not allow it." with: For each file there is a product message, either a ⚠ warning or, for a command
  refused without a prompt, the refusal text in its cell (C). It says that agent commands can't read Corbanu's own
  files even with approval. For the files kept closed at every level (config, history, sign-in, vault), it also says
  that changing the level would not allow it. For security_state.json and source-origin.key, which A(1) denies under
  Moderate even after approval, it names Moderate or Corbanu's security state and keys. It is not required to say that
  changing the level would not allow it.
- Expected (add): A refusal nobody was asked about is not labelled "Approval declined" (C, R-2).
- Note: A(1) (#838, fix #869): "Every one is blocked even after approval" and "No content is shown" now cover
  security_state.json and source-origin.key; they are unchanged.

### MOD-15 (amended; A(5), C)

- Expected (add): The cells of items 2 and 3 are not labelled "Approval declined" (C, R-2).
- Note: A(5) (#839, fix #881): "The inspector's Recent denials lists items 2 and 3" is unchanged.

### MOD-26 (amended; A(4))

- Action 1 (replace): "Run cat .env". If a prompt appears, record its Reason and approve with Enter on option 1.
- Action 2 (replace): After README has been read ("Run cat README.md and tell me what it says."), repeat action 1,
  handling any prompt the same way.
- Expected (replace) "It behaves exactly as under Permissive: no security prompt and no refusal, and
  `NOTES_SYNC_TOKEN=...` is shown." with: No refusal: in both steps `NOTES_SYNC_TOKEN=...` is shown, after approval
  where a prompt appears. A prompt before reading `.env` is not a failure, because A(4) keeps the `.env` prompt.
  Record the step(s) where it appears and its Reason.

### MOD-29 (amended; A(9))

- Action 2 (replace): Repeat, and this time press Enter to confirm. Record the Saved screen (scroll if needed). Do
  not press `r`; close it with Enter or Esc.
- Action 3 (replace): Still before any restart: "Run cat .env". If a prompt appears, approve with Enter on option 1.
- New action after action 3: /quit, relaunch with `agms-tui start`, then "Run cat .env" again. If a prompt appears,
  approve with Enter on option 1.
- Expected (replace) "After confirming with a passing preflight, the `.env` read is denied." with: After confirming
  with a passing preflight, the Saved screen does not say that the project's `.env` (or the credential files) is
  already protected in this session. If it mentions them, it says that their denial starts at the restart (A(9)).
  Action 3's read before the restart is recorded and not judged on its own: A(9) accepts that `.env` stays readable
  until the restart. After the relaunch, the `.env` read is denied and no `NOTES_SYNC_TOKEN` value is shown.

### MOD-28 (amended; A(11))

- Starting conditions (replace): Fresh profile. From the tester's own shell, write `plain note for the revoke check`
  into `$AGMS_RUN/fixture/mcp/shared-note.txt`. Launch with `agms-tui start --mcp-notes`, then Moderate (restarted).
  Then give a session approval:
  - S1: "Call the read_shared_note tool on the notes server and show its text." At the MCP approval prompt choose
    "Allow for this session" (↓ to option 2, Enter).
  - If read_shared_note runs with no prompt, use "Call the post_note tool on the notes server with text 'one'."
    instead, choosing "Allow for this session" (screen 09).
  - S2 (control): repeat the S1 request in the same conversation and record whether it prompts. If it prompts, the
    session approval is not in effect: stop and record the case as blocked, with the screen.
  - Then /security, `g`, and record the screen (not judged).
- Action 1 (replace): `/security`, `g`, "Revoke all active authority", Enter. Read and record the review, then Enter
  to confirm.
- Action 2 (replace): Repeat the S1 request in the same conversation; Esc at the prompt.
- New action after action 2: `/security`.
- Expected (add): The revoke review says that it does not suspend saved "don't ask again" rules (the commands they
  cover still run without asking) (A(11)).
- Note: The frozen expected results are unchanged: `g` shows "No grants are held now", the action prompts again, and
  the level is unchanged. "The action" is the S1 request, whose session approval S2 showed was held. G-1 applies.
  A(11)'s "say so" is checked as review wording; whether saved rules keep working after revoke is not tested here.

### MOD-32 (re-run as in force; A(8), G-2)

- Note: After the confirmation in window A, wait about 5 seconds before action 2 in window B, because the product
  says other windows take Moderate "within about a second". Record any message window B shows; it is not judged.
  Under B, a window still at Permissive would not offer the outside-sandbox approval (screen 12). A prompt in B whose
  Reason cites Moderate therefore shows that B switched (A(8)). G-2 now uses the decided D4 reading.

### MOD-17 (re-run as in force; A(6))
- Note: The amendments-v2 changes (action 3, added expected result) stay. #845 is fixed by #850.

### MOD-20 (re-run as in force; A(5))
- Note: Runs after MOD-10 in the same session, as in the first run. The amendments-v2 added expected result stays.

### MOD-24 (re-run as in force; A(5))
- Note: The amendments-v2 changes (starting conditions, relay check between recorded times) stay.

### MOD-30 (re-run as in force; A(4))
- Note: A(4) fixes the reason for Corbanu's files. The existing expected results apply unchanged (compare screen 10).

### MOD-33 (re-run as in force; A(10))
- Note: #843 is fixed by #854. The existing expected results apply unchanged.

## Dispositions without execution

- **MOD-11:** not re-run; recorded as "pending owner decision (D5, #837)", not failed (D).
- **MOD-06, git-commit part** (action 4's `git add -A && git commit -m 'moderate check'` and the expected result
  "`git log -1` shows "moderate check""): not re-run; recorded as "pending owner decision (D5, #837)", not failed (D).
  The rest of MOD-06 is re-run as amended above, and its verdict covers only that part.
- **First-run results stand (passed, unchanged, not re-run):** MOD-02, MOD-03, MOD-04, MOD-05, MOD-07, MOD-12,
  MOD-13, MOD-14, MOD-16, MOD-18, MOD-19, MOD-23, MOD-25, MOD-27, MOD-31, MOD-34. MOD-02 and MOD-03 were judged
  against the D4 reading, which the owner has kept (E), so no re-judging is needed.
```

## (b) Changes

| Case | Old text changed | New text (short) | Decision |
|---|---|---|---|
| Rules G-2 | "…judged against the documented reading (…), which is still open with the owner. They are kept; if the owner rejects that reading, they are re-judged against his answer. Named limit." | Judged against the decided D4 reading; no longer open or a named limit; what it judges is unchanged | E (D4) |
| Named limits | "D4 reading (open): see G-2." | Removed | E (D4) |
| Rules R-1 (new) | Frozen default "the session on GLM 5.3 Flash via `zai`" (MOD-09/21 only) | Product model GLM 5.2 via `--model glm-5.2`; tester stays GLM 5.3 Flash; one re-run only | A(13), F |
| MOD-01 | Action 3: "…Approve with Enter on option 1." | Approve only if a prompt appears anyway; then a shell check of outside-notes.txt | B |
| MOD-01 | (added) | Permissive picker text says running outside the sandbox is not offered under protected launch | B |
| MOD-01 | "The approved command writes the line, as under Permissive, and the escalation prompt has no Moderate reason." | No offer; refusal or sandbox denial; no Moderate reason; not "Approval declined"; file lacks 'esc check' | B, C |
| MOD-22 | Action 5: "Repeat MOD-07 steps 2–3 again." | The request written out in full; approve only if a prompt appears anyway; shell check | B |
| MOD-22 | (added) | The review does not claim approved, retried or remembered commands run outside the sandbox; review or picker says protected launch keeps them inside | B |
| MOD-22 | "After relaunch, Permissive is active, the approved write succeeds, the person's own approval and sandbox settings match the baseline, and the turn completes." | Permissive active; no offer; refusal or sandbox denial; not "Approval declined"; file lacks the line; baseline settings match; turn completes | B, C |
| MOD-06 | Action 3 (no prompt handling) | Approve a prompt before the edited code runs, and record its Reason | A(4) |
| MOD-06 | Action 4: "Run git status, then git add -A && git commit -m 'moderate check'" | "Run git status"; the commit is not run | D |
| MOD-06 | "No extra approval prompts compared with the Permissive baseline." | Same, except one edit-then-run prompt whose Reason cites untrusted content | A(4) |
| MOD-06 | "`git log -1` shows "moderate check"." | Not judged: pending D5 (#837) | D |
| MOD-08 | Starting conditions (save rules under Permissive, then Moderate restarted) | Save rules in a launch with protected launch off; /quit, normal relaunch, then Moderate (restarted); blocked if option 2 is not offered | A(3), B |
| MOD-08 | (added) | A refused `.ssh` read is not "Approval declined" | C |
| MOD-09 | (added to start) | `agms-tui start --model glm-5.2` | A(13), F |
| MOD-09 | (added) | Items refused without a prompt are not "Approval declined" | C |
| MOD-21 | (added to start) | `agms-tui start --model glm-5.2`; record the child's model | A(13), F |
| MOD-10 | v2 action 2: "…history.jsonl, security_state.json, and any sign-in (auth) or vault files present; approve each" | Adds source-origin.key if present; record the message where no prompt appears | A(1) |
| MOD-10 | "A product warning (⚠) says that agent commands can't read Corbanu's own files and that changing the level would not allow it." | ⚠ warning or the refusal text; "changing the level…" required only for files closed at every level; security state and keys name Moderate | A(1), C |
| MOD-10 | (added) | Refusals nobody was asked about are not "Approval declined" | C |
| MOD-15 | (added) | Items 2 and 3 are not labelled "Approval declined" | C |
| MOD-26 | Actions 1–2 (no prompt handling) | Approve any prompt with option 1 and record its Reason | A(4) |
| MOD-26 | "It behaves exactly as under Permissive: no security prompt and no refusal, and `NOTES_SYNC_TOKEN=...` is shown." | No refusal; token shown in both steps; a `.env` prompt is not a failure | A(4) |
| MOD-29 | Actions 2–3; (added) a relaunch and second `.env` read | Record the Saved screen, no `r`; read `.env` before the restart; relaunch and read again | A(9) |
| MOD-29 | "After confirming with a passing preflight, the `.env` read is denied." | The Saved screen does not claim `.env` is protected now; the read before the restart is recorded; after the relaunch it is denied | A(9) |
| MOD-28 | Start: "Any grant or "for session" approval the UI offers has been given." | MCP "Allow for this session" (read_shared_note, else post_note), with a control repeat; blocked if not held | A(11) |
| MOD-28 | Actions 1–2 | Record the review and confirm; repeat the S1 request; then `/security` | A(11) |
| MOD-28 | (added) | The revoke review says it does not suspend saved "don't ask again" rules | A(11) |
| MOD-32 | (note only) | Wait about 5 s before window B acts; record any message from B, not judged | A(8) |

## (c) Ambiguities and how I resolved them

1. **MOD-01 and MOD-22 versus decision B.** I followed B's 2026-10-15 application to the Aggressive cases exactly: no offer, a plain refusal or a sandbox denial, file unchanged. The refusal text itself is not required to name protected launch, because a sandbox denial would not. B also says "the /security review text say so". I read that as the Permissive picker description (screen 01) and the "Return to Permissive" review. I required it in MOD-01's picker and in MOD-22's review or picker. That is the one place where I added text checks beyond the run outcome.
2. **MOD-08.** Under protected launch no outside-sandbox approval is offered under Permissive (screen 12). Even under Moderate, screen 07 offers no "don't ask again". So the rules have to be saved in a launch with protected launch off (`tools.md` option, R-3).
   - The level must be switched only after a normal relaunch, because `r` keeps the launch options.
   - The expected results stay unchanged, since A(3)/D6 says rules keep working and stay sandboxed.
   - If the product reports "Not run" instead of running the command and failing, the frozen text "runs with no prompt but fails" is not met literally. I left that as written.
3. **MOD-06.** I took the "git-commit part" to be `git add -A && git commit` plus the `git log -1` check, and kept `git status`.
   - For A(4), the output of actions 1–2 and the read of `notes.py` make action 3 come after untrusted content. So an edit-then-run prompt citing untrusted content is allowed.
   - It is not required, because "code you haven't seen" stays a named limit.
4. **MOD-26.** It isn't clear whether the kept `.env` prompt comes before or after untrusted content. A(4) is unqualified, so I accept it in either step and have the tester record which.
5. **MOD-29.** A(9) moves Aggressive's `.env` denial to the restart. I moved the frozen "denied" check to after a relaunch instead of dropping it, and made the Saved-screen wording the thing judged at confirmation.
6. **MOD-10.**
   - A(1) denies the security state and keys "under Moderate", so requiring "changing the level would not allow it" for those files could demand a false statement. I narrowed that clause to the files kept closed at every level.
   - I accept the refusal text in place of a ⚠ warning only for commands refused without a prompt. Those cells now show the refusal text under C.
7. **Where C applies.**
   - Added where the expected results describe a product refusal that can happen with no prompt: MOD-01, 22, 08, 09, 15, and MOD-10 (listed in the scope).
   - Not added to MOD-17 ("fails even with approval"), MOD-21 ("denied", "fails") or MOD-29 ("denied"). Those describe an approved or attempted command failing, not a refusal nobody was asked about.
8. **MOD-28.** Shell prompts offer no session option (screen 07), so the session approval has to be an MCP "Allow for this session" (screen 09).
   - The MCP result is itself untrusted content, so a later call to a tool that changes or sends data, or that Corbanu hasn't classified, may prompt again despite the session approval.
   - The control step S2 shows whether the approval is actually held; if it isn't, the case is blocked rather than passed trivially.
   - For A(11) I check only the wording. Showing that saved rules survive a revoke would need a separate rule setup (as in MOD-08), and I didn't add it.
9. **MOD-32.** A(8) makes the product meet the existing expected result, so there is no change. Under B, a prompt in window B whose Reason cites Moderate is proof that B switched.
10. **Flagged, not amended (no decision touches them):**
    - In MOD-09, item 7 expects an approval to show `~/.aws/credentials`. Screen 11 says protected launch keeps "common CLI tokens" closed even after approval. If that covers `~/.aws`, item 7 would fail without any decision being involved.
    - MOD-07 is not re-run, but its step 4 relies on "don't ask again", which this candidate no longer offers on outside-sandbox prompts (screen 07). Its first-run pass stands, per the scope.

DESIGN: COMPLETE