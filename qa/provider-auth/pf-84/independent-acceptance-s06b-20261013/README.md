# PF-84-S06 slice B re-acceptance and the #555 cases (2026-10-13)

Scope:

- Re-verification of every slice B criterion of
  [PF-84-S06](../../../../docs/sprints/current/unified-provider-auth/pf-84-s06-aws-and-chatgpt-accounts.md) (named
  ChatGPT/OpenAI sign-in accounts, behind `named_accounts`) and of slice A criterion A10. This follows the fixes for
  #570, #571, #573 (PR #604) and #572 (PR #610).
- All 33 cases of the #555 code-blind design
  ([design.json](../fix-555-20261012/code-blind/design.json)). No separate executor had run them before.

The first S06 acceptance ([../independent-acceptance-s06-20261011/](../independent-acceptance-s06-20261011/README.md),
PR #574) accepted slice A and held slice B. Independent review: [REVIEW.md](REVIEW.md) (round 5, the final round: AGREE WITH CHANGES, no blocking findings). Its changes are applied; see the end of this file. Earlier rounds are in [review-rounds/](review-rounds/).

## Method

- **Isolation: instruction-only.** I followed the rules by instruction; no sandbox enforced them, and I recorded no
  negative access probes.
  - Scripts ran from inside the repository worktree. They reused the 10-11 acceptance's mock and leak scan.
  - They fetched real credentials from the operator's vault through the installed helper, which the vault policy allows.
  - They reached the network: the real provider APIs and `ssh glitch`.

  This report therefore does not claim that a code-blind execution gate passed. It is a code-blind functional
  acceptance by a separate executor.
- **Code-blind.** I read only these:
  - the S06 sprint record, the plan's PF-84 section, and `s06a-gate.md` / `s06b-gate.md`;
  - the previous S06 acceptance's README and REVIEW, for method and its failed criteria. I copied its scripts as a
    starting point; see the header of each file;
  - the #555 README, `code-blind/README.md`, `design.json` and `original-proposal.md` (the case list and home
    definitions), and `packet/intent.md`;
  - issues #555, #570–#573, #588, #595 and #609;
  - `docs/provider-accounts.md`, `docs/tmuxHarness.md`, `--help` output, `sec-common.md` and `wave2-common.md`.

  I read no `codex-rs/**`, `scripts/**`, PR diffs, worker logs, or code and worker reviews. I did read the 10-11
  acceptance REVIEW and this run's own review rounds. Every observation is external: CLI output,
  TUI screens, files in disposable homes, and what my mock received.
- **Builds: limited.** These were debug builds, copied without their companion binaries, not the packaged release
  candidate. Code Mode was unavailable in the TUI ("failed to spawn code-mode host", e.g. `tB-04`). Debug builds with `codex-core/developer-accounting,codex-tui/developer-accounting`, toolchain 1.95.0, one
  target dir each:
  - **CAND:** `origin/main` `53c02c9301`.
  - **BASE:** `82b1b2cf32`, main just before #561. That is before #561/#566 (#555), #604 and #610, so BASE shows the
    pre-fix behaviour. Some capture headers say only "main before #561"; it is the same commit.
    - BASE **already contains S06 slices A and B** (#543, #557). Flag-off comparisons against BASE therefore show
      "unchanged since the fixes", not "unchanged since before S06".
    - Pre-S06 flag-off parity (`3a0a38bed2`) was shown by the 10-11 acceptance (its C1/C2). The C1/C2/REG-01 checks
      found no flag-off difference against BASE.
  - **Linux:** a fresh clone of `53c02c9301` built on glitch, for REG-06. The binary reports only `corbanu 0.1.48`, and
    its SHA-256 was not recorded before the build directory was deleted.
- **Safety.**
  - Every run used `env -i`, a disposable `CORBANU_HOME` and `HOME`, and `CORBANU_TEST_NO_NATIVE_KEYRING=1`
    ([env.sh](scripts/env.sh), [cleanexec.py](scripts/cleanexec.py)).
  - Real keys came only from `corbanu.bak-20261010 vault auth-helper`, into stdin (`account add`, `login --with-api-key`),
    into a process environment, or once into a private tmux paste buffer (MSG-02's masked entry). They never went to argv
    and were never printed.
  - Each TUI ran on its own tmux server, killed afterwards.
  - Travis's ChatGPT login and the real `~/.local/bin/corbanu` were never touched.
- **Mock.** [mock.py](scripts/mock.py) is the previous acceptance's stdlib mock (OpenAI issuer with rotating refresh
  tokens, Responses, Bedrock SigV4), with one change: a `/responses` request with **no credentials now gets 401**, not
  200. A #571-style regression would therefore fail visibly. Mock logs: [captures/mock/](captures/mock/). The
  `ps/mcp` noise is removed.
- **Real providers.**
  - Z.AI `glm-5.3-flash` (`provider/zai_api_key`): every #555 case, and the TUI coordinator.
    - In homes `z1`–`z3` and on Linux, every Z.AI account held the same real key, so the "charged to X" checks there are
      Corbanu's own attribution only.
    - After review round 1 the charge checks were re-run in home **`z4`**, where only `work` holds the real key and
      `personal` and `default` hold fake keys. A request routed to the wrong account then fails with 401 at Z.AI
      ([fk_suite2.sh](scripts/fk_suite2.sh), captures `555r2-*`).
  - Kimi `kimi-k3` (`provider/kimi_api_key`).
  - OpenAI `gpt-5.4` (`openai-api-key`).
  - Claude Plan (`claude-plan-test-token`).
- **TUI.** The TUI ran through private tmux servers per `docs/tmuxHarness.md` ([tmuxlib.sh](scripts/tmuxlib.sh),
  [drv.sh](scripts/drv.sh), [tdrv.sh](scripts/tdrv.sh), [launch lines](captures/tui-invocations.txt)).
- **Raw attempts and re-runs.** This departs from the rule to keep every raw attempt: empty or invalid first-attempt
  captures were overwritten, not kept. Every re-run and every discarded or overwritten attempt is listed under
  "Re-runs" below. Kept discarded captures carry `-x-discarded` in their names.
- **Harness slips, fixed and re-run (no effect on results):**
  - Sessions sharing one tmux server inherited the first session's `CORBANU_HOME`. Every TUI now has its own server.
    The affected rs-suite run was discarded and the homes rebuilt.
  - A line-count bug left some captures empty. Those cases were re-captured
    ([rs_fixups.sh](scripts/rs_fixups.sh)).
  - A copied Corbanu home can't open its vault, because the fallback key is bound to the home path. GH homes are
    therefore built fresh, not copied.

## Verdicts: slice B and A10

| # | Criterion | Verdict | Evidence |
| --- | --- | --- | --- |
| B1 | Each sign-in account has its own vault store: no keychain item, never `auth.json`. | PASS (keychain: NOT VERIFIABLE) | After 7 named sign-ins, the home has only `auth.json` (the default), `secrets/local.age` and the fallback key. 0 home files hold a named refresh token or key in clear ([b-cli](captures/b-cli.txt), [leak-scan](captures/leak-scan.txt)). After every TUI add, t1 has no `auth.json` ([tB-10-i](captures/tB-10-i-t1-files-after-adds.txt); accounts in [tB-10-h](captures/tB-10-h-account-list-after-tui-adds.txt)). The b1 listing also shows `secrets/.vault.lock`, which is harmless. Under the no-keychain rule, only "no item created" can be observed. |
| B2 | A refreshed token is saved back to that account only, and other processes pick it up. | PASS | `work` and `play` ran in parallel, each on its own chain (29/30 refreshes with 8 s tokens). A new process on `work` continued gen 30→31 from the vault. `twin` (the same person) kept its own chain. Across the run there was **0** `refresh_token_reused`; the one rejection is `bad`'s expected one. The default `auth.json` hash stayed the same throughout ([b-cli](captures/b-cli.txt), [mock-b1](captures/mock/mock-b1.jsonl)). |
| B3 | `login --account`, `logout --account`, status, rename and remove work. | PASS (browser flow not exercised) | Device code and `--with-api-key` both work. `login --account ghost status` exits 1. Re-login as another user revoked the old `play` token; as the same user it revoked nothing. `logout --account play` revoked only `play`, and a second logout is harmless. `rename` prints the `[provider_accounts]` hint. `remove` revoked and deleted only `twin2`. The TUI never overwrites an account: a second add named `devz` was refused before any sign-in, with no sign-in or revoke at the mock ([tB-10-d](captures/tB-10-d-add-existing-name-refused.txt)). So the "refused add revokes its tokens" path was **not** reached (see Not exercised). |
| B4 | The default login is never read or changed by a named account. (#570) | **PASS for "never changed"** (#570 fixed). **"Never read" is contradicted** for side requests ([#658](https://github.com/CorbanuCore/CorbanuTerminal/issues/658)) | In [b-forced](captures/b-forced.txt), CAND kept `auth.json` byte-identical in every named case: 1, 2, 5, 6, 7, 8, 9, 10/10b (with `CODEX_API_KEY` and `OPENAI_API_KEY` decoys) and 11 (`exec resume`). BASE **deleted** it in 1b, 2b, 5b and 7b. TUI (embedded app server): named `k2` under `forced_login_method="api"` answered with the default ChatGPT `auth.json` unchanged. The mock logged the bearer as `UNKNOWN`, but its SHA-256 prefix `af0ebda0` is that of `k2`'s fake key ([tB-16](captures/tB-16-forced-tui.txt)). `auth.json` at `chmod 000` does not stop a named session. Not #570 evidence: a **default** session still logs out a ruled-out default login.<br>- CAND `exec` case 3 shows it, with no BASE control in this run. The 10-11 run showed the same on CAND and pre-S06 (its oa-forced cases 3 and 4).<br>- In the TUI, CAND and BASE both show it (the first two tB-16 blocks).<br> That is inherited and outside the named-account rule. The `b-cli` heading "default login still used" is inaccurate: that run went out with the `CODEX_API_KEY` decoy (default precedence, as before). Model requests never carried the default login. But a session on named `work` still sends the default ChatGPT token for plugin listings and wham settings/credits ([tB-19](captures/tB-19-named-start-default-token-traffic.txt), #658). |
| B5 | Display: kind, email hint, plan and fingerprint; `/status` names the sign-in; the fingerprint is stable across refreshes. | PASS | `ChatGPT sign-in · t…@example.com · Plus · fp 3c3b6f0807d1` and `OpenAI API key · fp …` ([tB-01](captures/tB-01-providers-rows-zai-session.txt)). `/status`: `Account: ChatGPT sign-in · t…@example.com · Plus (account work)`. The `fpy` fingerprint was `4934a2521a5b` before its first refresh and after it ([tB-09](captures/tB-09-fingerprint-across-refresh.txt)). The same user under two names gives the same fp (`work`, `twinz`). The fingerprint appears in `/providers` only; CLI `account list` and `login status` show kind, email hint and plan. |
| B6 | Signing the same ChatGPT user in twice warns, not refuses. | PASS (CLI and TUI) | CLI: `twin` names `work`, and `dup` names `default`. TUI: `twinz` was saved with "Warning: this is the same ChatGPT user as account(s) `work`…" ([tB-10-e](captures/tB-10-e-same-user-warning.txt)). |
| B7 | `forced_login_method` / `forced_chatgpt_workspace_id` apply to named accounts, against that account only. | **PASS** (#570 fixed) | Refused at thread start, with recovery: ChatGPT `chat` under `api`, API-key `key` under `chatgpt` (also with env decoys, now the named check rather than "Logging out"), and `chat` (ws-main) under `ws-other`. Allowed: `chato` (ws-other) and API-key `key` under `ws-other` (BASE: "Logging out", exit 1). The TUI hides the ruled-out add method: under `api`, *Add another account* goes straight to masked API-key entry ([tB-16](captures/tB-16-forced-tui.txt)). Checked at session start (`exec`, `exec resume`, TUI) only. Not checked: spawned agents, `corbanu fork` under a forced login, and the daemon path (#595). |
| B8 | A rejected refresh or key names `corbanu login --account <name>`; the default is unchanged. | PASS | `bad`: "(account `bad`: sign in again with `corbanu login --account bad`; the default login is unchanged)". `fake` (CLI and TUI V4): "OpenAI (openai) account `fake` was rejected. Sign it in again…". |
| B9 | `OPENAI_API_KEY`, `CODEX_API_KEY` and `CODEX_ACCESS_TOKEN` are never used for a named account. | PASS | With all three decoys set, named runs sent only their own token or key. The mock saw `envkey` only on the default-account V5 run, and `codexenv` only on the default `exec` (default precedence, as before). **Real OpenAI:** `--account fake`, with the real key in `OPENAI_API_KEY` and `CODEX_API_KEY`, got 401 ([real](captures/real.txt)). |
| B10 | A real named OpenAI API-key sign-in works. | PASS | `login --account real --with-api-key` (stdin), then `exec --account real -m gpt-5.4`, answered `pong`. The fake default `auth.json` is unchanged ([real](captures/real.txt)). |
| B11 | TUI: `/providers` add, *Use for this session*, `/status` and `/cost`, across providers. (#571, #573) | **PASS** (#571, #573 fixed) | Every variant's request carried the right credential, and `/status` and `/cost` named it. **V2:** Z.AI `--account openai:work`, then `/model` → GPT-6 Luna, sent `chatgpt tuiwork` ([tB-04](captures/tB-04-571-V2-account-flag-then-model-switch.txt)). **V3:** *Use for this session* `work`, then `/model`, sent `tuiwork` ([tB-05](captures/tB-05-571-V3-use-for-session-then-model-switch.txt)). **V4:** named API key `fake` got 401 with recovery ([tB-06](captures/tB-06-571-V4-named-apikey-fake.txt)). **V5:** default, env key ([tB-07](captures/tB-07-571-V5-default-account.txt)). **V1:** direct start ([tB-08](captures/tB-08-571-V1-direct-start-and-fingerprint-across-refresh.txt)). **V6:** the default ChatGPT `tuidefault`, then named `tuiwork2`; `auth.json` unchanged ([tB-15](captures/tB-15-V6-default-chatgpt-then-named.txt)). No `/responses` request lacked credentials (0 `noauth` in mock-t). **#573:** `login --account work --device-auth` works with `model_provider = "zai"` ([t1-setup](captures/t1-setup.txt)), and on a Z.AI session *Add another account* offers device code and API key ([tB-10-b](captures/tB-10-b-573-add-methods-on-zai-session.txt)). The API key is masked ([tB-10-f](captures/tB-10-f-masked-api-key-entry.txt)). Connector auth not verified (see Other observations). Note: *Use for this session* starts a new conversation, and V2 switched with `/model` before its first turn. So no variant switched provider or account inside a conversation that already had turns (see Not exercised). |
| A10 | A TUI switch onto a named Bedrock account mid-session. (#572) | **PASS** (#572 fixed) | From a Z.AI session: *Use for this session* `Amazon Bedrock · work`, then `/model` → Other offers five `… via amazon-bedrock` models, plus `GLM 5.2 via zai-anthropic`. GPT-5.4 was signed `AKIDWORKPROFILE us-west-2`, with the `AWS_PROFILE=other`, env-key and bearer decoys set. `/status` shows `· account work`, and `/cost` shows `Amazon Bedrock · account work` ([tB-02](captures/tB-02-A10-model-other-tab.txt), [tB-03](captures/tB-03-A10-bedrock-turn-status-cost.txt)). *Use for this session* starts a **new conversation** ("To continue this session, run corbanu resume …", then a new session id). |
| B-live | Live ChatGPT sign-in | NOT VERIFIABLE | There is no disposable ChatGPT account, and Travis's login was not used. |
| C1 | Flag off: behaviour unchanged. | PASS | In a home made by BASE, CAND with the flag off used the same ChatGPT login. `login`, `logout`, `login status` and `exec` with `--account` give the same refusal text from BASE and CAND ([carry-flagoff](captures/carry-flagoff.txt)). In the TUI, the flag-off `/providers` rows are identical in BASE and CAND, with no named rows (the `/status` cards in those files differ only in session ids and token totals) ([tB-12](captures/tB-12-flag-off-providers-cand.txt)). See also REG-01. This is parity with BASE, which already has S06; pre-S06 parity is the 10-11 C1. |
| C2 | An existing single-account home carries over untouched. | PASS | The snapshot covers `auth.json`, `config.toml`, `installation_id` and skills. It excludes `log/`, `sessions/`, `tmp/`, `*.sqlite*`, `*.lock` and model caches. It was identical after CAND ran flag off, `--enable named_accounts`, and `--account default` / `openai:default`, and after the flag-off command parity runs. BASE still works on the home afterwards. The home `hm` was made by BASE, which already contains S06. Carry-over from pre-S06 is the 10-11 C2. |
| C3 | A missing or wrong account fails and never falls back. | PASS | `ghost`, `openai:ghost` and a logged-out `play` were refused at thread start, with 0 Responses requests ([b-cli](captures/b-cli.txt), C3 and B3 sections). A real-key `ghost` was refused with the real key in `OPENAI_API_KEY` ([real](captures/real.txt), oa-ghost). The TUI refusal of `fpx` before its sign-in was seen but its capture was overwritten, so it is not counted. |
| C4 | Isolation with fake keys and real providers. | PASS | Real named accounts answered `pong`: Z.AI `real`, Kimi `alt`, Claude Plan `cp`, OpenAI `real`. Each fake named account got the provider's 401, with the real key in that provider's env var where one exists ([real](captures/real.txt)). |
| C5 | GLM 5.3 Flash coordinator, TUI. | PASS | The coordinator ran four `corbanu exec` workers. Bedrock `work` was signed `AKIDWORKPROFILE`. Bedrock `gone` was refused before any request. OpenAI `fake` got 401 with recovery. OpenAI `work` was answered as `tuiwork`. The coordinator replied `pong` ([tB-11](captures/tB-11-C5-glm-coordinator-workers.txt), [mock lines](captures/tB-11-C5-mock-lines.txt)). |
| C6 | No credential in logs, rollouts, the state DB or captures. | PASS for everything scanned. The round-2 home state was deleted unscanned | **First full scan** ([leak-scan](captures/leak-scan.txt), 02:22 MST, 73,627 files): all disposable homes, fake HOMEs, logs and this folder up to that time. Each of the 4 real keys, and three 12-character fragments of each, is in **0 files**. Named API keys and named refresh tokens (including the same-user `~2` variants) appear in 0 home or log files; the only hits are the default logins' own `auth.json` files and my scripts. **Final scan** ([leak-scan-final](captures/leak-scan-final.txt), after all captures, over this folder and the run logs): 0 files for every real key and fragment. **Not scanned:** the homes used by the round-2 runs after 02:22 (`tB-19`, `555r3-*`, `tB-20`: `z4`, `z5`, `z6`, `t2`). They were deleted at clean-up (09:44Z) before a scan. Glitch: 0 files for the full Z.AI key only, with no fragment needles. |
| C7 | `/cost` per account (developer-accounting). | PASS under Permissive; **FAIL under Aggressive** ([#654](https://github.com/CorbanuCore/CorbanuTerminal/issues/654); not specific to named accounts, and also on BASE) | **Under Aggressive** (`security_levels`, saved through `/security` and restarted; the TUI shows `Aggressive active`), in every Aggressive turn tested (2 per build, default and named), the TUI warned "Developer accounting could not record one or more model requests", and at least one request went unrecorded. `/cost` was not captured under Aggressive. The log says `accounting: request sent unrecorded … cause=accounting route mismatch`. That holds on the default and on a named account, on both CAND and BASE ([tB-20 cand](captures/tB-20-cand-aggressive-default.txt), [log](captures/tB-20-cand-accounting-log.txt), [tB-20 base](captures/tB-20-base-aggressive-named-work.txt), [log](captures/tB-20-base-accounting-log.txt)). The same spawn onto another account under Permissive is recorded ([tB-17](captures/tB-17-spawn-other-account-accounting.txt)). **Under Permissive,** `/cost` lists these separately: `Z.AI · account work` / `personal` / `default` (555-FK09), `Amazon Bedrock · account work` ([tB-03](captures/tB-03-A10-bedrock-turn-status-cost.txt)), `OpenAI · account work` ([tB-04](captures/tB-04-571-V2-account-flag-then-model-switch.txt)), `OpenAI · account default` and `OpenAI · account fake` (401, no charge) ([tB-11](captures/tB-11-C5-glm-coordinator-workers.txt)). The Z.AI rows are in [555-FK09](captures/555-FK09-spawn-and-cost.txt). |
| D3 | Under Aggressive, a model-chosen account switch needs approval. | PASS for the approval leg (as the plan defines D3); the "refused with approvals off" leg was not exercised | The plan's decision table defines D3 as the **model-chosen** account: a spawn `account` "requires approval under `/security` Aggressive" (`unified-provider-auth.md` D3 row). The sprint note "switching account still needs approval (D3)" refers to that. Whether a user's own *Use for this session* should also prompt under Aggressive is a product question; see Decisions for Travis. *Cancel* was also tried: no child ran, and the model replied `refused` ([tB-14-D3-c](captures/tB-14-D3-c-cancel.txt)). That leg ran in the degraded "stored level unreadable … Aggressive enforced" state, which counts as Aggressive under the plan's D3/#428 row. With `security_levels` on and Aggressive active, a GLM `spawn_agent` call with `account = "work"` asked "Allow the spawned agent to run on account `work` of `zai` …?". ([tB-14-D3-a](captures/tB-14-D3-a-spawn-account-approval.txt)). After *Allow once*, the child ran on `account work` ([tB-14-D3-b](captures/tB-14-D3-b-after-allow.txt)). A user's own *Use for this session* is not model-chosen. It was not prompted **at selection**: a cross-provider choice of `openai · work` from a Z.AI session ([tB-13](captures/tB-13-D3-aggressive-user-switch-no-prompt.txt)). First use of that account, and a same-provider user switch, were not observed under Aggressive. |

## Verdicts: the 33 #555 cases

Homes follow the design: H-ZAI = `z1`/`z2`/`z3`/`z4`, H-OAI = `o1`, H-OAI-bare = `o2`, RS-03 = `o3`, H-NOKEY = `n1`.

- **Discriminating ("D").** The case ran where a wrong route would fail at the provider:
  - in `z4`, only `work` holds the real key, and `personal` and `default` are fake;
  - in `o1`, `work`'s key is the only Z.AI credential.

  "D" proves a request went to `work` (it answered) or did **not** go to `work` (401). A 401 cannot tell `personal`
  from `default`.
- **Attribution ("A").** The charge was read only from `/status` "Account usage" and `/cost`, in homes where all
  Z.AI accounts share one real key.

| Case | Pri. | Verdict | Evidence |
| --- | --- | --- | --- |
| FK-01 `/fork` stays on the source account | B | PASS (D) | `z4`: the fork answered `forked` on `account work`, `work` only ([555r2-FK01](captures/555r2-FK01-fork.txt)). First pass (A): "Thread forked from …". `/status`: `· account work` and "Forked from". Account usage: `work` only ([FK01](captures/555-FK01-fork-status.txt)). In `/cost`, this run's conversation is all `Z.AI · account work`. The home's other `default` (1) and `personal` (4) requests come from earlier sessions: `personal` chosen explicitly or by config, and `default` by precedence (the first manual PR-01/FK-06 pass) ([FK09](captures/555-FK09-spawn-and-cost.txt)). |
| FK-02 fork of a resumed conversation | B | PASS (D) | `z4`: resume with no flag answered `resumed`, and its fork answered `resfork`, both on `account work` ([555r2-FK02](captures/555r2-FK02-PR01b.txt)). First pass (A): `resume <id>` without a flag (config `personal`), then `/fork`: `account work`, `work` only ([FK02](captures/555-FK02-PR01b-resume-then-fork-status.txt)). |
| FK-03 edit an earlier prompt | B | PASS (D) | `z4`: the edited branch answered `edited` on `account work` ([555r2-FK03](captures/555r2-FK03-edit.txt)). It shows no "Forked from" line. First pass (A): Esc Esc, ← to the first prompt, edit: "You're continuing from this point in a new conversation". New session id, `account work`, `work` only ([FK03](captures/555-FK03-edit-prompt-status.txt)). |
| FK-04 `/side` | B | PASS (D) | `z4`: `side` and then `main` both answered on `account work` ([side](captures/555r2-FK04-side.txt), [main](captures/555r2-FK04-main.txt)). First pass (A): The side conversation is on `account work`. Esc Esc there shows "Editing previous prompts is unavailable in side conversations." After Ctrl+/ back to main: `account work`, `work` only ([FK04 side](captures/555-FK04-side-status.txt), [main](captures/555-FK04-main-after-side-status.txt)). |
| FK-05 `corbanu fork <id>` | B | PASS (D) | `z4`: answered `clifork` on `account work` ([555r2-FK05](captures/555r2-FK05.txt)). First pass (A): `account work`, "Forked from", `work` only ([FK05](captures/555-FK05-cli-fork.txt)). |
| FK-06 explicit `--account` beats the source | B | PASS (D: rules out `work` only) | `z4`: `--account zai:personal` ran on `personal` and got Z.AI's 401: "account `personal` was rejected" ([555r2-FK06](captures/555r2-FK06-expect-401-on-personal.txt)). First pass (A), both spellings: `--account personal` and `--account zai:personal` both run on `account personal`; `work` was not charged ([1](captures/555-FK06-cli-fork-account-personal.txt), [2](captures/555-FK06-cli-fork-account-zai-personal.txt)). |
| FK-07 fork after the source account was removed | B | PASS (wording nit, #588) | `corbanu fork <id>` exits 1: "This conversation ran on zai account `work` and **resumes** on it: … not configured; add it with `corbanu account add zai work` or pick another with `--account`". There was no fallback to `personal` (the config default). The in-session `/fork` can't be reached, because resume is refused the same way ([FK07](captures/555-FK07-fork-after-source-account-removed.txt)). |
| FK-08 fork of a fork | A | PASS (D) | `z4` ([555r2-FK08](captures/555r2-FK08-fork-of-fork.txt)). First pass (A): `account work`; "Forked from" shows the immediate parent ([FK08](captures/555-FK08-fork-of-fork-status.txt)). |
| FK-09 worker inherits the account | A | PASS (D) | `z4`: the child (`account work`) completed with `child` ([555r2-FK09](captures/555r2-FK09-spawn-cost.txt)). First pass (A): "Spawned … (glm-5.3-flash max · account work)". `/cost` for the conversation: `Z.AI · account work` only ([FK09](captures/555-FK09-spawn-and-cost.txt)). |
| RS-01 other provider, credentials present | B | PASS (D) | In `o1` (OpenAI home), `resume <zai-work-id>` loads the transcript with no onboarding and runs on Z.AI `· account work`, `work` only ([RS01 a](captures/555-RS01-a-after-launch.txt), [b](captures/555-RS01-b-resumed-status.txt)). |
| RS-02 other provider, credentials missing | B | PASS | `o2` exits 1 with no onboarding: "This conversation ran on zai account `work` and resumes on it: … add it with `corbanu account add zai work` or pick another with `--account`" ([RS02](captures/555-RS02-MSG03-missing-credentials.txt)). |
| RS-03 only `default` enrolled for the recorded provider | B | PASS | `o3`, with the real Z.AI key in `ZAI_API_KEY`: the same error naming `work`, exit 1, no fallback to `default` ([RS03](captures/555-RS03-only-default.txt)). |
| RS-04 picker matches resume by id | B | PASS (D) | `o1`: picker, Enter, then a turn answered `pickturn` on `account work` ([555r2-RS04](captures/555r2-RS04-o1-picker-enter-turn-status.txt)). First pass, without a turn: `o1`: picker → Enter → `account work` ([b](captures/555-RS04-b-o1-picker-enter-status.txt)). `o2`: picker → Enter → the RS-02 error ([d](captures/555-RS04-d-o2-picker-enter.txt)). |
| RS-05 resume twice | A | PASS | The second resume is still `account work` ([RS05](captures/555-RS05-resume-twice-status.txt)). |
| GH-01 `ghost` in config, recorded account exists | B | PASS (D) | `z4` with `zai = "ghost"`. The config and account list are recorded in [555r3-GH01-config](captures/555r3-GH01-config.txt). A conversation whose last turn ran on `work` (FK-05's fork) resumed and answered `ghostok` on `account work` ([555r3-GH01](captures/555r3-GH01-ghost-resume.txt), [r3_suite.sh](scripts/r3_suite.sh)). Before that, two other `z4` captures:<br>- `fk_suite2.sh`'s step resumed `$SRC`, whose last turn was the explicit `--account personal` attempt, so it resumed on `personal` ([discarded](captures/555r2-GH01-x-discarded-source-recorded-on-personal.txt)).<br>- A manual re-run with no recorded config ([555r2-GH01](captures/555r2-GH01.txt)) is superseded by r3.<br>First pass (A): `z2` (`zai = "ghost"`): resume runs on `account work`, `work` only ([GH01](captures/555-GH01-ghost-resume-on-work.txt)). |
| GH-02 recorded account missing too | B | PASS (does not mention `ghost`) | Exit 1, names `work`, and gives `corbanu account add zai work` ([GH02](captures/555-GH02-ghost-and-work-missing.txt)). The design's "ideally also mentions `ghost`" is not met. Control: `--account personal` resumes on `personal` ([GH02b](captures/555-GH02b-control-explicit-account.txt)). |
| GH-03 new session with `ghost` | B | PASS | Exit 1: "account `ghost` of provider `zai` is not configured; add it with `corbanu account add zai ghost`…" ([GH03](captures/555-GH03-ghost-new-session.txt)). |
| MSG-01 in-TUI wording | B | PASS | "■ … add it or pick another account in `/providers`". It doesn't mention `corbanu account add` or `--account`, and the session stays open ([MSG01](captures/555-MSG01-b-missing-account-message.txt)). |
| MSG-02 recover through `/providers` | A | PASS (A) | Z.AI → *Add another account* → `work` → masked key (pasted, shown as dots) → "Saved zai account `work`". Then `/resume` of the same conversation answers `pong`, with `account work` and `work` only ([MSG02 a–e](captures/555-MSG02-e-resumed-on-work-status.txt)). |
| MSG-03 command-line wording | B | PASS | It points to `corbanu account add zai work` / `--account`, not to `/providers` (RS-02, GH-02). |
| MSG-04 session after a failed in-app resume | A | PASS (A) | Same session id; `account personal` charged ([MSG04](captures/555-MSG04-session-after-failed-resume.txt)). |
| ESC-01 Esc in the `corbanu resume` picker, no credentials | B | PASS | `n1`: the picker footer says `esc quit`. Esc gives `EXITED-0`, with no onboarding ([a](captures/555-ESC01-a-picker.txt), [b](captures/555-ESC01-b-after-esc.txt)). That the terminal was restored was not observed directly: the captures show only `EXITED-0`. |
| ESC-02 Ctrl+C in that picker | A | PASS | `EXITED-0` ([ESC02](captures/555-ESC02-after-ctrl-c.txt)). |
| ESC-03 Esc in the in-session `/resume` | B | PARTIAL (draft untested) | In one `z4` session: picker (footer `esc exit`), then Esc, then `/status` shows the same id `01a13488…` and `account work` ([a](captures/555r2-ESC03-a-picker.txt), [b](captures/555r2-ESC03-b-after-esc.txt)). The first-pass captures came from two different sessions (`555-ESC03-a` was overwritten by the MSG run). Design ambiguity for disposition: as far as I could tell, `/resume` can only be typed into the composer, so I found no way to have a draft there while it opens. |
| ESC-04 normal onboarding | B | PASS (no BASE comparison) | In a fresh home, `corbanu` shows the provider sign-in choices ([ESC04](captures/555-ESC04-fresh-home-onboarding.txt)). |
| PR-01 full precedence | B | PASS (D for `work`; for `personal`/`default`, D rules out `work` only) | `z4`: resume `--account personal`, then a new session with config `personal`, both got Z.AI's 401 naming `personal`. Without `[provider_accounts]`, the 401 shows the unnamed-credential message ("Z.AI (zai) credential was rejected. Open /providers…"). Because `personal` and `default` both hold fake keys, a 401 shows only "not `work`". Which of the two it was rests on Corbanu's own message. These re-runs have no `/status`. Resume with no flag answered on `work` ([a](captures/555r2-PR01a-expect-401-on-personal.txt), [c](captures/555r2-PR01c-expect-401-on-personal.txt), [d](captures/555r2-PR01d-expect-401-on-default.txt), [b](captures/555r2-FK02-PR01b.txt)). First pass (A): Resume with `--account personal` → `personal (active)` ([a](captures/555-PR01a-resume-with-account-personal.txt)). Resume with no flag → `work` (FK-02). New session → `personal` ([c](captures/555-PR01c-new-session-noflag.txt)). New session without `[provider_accounts]` → `default` ([d](captures/555-PR01d-new-session-without-provider-accounts.txt)). |
| PR-02 `--account openai:x` on a Z.AI resume | A | DEVIATION (needs a disposition) | Exit 1 before the UI: "account `x` of provider `openai` is not signed in; …". It is reported clearly, but the Z.AI conversation does not resume, whereas the design expected Z.AI to keep using `work` ([PR02](captures/555-PR02-a-launch.txt)). Product authority should accept or reject this. |
| REG-01 `named_accounts` off | B | PARTIAL | FK-01 with ESC-03, RS-01 and ESC-01 run identically on CAND and BASE: no account suffix and no Account usage line. RS-01 fails the same way on both (`thread's recorded account zai:work: named accounts need the named_accounts feature …`, with a raw `Location:` trace that differs only in its line number, `app.rs:1042` against `:1037`). ESC-01 opens the Z.AI key onboarding on both, as before. So the Esc-in-picker path was **not** reached with the flag off ([cand](captures/555-REG01-cand-FK01-ESC03-flag-off.txt), [base](captures/555-REG01-base-FK01-ESC03-flag-off.txt)). The token totals differ only through Z.AI cache hits. The raw RS-01 report is filed as [#656](https://github.com/CorbanuCore/CorbanuTerminal/issues/656), low, pre-existing. |
| REG-02 no secrets shown | B | NOT MET (inherited; disposition requested) | C6. Masked entry shows dots only ([MSG02 c](captures/555-MSG02-c-key-masked.txt)). `corbanu login status` prints a masked prefix and suffix of the default API key (`sk-canar***y-S6B`, fake), which is inherited behaviour. The design says "any part of a key", so this is not met. `login status` never ran in a home holding a real key, and the real-key fragment scan would not catch this mask form. |
| REG-03 footer vs `/status` | A | NOT MET (UX follow-up) | The footer word after `via zai` is the reasoning effort, not the account: it reads `via zai high` with `model_reasoning_effort="high"` ([FK02](captures/555-FK02-PR01b-resume-then-fork-status.txt)). The footer never names the account, so "both name the same account" is not met. This is a UX follow-up, not a contradiction. Stronger evidence: the footer reads `via zai default` even on `personal` sessions (`555-MSG04`, `555r2-FK06`). |
| REG-04 usage totals | A | EXPLAINED | In a fork, `Token usage` counts the inherited history plus the fork's own requests, excluding cached input: 6,127 + 56 + 11 ≈ 6.19K. `Account usage` counts only the fork's requests, with cached input shown separately. Accounts that weren't used are omitted, not listed with 0 ([FK01](captures/555-FK01-fork-status.txt)). |
| REG-05 "Account:" line | A | OPEN OBSERVATION (unexplained) | The line reads `Z.AI API key configured` after a turn in RS-01 (resume by id, [RS01 b](captures/555-RS01-b-resumed-status.txt)) and GH-01. It is missing in RS-05 (resume, no turn), and also in `555r2-RS04` (picker resume, **after** a turn). So "no turn yet" does not explain it. Even when present, the line does not name the active account on Z.AI (`Z.AI API key configured`), unlike OpenAI's `OpenAI API key (account fake)` (`tB-06`). This is in the #609 status-detail family. |
| REG-06 both platforms | A | PARTIAL (7 of 22 blocker cases, attribution only) | Linux (glitch, a fresh clone of `53c02c9301`): FK-05, FK-06, GH-01, GH-02/MSG-03, GH-03 and ESC-01 match macOS. Real-key scan on glitch: 0 files ([linux-reg06](captures/linux-reg06.txt)). |

(Pri.: B = blocker, A = advisory.)

## Known open items (confirmed, not re-filed)

- **[#595](https://github.com/CorbanuCore/CorbanuTerminal/issues/595), open.** A TUI that reuses a running app-server
  daemon still logs out a ruled-out default login. NOT VERIFIABLE here: `app-server daemon start` needs the managed
  standalone install. The embedded-server path does leave the default in place (B4, [tB-16](captures/tB-16-forced-tui.txt)).
- **[#609](https://github.com/CorbanuCore/CorbanuTerminal/issues/609), open.** Status wording, seen again:
  - On a Z.AI session, `/providers` marks `OpenAI · default` and `Amazon Bedrock · default` as "this session"
    ([tB-01](captures/tB-01-providers-rows-zai-session.txt)).
  - With a default ChatGPT login, `/status` after a switch says "OpenAI API key configured (run corbanu login to use
    ChatGPT)" (V6).
- **[#588](https://github.com/CorbanuCore/CorbanuTerminal/issues/588), open.** A fork's missing-account error says
  "resumes" (FK-07). #588 also asked for the 33 #555 cases to be run by a separate executor; this run does that.

## Other observations (no issue filed)

- **Default logins under forced login methods.** These checks used my 401-on-missing-credentials mock and the TUI.
  The `exec` path was not compared with BASE in this run (see below); the docs scope the fix to named sessions.
  - A default session still logs out a ruled-out default login, both via `exec` (b-forced case 3) and in the TUI on Z.AI
    (tB-16, where BASE does the same).
  - The `exec` default session then still sends one `/responses` request **without credentials** (b-forced case 3).
    BASE was not run for this case. The 10-11 run showed it on its candidate and on pre-S06 (its oa-forced cases 3
    and 4, where the mock answered 200).
- **Connector traffic seen at the mock.**
  - **In homes without a default ChatGPT login,** `/backend-api/codex/plugins/featured` carries no credentials in
    named-account and Z.AI sessions: 35 requests in mock-t, 13 in mock-b1, 16 in mock-b4.
  - **In a home with one** (t2, `tB-19`), it carries the default token instead (#658).
  - A default-only ChatGPT home sends its token (6 of 6 in mock-c).
  - `ps/plugins/suggested` uses the named token only partly (2 named against 5 default in `tB-19`).
  - This still does not show the docs' "connectors … use it".
- **Default-login token on a named-account session: filed as
  [#658](https://github.com/CorbanuCore/CorbanuTerminal/issues/658).** In a session started directly on named `work`, with
  a default ChatGPT login present
  ([tB-19](captures/tB-19-named-start-default-token-traffic.txt)), the default token is sent for these requests, which
  the docs and gate don't list:
  - `ps/plugins/installed` (34), `ps/plugins/list` (5) and `ps/plugins/suggested` (5, plus 2 with `work`);
  - `plugins/featured`;
  - `wham/settings/user` and `wham/rate-limit-reset-credits`.

  Two others look documented, but neither fits the documented limit:
  - **Analytics.** The gate says analytics "keep the manager the session started with". This session **started** on
    named `work`, yet both analytics posts carried the default token, so it is a further #658 gap (added to the issue).
  - **`wham/usage`.** It is documented only "before the first reply", and I did not check the timing.

  `connectors/directory/list` (the `/apps` directory) is documented.
- **Default-login traffic in a process whose session uses a named account (V6 timeline).** Requests that used the default
  `tuidefault` token while a named account ran: analytics events, `ps/plugins/installed|list`, and
  `wham/settings/user`. In t2, which has a default login, `plugins/featured` also carried the default token during V6.
  - V6 **started** on the default account and then switched, so its analytics on the default token match the gate's
    limit. The plugin and settings requests are the #658 gap.
  - Connector (MCP) auth could not be checked: my mock does not serve MCP.
- **Refresh churn.** With 8 s access tokens, each `exec` refreshes about 30 times. The chain stays consistent. This is
  the same as the first acceptance.
- **Default ChatGPT login with `model_provider = "zai"`.** `corbanu login --device-auth` says "ChatGPT login is
  disabled. Use API key login instead." (exit 1; first line of [tB-15 setup](captures/tB-15-V6-setup.txt)). This is
  inherited; #573 only covered named accounts. With `-c model_provider="openai"` it works.
- **Rejection wording in the TUI.** A rejected **named** Z.AI account points to `corbanu account add zai personal`
  (`555r2-FK06`, `PR01a`). A rejected default credential points to `/providers` (`PR01d`). This is in the #609/#588
  wording family.
- **MSG-01 capture.** The composer line reads "Runs/review". That is a placeholder hint overlapping a stray keystroke,
  and it has no effect.
- **A login hint.** `corbanu login status` prints a masked prefix and suffix of the default API key
  (`sk-canar***y-S6B`).

## Re-runs and discarded attempts

- **First manual FK/PR pass on `z1`** (thread `01a13438…`): its captures were empty because of the line-count bug, and
  it was replaced by `fk_suite.sh`. Its requests remain in `z1`'s `/cost`: `default` 1 and `personal` 4.
- **First `rs_suite.sh` run:** discarded, because a shared tmux server carried one home into the next sessions. All
  `o*`/`n1` homes were rebuilt and the suite re-run. `rs_fixups.sh` re-captured RS-02, RS-03, ESC-01b, ESC-02, GH-02,
  GH-03 and RS-04b/d, whose second-run captures were empty. ESC01-a and ESC01-b therefore come from two processes in
  the same home.
- **First C5 run:** its capture was empty, so it was re-run. Both runs match.
- **First V1 run (`tB-08`):** `fpx` was not yet signed in, and the TUI refused it before the UI. That capture was
  overwritten by the successful run after sign-in, so C3 doesn't count it.
- **First-pass `555-ESC03-a`:** overwritten by the MSG run (another session). Superseded by `555r2-ESC03-*`.
- **Review round 1:** the charge-discriminating re-runs in `z4` (`555r2-*`), the RS-04 turn, D3 *Cancel*, and the
  accounting checks `tB-17` and `tB-18`.
  - `tB-18-b` failed on the fixture: `work` had been removed from z3 by FK-07.
  - `tB-18`, `tB-18-c` and `tB-14-D3-c` ran in a **degraded** security state. I had deleted `security_level.toml` but
    not its lock, so the TUI showed "stored level unreadable, Aggressive enforced". That is a fixture side effect.
    Superseded for C7 by `tB-20` (round 2).
  - The first `555r2-GH01` try is kept as `-x-discarded`.
- **Review round 2:** `r3_suite.sh` ran twice. In the first pass, the `/security` step pressed Esc instead of
  confirming, so the "Aggressive" sessions actually ran under Permissive. Their `tB-20` captures (no accounting warning)
  were deleted and replaced by the second pass, which saved Aggressive. Both passes are in `tui-invocations.txt`
  (lines 115–130).
- **Review round 2, other runs:** GH-01 scripted with its config recorded (`555r3-*`). Accounting under a cleanly saved Aggressive
  level, CAND and BASE (`tB-20-*`). The default-token traffic check (`tB-19`). The t1 file listing (`tB-10-i`).

## Not exercised

- **Browser sign-in.** The local-callback flow would open the host browser.
- **Unreadable stored sign-in.** Logout and remove can't be checked from outside, because no command writes a corrupt
  vault entry.
- **A refused add revoking tokens it already obtained.** This needs a race. The name clash is refused before sign-in.
- **Same-account concurrent processes (B2).** Covered in the 10-11 acceptance; not re-run here.
- **`corbanu fork` under a forced login, and `corbanu fork` with a missing `[provider_accounts]` entry.** Both are
  documented in `docs/provider-accounts.md` and #588.
- **The running app-server daemon path** (#595).
- **A same-provider spawn on a named ChatGPT account.**
- **Spawned agents with a named ChatGPT `account`, and spawned agents under login restrictions.** Spawned agents keep
  the session's provider, and the mock can't make tool calls. D3 was checked on Z.AI.
- **D3 "refused with approvals off".** Under Aggressive, approvals stay on even with `approval_policy = "never"` in
  config. The z3 runs still asked, and `/status` shows "Profile corbanu-aggressive (workspace, untrusted)"
  ([tB-14-D3-a](captures/tB-14-D3-a-spawn-account-approval.txt)). I found no external way to reach that leg.
- **A provider or account switch inside a conversation that already has turns.** *Use for this session* starts a new
  conversation, and V2 switched before its first turn. That is the path where history carries across (#578 in the
  gate).
- **Live checks:** live ChatGPT (B-live) and live Bedrock (slice A).

## Decisions for Travis

1. **PR-02.** `resume <zai-id> --account openai:x`, with `openai:x` missing, refuses the whole launch instead of
   resuming the Z.AI conversation and reporting `openai:x`. Options:
   - (a) accept the refusal, since it is fail-closed and clear. It is documented behaviour: `docs/provider-accounts.md`
     says an account named with `--account` "(of any provider)" is checked before the session starts. The mismatch is
     between the frozen #555 design and the docs;
   - (b) resume, and warn about the unused other-provider account.

   Recommendation: (a).
2. **D3 scope.** Under Aggressive, a model-chosen account (spawn `account`) asks for approval. A user's own
   *Use for this session* does not ask at selection. This was observed only for a cross-provider choice, and not at the
   account's first use. Options:
   - (a) keep it as is: the user's explicit choice is the approval;
   - (b) prompt for user switches too.

   Recommendation: (a).
3. **REG-02 (a #555 blocker case, NOT MET, inherited).** `corbanu login status` shows a masked 8-character prefix and
   5-character suffix of the default API key. Options:
   - (a) accept it as inherited behaviour outside S06;
   - (b) file it and mask the key fully.

   Recommendation: (b), as a separate low-priority issue. It does not block S06.
4. **Limited testing.** This run's isolation was by instruction only, and it used debug builds, not the packaged
   candidate. The B/C/D criteria were derived by me from the
   gate, not from a frozen independent design (only the #555 cases come from one). The agreement should also cover
   these untested or partial #555 legs:
   - ESC-03's draft;
   - REG-01's flag-off Esc-in-picker leg, which never reaches the picker;
   - GH-02 not mentioning `ghost`;
   - the z4 PR-01 re-runs having no `/status`;
   - ESC-04 having no before-build comparison;
   - ESC-01's "terminal is restored", which was not observed (only `EXITED-0`);
   - D3's "refused with approvals off" leg.

   Recommendation: accept slice B under an explicit limited-testing agreement that names these legs. Keep the sprint's
   "code-blind functional design frozen" item open.

## Recommendation

**Accept S06 slice B behind the flag, under an explicit limited-testing agreement.**

- Isolation was by instruction only.
- The builds were debug builds, without their companion binaries, not the packaged candidate.
- The B/C/D criteria came from the gate, not from a frozen independent design.
- This run does not close the sprint's "code-blind functional design frozen" item.

The four defects that held slice B are fixed on `53c02c9301`:

- #570: a named session no longer touches the default login; BASE still deletes it.
- #571: switching across providers sends the account's ChatGPT token.
- #573: named ChatGPT sign-in works from a Z.AI configuration.
- #572: Bedrock models are offered after *Use for this session*.

Every other slice B criterion passes again on what could be exercised. These remain NOT VERIFIABLE or not
exercised:

- B-live;
- the B1 keychain;
- B3's browser flow and the refused-add revocation;
- B7's spawned agents, `fork` and daemon paths.

Two findings:

- C7 fails under Aggressive ([#654](https://github.com/CorbanuCore/CorbanuTerminal/issues/654)). In every Aggressive
  turn tested, at least one request went unrecorded. This also happens on BASE and on the default account.
- The default-login token is still sent for plugin listings and settings
  ([#658](https://github.com/CorbanuCore/CorbanuTerminal/issues/658)).
  - This contradicts B4's "never read" and the docs' "plugin installs … use it".
  - It may also touch the gate's "Plugin installs read the active sign-in per use" and the sprint's checked
    "mid-session switch" Done item. I read plugin **listings** as not **installs**, so I don't mark that item failed.
  - It should be fixed or documented before the flag is removed.

For the **22 #555 blocker cases**, 19 pass. FK-01–06, RS-01, RS-04, GH-01 and PR-01 are checked at the provider (D),
with the limits stated for FK-06 and PR-01. The other three:

- ESC-03: PARTIAL, because the draft was untested.
- REG-01: PARTIAL, because flag-off ESC-01 never reaches the picker.
- REG-02: NOT MET, inherited. A masked fragment of the default API key appears in `login status`.

Advisory items:

- PR-02 refuses the launch, which is documented behaviour; it needs a disposition.
- REG-03, the footer naming no account, is a UX follow-up.
- REG-06 ran a Linux subset, with attribution only.
- REG-04 is explained.
- REG-05 is an open observation: the `Account:` line is sometimes missing after a resume.
- FK-08, FK-09, RS-05, MSG-02, MSG-04 and ESC-02 pass.

The remaining wording nits are tracked in #588, #609 and #656.

**Before the flag is removed:**

- Travis decides #595 (daemon reuse).
- #658 is resolved, together with the sprint's own Remaining item: early `/status` rate limits and the `/apps`
  directory still come from the default login.
- Live ChatGPT and live Bedrock are checked.

## Clean-up

Done at 2026-10-13T09:44Z, before the PR, with `rm -r … < /dev/null` (exit 0):

- every disposable Corbanu home and fake HOME under the scratch folder. That includes every vault that held a real
  key (`r1`, `z1`–`z6`, `o1`, `n1`) and its fallback key file;
- both target directories, the copied binaries, the BASE worktree and the stale tmux sockets.

`ls` then shows only `build.sh`, `logs/` (mock and run logs, scanned in C6) and the review files
([cleanup.txt](captures/cleanup.txt), taken at 09:55Z after the deletion). The `rm` exit codes (0) were seen in the
console and not captured. In `cleanup.txt`, paths were shortened by `sed`: the `/Volumes/CorbanuDrive/Corbanu/` prefix was
dropped, and `/home/ambient` shows as `~`. The remote glob was expanded by the remote shell, so `ls` reporting no match
means nothing was left. On glitch, `~/corbanu-rtx/s06b*`
(clone, target, homes) was deleted after REG-06; a later `ls -d ~/corbanu-rtx/s06b*` finds nothing.

## Reproduce

- Scripts are in [`scripts/`](scripts/) ([env.sh](scripts/env.sh) has this host's paths).
- CLI:
  1. `b_cli.sh`
  2. `b_forced.sh`
  3. `real.sh`
  4. `carry_b.sh`
- TUI:
  1. Start `mock.py --port 34731` (t1/t2) and `mock.py --port 34741` (o-homes), then run `source z_setup.sh init`.
  2. `fk_suite.sh`, then `rs_suite.sh` and `rs_fixups.sh`, then `msg_suite.sh` and `regoff.sh`.
  3. After review round 1: `fk_suite2.sh`. Its GH-01 step gives the discarded result.
  4. After review round 2: `r3_suite.sh`, which runs the kept GH-01 and the `tB-20` Aggressive accounting runs on CAND
     and BASE.
  5. The slice-B TUI steps: `source tdrv.sh <name>`, as in the capture order `tB-01` … `tB-19`. V6 and `tB-19` use
     `v6_setup.sh init`. Every launch line is in [tui-invocations.txt](captures/tui-invocations.txt).
- Linux: `linux_launch.sh`.
- Leak scan:
  - `S=… EV=… python3 leak_run.py`, the first full scan;
  - `LEAK_ROOTS="$EV:$S/logs" … python3 leak_run.py`, the final scan.

## Changes after the final review (round 5)

All of round 5's should-fix items and nits were applied without new test runs.

- **Analytics.** Analytics in a session that started on a named account is a #658 gap, not the gate's limit. I added
  it to #658. `wham/usage` timing was not checked.
- **Decision 4** now also names ESC-04's missing before-build comparison, ESC-01's unobserved terminal restore and
  D3's approvals-off leg. The D3 verdict is qualified to the approval leg.
- **Rows:**
  - REG-05 notes the Z.AI `Account:` line naming no account (#609 family).
  - REG-06 counts 7 blocker cases.
  - REG-01 notes the line-number difference.
  - B11 and D3 cite their captures.
- **Re-runs** lists the overwritten `fpx` refusal.
- **Clean-up** explains the shortened paths.
- **Wording:** the Method wording on reviews, the #588 sentence and the list formatting.

