# Proposed tests: named provider accounts for forks, resume and recovery

I based these only on `intent.md` and `screens/*.txt`. The screens back up some cases, but none of them prove the feature works. Several requirements have no screen for the candidate build at all.

## Shared setup

These setups are used throughout. Every home has `named_accounts` turned on unless a case says otherwise.

- **H-ZAI:** Z.AI is the configured provider. It has three accounts, `default`, `work` and `personal`, each with its own key. `[provider_accounts] zai = "personal"` is set, so a session that ends up on `personal` or `default` is clearly the wrong one.
- **H-OAI:** OpenAI is the configured provider and has usable credentials. Z.AI account `work` is enrolled with a valid key.
- **H-OAI-bare:** Same as H-OAI, but with no Z.AI account enrolled.
- **H-NOKEY:** The configured provider has no usable default credentials. It does have stored conversations, including one recorded on Z.AI `work`.
- **Usage check:** Unless a case says otherwise, "charged to X" means the "Account usage" line in `/status` grows only for X. All other accounts stay at 0 or unchanged. The provider-side usage for each key should agree.

---

## A. Forks follow the source conversation's account

### FK-01: `/fork` stays on the source account
- **Priority:** blocker
- **Starting conditions:** H-ZAI. Start with `corbanu --account work`, so the config points to `personal` and the session runs on `work`. Send 1 prompt.
- **Actions:** `/fork`, then send a prompt in the fork, then `/status`.
- **Expected:**
  - The screen shows "Thread forked from <source id>".
  - In `/status`, "Model provider" ends in `· account work` and "Forked from" shows the source id.
  - "Account usage" grows for `work` only. `personal` and `default` show no new tokens.
  - Screen 01 is consistent with this. On the earlier build (screen 11), the tokens went to `personal` while `/status` said `work`.

### FK-02: Fork of a resumed conversation (account comes from the recording, not a flag)
- **Priority:** blocker
- **Starting conditions:** H-ZAI. Resume a conversation recorded on `work` without using `--account`.
- **Actions:** `/fork`, send a prompt, then `/status`.
- **Expected:**
  - `/status` says `account work`.
  - Only `work` is charged, not `personal` from the config.

### FK-03: Editing an earlier prompt (which forks)
- **Priority:** blocker
- **Starting conditions:** As in FK-01, with at least 2 turns.
- **Actions:**
  1. Go back and edit the first prompt.
  2. Submit the edit.
  3. Run `/status`.
- **Expected:**
  - A new forked thread is created.
  - `/status` says `account work`.
  - Only `work` is charged.
  - No screen covers this case.

### FK-04: Side conversation (`/side`)
- **Priority:** blocker
- **Starting conditions:** As in FK-01.
- **Actions:**
  1. `/side`, send a prompt, then `/status`.
  2. Switch back to the main thread (Ctrl+/), send a prompt, then `/status`.
- **Expected:**
  - Both the side and main conversations show `account work`.
  - Only `work` is charged in both.
  - Screen 02 shows `work` for the side conversation.
  - Editing previous prompts is blocked in the side conversation with the notice shown in screen 02.

### FK-05: `corbanu fork <id>` from the command line
- **Priority:** blocker
- **Starting conditions:** H-ZAI, with a conversation recorded on `work`.
- **Actions:** Run `corbanu fork <id>`, send a prompt, then `/status`.
- **Expected:**
  - The fork opens on `work`.
  - `/status` shows `account work` and the "Forked from" id.
  - Only `work` is charged.
  - No screen covers this case.

### FK-06: An explicit `--account` beats the source account
- **Priority:** blocker
- **Starting conditions:** As in FK-05.
- **Actions:** Run `corbanu fork <id> --account personal`, then run it again with `--account zai:personal`.
- **Expected:**
  - Both forks run on `personal`, and `/status` says `personal`.
  - `work` is not charged.

### FK-07: Fork when the source account has since been removed
- **Priority:** blocker
- **Starting conditions:** A conversation recorded on `work`, after which `work` is removed from the vault.
- **Actions:** Run `corbanu fork <id>`. Separately, run `/fork` inside a session resumed on that conversation, if that can be reached.
- **Expected:**
  - An error names `zai` / `work` and how to recover.
  - Nothing falls back silently to `default` or `personal`, and no tokens are charged.

### FK-08: Fork of a fork
- **Priority:** advisory
- **Starting conditions:** As in FK-01.
- **Actions:** `/fork`, then `/fork` again, send a prompt, then `/status`.
- **Expected:**
  - The account is still `work`.
  - "Forked from" shows the immediate parent.

### FK-09: Worker or child agent inherits the parent's account
- **Priority:** advisory
- **Starting conditions:** A session on `work` that spawns a worker agent.
- **Actions:** Let the worker run a turn, then check usage.
- **Expected:** The worker's tokens are charged to `work`, as the precedence constraint requires.

---

## B. Resume on a provider other than the home's configured one

### RS-01: Credentials present
- **Priority:** blocker
- **Starting conditions:** H-OAI.
- **Actions:** Run `corbanu resume <zai-work-id>`, send a prompt, then `/status`.
- **Expected:**
  - The transcript loads with no OpenAI sign-in or onboarding screen.
  - The model is Z.AI.
  - "Model provider" is Z.AI `· account work`, and only `work` is charged.
  - Screens 03 and 04 are consistent with this. On the earlier build, screen 14 showed the Z.AI key prompt instead.

### RS-02: Credentials missing
- **Priority:** blocker
- **Starting conditions:** H-OAI-bare.
- **Actions:** Run `corbanu resume <zai-work-id>`.
- **Expected:**
  - A message names provider `zai` and account `work`, and gives the recovery: `corbanu account add zai work` and/or `--account`.
  - No onboarding, sign-in or key-entry screen appears.
  - The command exits with a non-zero code, and no model call is made.
  - No candidate screen covers this case.

### RS-03: Only `default` is enrolled for the recorded provider
- **Priority:** blocker
- **Starting conditions:** H-OAI with only Z.AI `default` enrolled.
- **Actions:** Resume the conversation recorded on `work`.
- **Expected:** An error names `work`. The session does not fall back silently to `default`.

### RS-04: Resume through the picker matches resume by id
- **Priority:** blocker
- **Starting conditions:** H-OAI, and H-OAI-bare.
- **Actions:** Run `corbanu resume`, select the Z.AI conversation, and press Enter.
- **Expected:** The outcome is the same as RS-01 (H-OAI) or RS-02 (H-OAI-bare).

### RS-05: Resuming twice keeps the recorded account
- **Priority:** advisory
- **Starting conditions:** RS-01 has been completed.
- **Actions:** Exit, then run `corbanu resume <same id>` again.
- **Expected:** The session is still on `work`. Resuming did not rewrite the recorded account to the home's defaults.

---

## C. `[provider_accounts]` names an account that does not exist

### GH-01: Recorded account exists
- **Priority:** blocker
- **Starting conditions:** H-ZAI with `zai = "ghost"` in the config. The conversation is recorded on `work`.
- **Actions:** Run `corbanu resume <id>`, send a prompt, then `/status`.
- **Expected:**
  - The session resumes on `work`, and `/status` says `account work`.
  - Only `work` is charged.
  - Screen 05 is consistent with this. On the earlier build, screen 13 exited with code 1.

### GH-02: Recorded account is missing too
- **Priority:** blocker
- **Starting conditions:** Same config as GH-01, but `work` has been removed.
- **Actions:** Run `corbanu resume <id>`.
- **Expected:**
  - The error names the recorded account, `work`, and gives `corbanu account add zai work` as the recovery.
  - It ideally also mentions `ghost`.
  - Nothing falls back to `default`, and the exit code is non-zero.
  - No screen covers this case.

### GH-03: New session with `ghost` in the config
- **Priority:** blocker
- **Starting conditions:** The same `ghost` config.
- **Actions:** Run `corbanu` to start a new session, not a resume.
- **Expected:** An error names `ghost` with the recovery. The session does not start silently on `default`.

---

## D. Recovery wording depends on where the error appears

### MSG-01: Inside the TUI
- **Priority:** blocker
- **Starting conditions:** A running session on any account, with a stored conversation recorded on `work`, which has since been removed.
- **Actions:** `/resume`, then select that conversation.
- **Expected:**
  - The message names `zai` / `work` and points to `/providers`. It does not mention `corbanu account add` or `--account`.
  - The current session stays open and usable.
  - Screen 10 is consistent with this. On the earlier build, screen 15 pointed to the command line.

### MSG-02: Recovery through `/providers` without restarting
- **Priority:** advisory
- **Starting conditions:** Right after MSG-01.
- **Actions:**
  1. Open `/providers` and add account `work`.
  2. `/resume` the same conversation again.
  3. Send a prompt, then `/status`.
- **Expected:** The conversation resumes on `work`, and only `work` is charged.

### MSG-03: On the command line
- **Priority:** blocker
- **Starting conditions:** The same situation as RS-02 or GH-02.
- **Actions:** Run `corbanu resume <id>`.
- **Expected:** The message points to `corbanu account add …` and `--account`. It does not mention `/providers`.

### MSG-04: Session after a failed in-app resume
- **Priority:** advisory
- **Starting conditions:** Right after MSG-01.
- **Actions:** Send a prompt, then `/status`.
- **Expected:** The original session id and account are unchanged. The tokens are charged to that original account.

---

## E. Esc behaviour

### ESC-01: Esc in the `corbanu resume` picker when there are no credentials
- **Priority:** blocker
- **Starting conditions:** H-NOKEY.
- **Actions:** Run `corbanu resume`, then press Esc.
- **Expected:**
  - The process exits cleanly with code 0 and the terminal is restored.
  - No sign-in or onboarding screen appears.
  - Screens 06 and 07 are consistent with this. On the earlier build, screen 12 showed the onboarding screen.

### ESC-02: Ctrl+C in the same picker
- **Priority:** advisory
- **Starting conditions:** H-NOKEY.
- **Actions:** Run `corbanu resume`, then press Ctrl+C.
- **Expected:** Same as ESC-01.

### ESC-03: Esc in the in-session `/resume` picker
- **Priority:** blocker
- **Starting conditions:** A running session with a draft in the composer and at least one turn.
- **Actions:** `/resume`, press Esc, then `/status`.
- **Expected:**
  - You are back in the same session.
  - The session id, account and transcript are unchanged, and the draft is still there.
  - Screen 08 is consistent with this.

### ESC-04: Normal onboarding is not broken
- **Priority:** blocker
- **Starting conditions:** A fresh home with no credentials.
- **Actions:** Run `corbanu`, which is not a resume.
- **Expected:** Onboarding still appears as it did before the change.

---

## F. Precedence, regression and sanity checks

### PR-01: Full precedence order
- **Priority:** blocker
- **Starting conditions:** H-ZAI, plus a conversation recorded on `work`.
- **Actions:** Run each of the following, then `/status`:
  1. Resume with `--account personal`.
  2. Resume with no flag.
  3. Start a new session with no flag.
  4. Start a new session with no flag after removing `[provider_accounts]`.
- **Expected:** The accounts used are `personal`, `work`, `personal` and `default`, in that order.

### PR-02: `--account` for another provider does not affect Z.AI
- **Priority:** advisory
- **Starting conditions:** A conversation recorded on Z.AI `work`.
- **Actions:** Resume it with `--account openai:x`.
- **Expected:** The Z.AI session still uses `work`. Whatever happens with `openai:x` is clearly reported.

### REG-01: `named_accounts` turned off
- **Priority:** blocker
- **Starting conditions:** The same homes, with the feature off.
- **Actions:** Repeat FK-01, RS-01, ESC-01 and ESC-03.
- **Expected:**
  - Behaviour and text match the build from before this change.
  - `/status` shows no account suffix or per-account usage, as was the case before.

### REG-02: No secrets shown
- **Priority:** blocker
- **Starting conditions:** All the error and status paths above.
- **Actions:** Check the screens, exit output, transcripts and logs.
- **Expected:** No API key or any part of one appears anywhere.

### REG-03: Footer and `/status` agree
- **Priority:** advisory
- **Starting conditions:** A session on `work`.
- **Actions:** Compare the bottom status line with `/status`.
- **Expected:**
  - Both name the same account.
  - Every candidate screen shows "via zai default" in the footer, even when `/status` says `account work`. Find out whether that "default" means the account; if it does, it is wrong.

### REG-04: Usage totals add up
- **Priority:** advisory
- **Starting conditions:** Any session after a few turns.
- **Actions:** Compare the per-turn "Token usage" line, "Token usage" in `/status`, and "Account usage".
- **Expected:**
  - The numbers agree, given how the inherited fork history is counted.
  - Other accounts are listed with 0, or the reason they are left out is clear.
  - In screen 01, "Token usage" is 27.3K but `work` shows 13,023 input. In screen 09, there are 20,160 cached tokens against 8.4K total.

### REG-05: "Account:" line in `/status`
- **Priority:** advisory
- **Starting conditions:** RS-01 and GH-01.
- **Actions:** Run `/status`.
- **Expected:**
  - The "Account:" credential line shows the active credential, or there is an explanation for why it is missing.
  - It is missing in screens 04 and 05, but present in screens 01, 02 and 09.

### REG-06: Both platforms
- **Priority:** advisory
- **Actions:** Run the blocker cases on macOS and Linux.
- **Expected:** The results are the same on both.

---

## Missing information

- **Several requirements have no candidate screen:**
  - Editing an earlier prompt (FK-03) and `corbanu fork <id>` (FK-05).
  - Resume when the recorded provider's credentials are missing (RS-02).
  - The recorded account also missing (GH-02).
  - Command-line wording for the missing-account error (MSG-03).
  - Precedence with an explicit `--account`, child or worker agents, and `named_accounts` turned off.
- **The screens leave out key starting conditions:** the config and launch flags for screens 01–02, 04 and 09, whether `work` was ever enrolled in the screen 10 home, and the exit code after a successful resume. Screen 07 only shows `EXITED 0`, so it does not prove that no onboarding screen flashed past.
- **It's unclear how to check which account was really charged.** `/status` is the only source in the packet, and the earlier build shows that `/status` can disagree with the real charge. A provider-side or vault-side usage view for each key is needed.
- **Account usage display:**
  - In the candidate, "Account usage" shows only one account. It's unclear whether accounts with no usage are hidden, which would also hide a wrong charge.
  - It's unclear whether the numbers include tokens inherited by a fork.
- **Footer meaning:** "via zai default" is unexplained.
- **Expected results that aren't specified:**
  - Exit codes and text for each error.
  - Whether GH-02 should mention the `ghost` config.
  - Whether a failed in-app resume should change anything in the current session.
  - What `/fork` should do if the source account disappears mid-session.
- **Edge cases not covered:**
  - Account names with odd characters.
  - Two providers that each use named accounts.
  - Conversations recorded before this feature existed, which have no recorded account.