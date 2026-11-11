# PF-84 live ChatGPT check (2026-10-21, D79)

This record covers flag-removal prerequisite 1 of the PF-84 plan: a live check with a disposable ChatGPT account.
Travis created a **free** ChatGPT test account (not his own) and approved the device-code sign-in in his browser. No
token, password, account email or device code appears here; the product's masked email hint is shown as
`<email-hint>`.

**Result:** all 10 checks run passed. **Prerequisite 1 is not complete:** "refresh written back to that account" was not
checked live, and only the S06 mock tests cover it (see "Not verified"). Two low defects were filed: #1205 and #1208.
This was a macOS-only run.

## Setup

- **Build:**
  - The installed signed build: `corbanu 0.1.48`, main `2f9d05c51e` (INSTALL.json).
  - Its SHA-256 `a4f9804b…c494` matches INSTALL.json, and it is signed with Developer ID K5N2T25C23.
  - It is a `dev`-profile build with debug assertions.
  - It includes #1090 (#1085), #1096, #1140 (#1129) and #1163 (#1144). It does **not** include #1192 (D86).
- **Home:** a throwaway home in a throwaway run directory, `.codex-work/pf84-live1/`.
  - `CODEX_HOME`, `CORBANU_HOME` and `PFTERMINAL_HOME` pointed at it, and `HOME` at a throwaway directory.
  - Its config had `cli_auth_credentials_store = "file"` and `[features] named_accounts = true`.
  - Details: [scripts/env.sh](scripts/env.sh).
- **Default login:** a fake OpenAI API key (never valid) in `auth.json`. It is deliberately not the test account. Any
  request that fell back to it would fail with `401 invalid_api_key`, so a fallback would show.
- **Named account:** added with `corbanu login --account t1 --device-auth`. The URL and one-time code went to the
  coordinator, and Travis approved them. The code was posted at 08:38:47Z and sign-in finished at 08:40:34Z
  ([01](captures/01-device-code-signin.txt), [05](captures/05-run-transcript-extracts.txt)).
- **Keeping the macOS keychain out:**
  - **What kept it out:** `CORBANU_TEST_NO_NATIVE_KEYRING=1`, which makes the build refuse every native-keyring call.
    This works only because the build has debug assertions; a release build ignores it. The second guard was
    `cli_auth_credentials_store = "file"`.
  - **Result:** the vault saved its key in the home's file fallback (`secrets/keyring-fallback/*.key`). That happens
    only when the keyring save is refused. Nothing queried the keychain to confirm that no item exists; doing so is
    off-limits.
  - **Sandbox:** every product run went through `sandbox-exec` with [scripts/nokeychain.sb](scripts/nokeychain.sb).
    It only denies running `/usr/bin/security` and the data-protection keychain and security-agent XPC services.
  - **Sandbox limit:** it allows `com.apple.SecurityServer` (securityd). A first, stricter profile that also denied
    it broke TLS: sign-in failed with "error sending request" ([05](captures/05-run-transcript-extracts.txt)). securityd can access the login keychain and can show
    its prompts, so this profile is **not** keychain protection on its own. Don't reuse it with a release build.
  - The binary ran directly, not through the `~/.local/bin` launcher, and `SSH_CONNECTION` was unset. So the
    launcher's `security unlock-keychain` path never ran.
  - The operator's real Corbanu home (`.codex-work/corbanu-terminal/home`) shows no sign of change: neither its
    `auth.json` nor anything at the top level of its `secrets/` is newer than the run start
    ([05](captures/05-run-transcript-extracts.txt)).
- **Models:** the free plan's catalog for `t1` listed `gpt-6-luna`, `gpt-5.6-terra` and `gpt-5.6-luna`. Turns used
  `gpt-5.6-luna`, and the Z.AI step used GLM 5.3 Flash.

## Results

Rollout `turn_context` per turn: every OpenAI turn in the four rollouts records `t1`, and the single Z.AI turn
records `zai:glm-5.3-flash:default` ([04](captures/04-turn-accounts-hashes-vault.txt)).

| # | Check | Result | Evidence |
|---|---|---|---|
| 1 | Device-code sign-in of a named account | PASS. Output: `Signed in OpenAI account t1 (ChatGPT sign-in · <email-hint> · Free)`. Afterwards, `login status` still reports the default API key, and `login --account t1 status` and `account list` show `t1`. | [01](captures/01-device-code-signin.txt), [05](captures/05-run-transcript-extracts.txt) |
| 2 | Requests made under the account use it | PASS. `exec --account t1 -m gpt-5.6-luna` answered `PF84-LIVE-OK`; the default login is a fake key, so only `t1` could serve it. The config's default model `gpt-6-sol` was refused by the server as "not supported when using Codex with a ChatGPT account", so that request also went out on the ChatGPT sign-in. The model list was cached under `models-cache/openai/t1/`. | [02](captures/02-exec-named-account.txt), [05](captures/05-run-transcript-extracts.txt) |
| 3 | No cross-use | PASS. The same prompt without `--account` (run with `--ephemeral`, so it left no rollout) went to `api.openai.com` with the fake default key and got `401 invalid_api_key`. | [03](captures/03-exec-default-control.txt) |
| 4 | Default login never modified | PASS. `auth.json` had the same SHA-256 before sign-in, after sign-in, after the exec turns, after the TUI sessions and after sign-out (#595/#658/#1085). | [04](captures/04-turn-accounts-hashes-vault.txt), [05](captures/05-run-transcript-extracts.txt) |
| 5 | `/status` shows the account | PASS. It shows `Model provider: OpenAI · account t1`, `Account: ChatGPT sign-in · <email-hint> · Free (account t1)` and `Account usage: t1 · …`. Limits appeared after the first reply in that session; the Limits row itself does not name an account. | [resume](captures/tui-status-resume.txt), [switch back](captures/tui-status-switchback.txt) |
| 6 | Resume keeps the account | PASS. A TUI `resume` of the exec thread showed `t1` in `/status`, and its turn (`PF84-RESUME-OK`) ran on `t1`. | [resume](captures/tui-status-resume.txt), [04](captures/04-turn-accounts-hashes-vault.txt), [05](captures/05-run-transcript-extracts.txt) |
| 7 | Switching keeps the account | PASS. `/model` → Z.AI ran a turn on `zai:default`. Switching back to OpenAI printed "Connectors, usage and the model list now use openai account `t1`", and the next turn ran on `t1`. The Z.AI-side message has a wording defect, #1205 (its text is in 05). | [switch back](captures/tui-status-switchback.txt), [04](captures/04-turn-accounts-hashes-vault.txt), [05](captures/05-run-transcript-extracts.txt) |
| 8 | Fork keeps the account | PASS. After `corbanu fork <id>`, `/status` shows `OpenAI · account t1` and `Forked from: …`. | [fork](captures/tui-status-fork.txt) |
| 9 | Spawned agent and reload keep the account (#1129/#1140/#1163) | PASS. In the fork, a spawned child `01a15da2-e95a…` showed `gpt-5.6-luna low · account t1` and finished. The fork was then quit and resumed, and a follow-up task reloaded the child (shown as "Linnaeus [orc]"). It completed, and the child's rollout records `t1` for both turns. On resume, the TUI said a "legacy /spawn hierarchy" would not dispatch work, yet the follow-up was dispatched; #1208 tracks the mismatch. | [spawn](captures/tui-fork-spawn.txt), [reload](captures/tui-resume-child-reload.txt), [04](captures/04-turn-accounts-hashes-vault.txt) |
| 10 | Removing the account refuses rather than falling back | PASS. `logout --account t1` printed "Signed out OpenAI account `t1`" (the CLI help says it revokes the tokens), and `account list` became empty. After that, `exec resume <id>`, `exec --account t1`, TUI `resume <id>` and TUI `fork <id>` were all refused before any request. Each refusal names `t1` and the recovery (`corbanu login --account t1` or `--account`), and none of them reached the default login (no 401). | [10](captures/10-refusals-after-removal.txt), [resume](captures/tui-resume-after-remove.txt), [fork](captures/tui-fork-after-remove.txt) |

## Not verified

- **Refresh written back to the account** (part of prerequisite 1). The product refreshes ahead of time only within
  5 minutes of the access token's expiry. Only when that expiry can't be read does it fall back to 8 days since the
  last refresh. The new token was far from expiring, and forcing a refresh would have meant editing the encrypted
  vault or the clock. To close it, either:
  - run a later check that keeps a test sign-in until its token expires, or
  - decide to accept the S06 mock refresh tests.
- **Removal while a session is running.** `t1` was already removed. Signing in again for this check was tried twice
  (09:03Z, 09:18Z), but both codes expired unused. Only removal between sessions (row 10) is covered.
- **D86** (`/status` in homes without a named account): #1192 is not in this build. It doesn't apply to this home,
  which has a named account.
- **App server and daemon (#1085)** were not driven directly. On resume, `/status` showed "Limits: data not available
  yet" until the first reply, which matches the documented hold.

## Observations (not filed)

- The `/model` picker lists OpenAI models that are not in `t1`'s catalog (`gpt-6-sol`, `gpt-6-astra`, `gpt-6.1-sol`,
  `gpt-5.6-sol`). Some carry the note "ChatGPT sign-in may reject it".
- The `exec` header names the provider but not the account.

## Cleanup

- `t1` was signed out with `logout --account t1`.
- The throwaway home and its credential files (`auth.json`, and `secrets/` with the vault and its key file) were
  deleted, and `ls` confirmed they were gone.
- The whole run directory, including the raw logs, was then deleted.
- The Z.AI key was only an environment variable of the run's tmux server, which was stopped.
