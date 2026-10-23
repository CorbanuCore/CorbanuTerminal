# PF-84-S06 independent functional acceptance (2026-10-11)

Scope: [PF-84-S06](../../../../docs/sprints/current/unified-provider-auth/pf-84-s06-aws-and-chatgpt-accounts.md),
named Amazon Bedrock AWS-profile accounts (slice A) and named ChatGPT/OpenAI sign-in accounts (slice B), behind
`named_accounts`. Gates under test: [s06a-gate.md](../s06a-gate.md), [s06b-gate.md](../s06b-gate.md). Independent
review: [REVIEW.md](REVIEW.md).

## Method

- **Code-blind.** I read only these:
  - the S06 sprint record and the plan's PF-84 section;
  - the S06 gate files, including the gate's mock scripts, which I read to learn the test hooks;
  - the earlier acceptances' README/REVIEW, for method;
  - `qa/demos/index/PF-84-S0*.md`, `docs/provider-accounts.md`, `docs/authentication.md` and `docs/tmuxHarness.md`;
  - `sec-common.md` and `--help` output.

  I did not read `codex-rs/**`, `scripts/**`, PR diffs, worker logs or reviews. Every observation comes from the
  outside: CLI output, TUI screens, files in the disposable homes, and what my own mock server received.
- **Candidate.** `origin/main` at `a22fdb903d` (merge of #557, slice B; slice A is #543). There were two debug builds
  with `codex-core/developer-accounting,codex-tui/developer-accounting`, toolchain 1.95.0, each with its own target dir:
  - CAND: `a22fdb903d`;
  - BASE: `3a0a38bed2`, the pre-S06 first parent of #543, for the flag-off and carry-over comparisons.
- **Safety.**
  - Every candidate and baseline process ran with `env -i`, a disposable `CORBANU_HOME`, a disposable `HOME` (fake
    `~/.aws` only) and `CORBANU_TEST_NO_NATIVE_KEYRING=1` ([env.sh](scripts/env.sh), [cleanexec.py](scripts/cleanexec.py)).
  - Real keys came only from `/Users/Neo/.local/bin/corbanu.bak-20261010 vault auth-helper <label>`, inside the
    consuming command. They went to stdin (`account add`, `login --with-api-key`) or to a process environment, never
    to argv, and were never printed.
  - Keyed TUIs ran on their own tmux servers, which were killed afterwards.
  - Travis's ChatGPT login and the real `~/.local/bin/corbanu` were never touched.
- **Mocks.** I wrote one stdlib server, [mock.py](scripts/mock.py), not the gate's mocks. It plays these parts:
  - **Bedrock Mantle.** It logs the SigV4 access key id, region and session-token presence, or which bearer canary it saw.
  - **Credential sources a named account must never touch.** IMDS, ECS container credentials, STS and the SSO portal.
    Each answers with a distinctive fake key id, so a fallback would show.
  - **OpenAI issuer.** Device code, code exchange, refresh with rotation (a reused refresh token is rejected) and
    revoke.
  - **OpenAI Responses.** It reports the user, token generation or API-key canary, and sends rate-limit headers.

  All values are fake. For slice A, every named-account run had decoys of every other kind in its environment:
  `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`/`AWS_SESSION_TOKEN`, `AWS_PROFILE=other`,
  `AWS_BEARER_TOKEN_BEDROCK`, IMDS, ECS and STS endpoints pointing at the mock, and a web-identity role. For slice B,
  the decoys were `OPENAI_API_KEY`, `CODEX_API_KEY` and `CODEX_ACCESS_TOKEN`.
- **Real providers.**
  - Z.AI `glm-5.3-flash` (`provider/zai_api_key`): the TUI coordinator, named `real`, and `fake`.
  - Kimi `kimi-k3` on `kimi-code` (`provider/kimi_api_key`): named `alt` and `kfake`.
  - Claude Plan (`claude-plan-test-token`): named `cp` and `cpfake`.
  - OpenAI `gpt-5.4` (`openai-api-key`): a named API-key sign-in `real`, made with `login --account real --with-api-key`,
    and a named `fake`.
- **TUI.** The TUI ran through private tmux sockets per `docs/tmuxHarness.md`
  ([tmuxlib.sh](scripts/tmuxlib.sh), [launch lines](captures/tui-invocations.txt)).

## Verdicts

| # | Criterion | Verdict | Evidence |
| --- | --- | --- | --- |
| A1 | An AWS-profile account stores only the profile name: validated, not secret, shown in clear. | PASS | `account add … --kind aws-profile --value work`. The candidate's own `vault auth-helper` returns just `work`. Bad names and a missing `--value` are refused, and `--kind api-key` is refused for Bedrock. `account list` and `/providers` show `AWS profile \`work\``, and `/vault list` shows `provider/amazon-bedrock/accounts/<name>/aws_profile`. `~/.aws` is byte-identical before and after. 0 Corbanu-home files hold an AWS secret ([aws-cli](captures/aws-cli.txt), [providers](captures/tui-4-providers.txt), [vault](captures/tui-16-vault-list.txt), [leak scan](captures/leak-scan.txt)). |
| A2 | It signs only with that profile: env keys, `AWS_PROFILE`, the bearer token, and env/instance/container sources are ignored. | PASS (managed-key leg not exercised) | With every decoy set: `work` was signed by `AKIDWORKPROFILE`, `alt` by `AKIDOTHERPROFILE`, a renamed account by its profile, and `credential_process` by `AKIDPROCESSHELPER`. A `source_profile` role chain made STS calls signed by `work`, then signed with `ASIAROLECHAIN`. The mock saw **0** IMDS, ECS or web-identity calls ([aws-cli](captures/aws-cli.txt), [mock](captures/mock-a.jsonl)). I found no way to set a *managed* Bedrock key: the `/providers` default row only offers *Add another account*. |
| A3 | `credential_source` Environment, Ec2InstanceMetadata or EcsContainer is refused. | PASS | Each one was refused with the reason and recovery text. The mock received 0 requests per run. |
| A4 | A missing or broken profile is refused before any request, with no fallback. | PASS | **0 mock requests** each. These ended with the reason and "Corbanu never falls back to another profile or to environment credentials": a missing profile, a profile with no region and no `aws.region`, a failing `credential_process` (stderr withheld: the helper canary is in 0 Corbanu files), and an SSO profile with no cached token. These were refused at thread start with "is not configured; add it with … --kind aws-profile …": an unconfigured account `ghost`, `[provider_accounts]` naming `ghost`, and `keyacct` (whose `--kind api-key` add had been refused) ([aws-cli](captures/aws-cli.txt), [aws-cli2](captures/aws-cli2.txt)). In the TUI, `gone` showed the refusal with 0 requests, and `--account ghost` exited 1 before the UI ([tui-19](captures/tui-19-gone-refused.txt)). |
| A5 | SSO profiles are allowed. | PASS (mock); live NOT VERIFIABLE | An `sso_session` profile with a cached token asked the SSO portal (the profile's `services` endpoint, the mock) for role credentials and signed with `ASIASSOPROFILE` plus a session token. A real `aws sso login` was not run (no AWS account). |
| A6 | Region: `aws.region`, else the profile's region. `AWS_CONFIG_FILE` and `AWS_SHARED_CREDENTIALS_FILE` are honoured. | PASS | A config region beats the profile's region, and `-c …aws.region` overrides per run. With no config region the profile's region is used, and the `AWS_REGION=ap-south-1` decoy is ignored. The alternate files signed with `AKIDALTFILEWORK` in `sa-east-1` ([aws-cli2](captures/aws-cli2.txt)). |
| A7 | The default Bedrock account keeps today's order. | PASS | With decoys, the bearer env wins. Without it, the SDK chain is used (env keys). In the carried-over home, the baseline and the candidate both signed with `AKIDDEFAULTPROFILE` ([carry](captures/carry-flagoff.txt)). |
| A8 | CLI add/list/rename/remove, and the `/providers` row and add flow. | PASS | CLI: [aws-cli](captures/aws-cli.txt). TUI: *Add another account* takes a name, then a profile. An invalid profile is refused with the rule, and a valid one is saved ([tui-14](captures/tui-14-bedrock-add.txt)). |
| A9 | `/status` and `/cost` name the Bedrock account, and the selection survives a restart. | PASS | `Amazon Bedrock … · account work`, `Account usage: work · 120 input · 9 output`. `/cost` shows `Amazon Bedrock · account work`. *Make default* wrote `[provider_accounts] amazon-bedrock = "<name>"`, and after a restart without `--account` the turn ran on that account. `tuialt`'s profile is `other`, the same name as the `AWS_PROFILE` decoy, so I repeated the restart with a profile no decoy names. `third` was signed by `AKIDTHIRDPROFILE` and `/status` showed `· account third` ([tui-8](captures/tui-8-bedrock-turn.txt), [tui-9](captures/tui-9-bedrock-cost.txt), [tui-18](captures/tui-18-restart.txt), [tui-21](captures/tui-21-review-fixes.txt)). |
| A10 | A TUI switch onto a named Bedrock account mid-session. | **FAIL** (low, [#572](https://github.com/CorbanuCore/CorbanuTerminal/issues/572)) | *Use for this session* says "requests to amazon-bedrock use account `work`", but `/model` offers no Bedrock model in any tab. The full lists are Z.AI 3, Other 1 (`zai-anthropic`) and OpenAI 6. So the session cannot reach Bedrock ([tui-20](captures/tui-20-bedrock-switch.txt), [tui-21](captures/tui-21-review-fixes.txt)). The baseline picker was not checked. Starting on Bedrock works (A9). |
| A-live | Live Bedrock | NOT VERIFIABLE | There is no AWS account or profile (Travis, 2026-10-11). |
| B1 | Each sign-in account has its own vault store: no keychain item, never `auth.json`. | PASS (keychain: NOT VERIFIABLE) | `/vault list`: `provider/openai/accounts/<name>/chatgpt_auth`, one per account. The default `auth.json` stayed byte-identical (`2bbf0ff0…`) through 8 named sign-ins, all refreshes and the logouts. No clear-text named token or key is in any home file. Under the no-keychain rule, only "no item created" could be observed, not the keychain itself ([oa-cli](captures/oa-cli.txt), [vault](captures/tui-16-vault-list.txt), [leak scan](captures/leak-scan.txt)). |
| B2 | A refreshed token is saved back to that account only, and other processes pick it up. | PASS | An expired `r1`: process 1 refreshed once (gen 1→2), and a new process 2 used gen 2 with **no** refresh. `work` and `play` ran in parallel, each on its own chain. Over a 146-step chain across about 8 processes, the mock saw **0** `refresh_token_reused` rejections. `twin`, the same person as `work`, kept its own chain. Two processes on one expired account both succeeded. The default `deflt` was never refreshed or sent ([oa-cli](captures/oa-cli.txt), [oa-cli2](captures/oa-cli2.txt), [mock-b](captures/mock-b.jsonl)). |
| B3 | `login --account`, `logout --account`, status, rename and remove work. | PASS (browser flow not exercised) | Device code and `--with-api-key` both work. `login --account x status` shows the kind, email hint and plan, and a missing account exits 1. `logout --account play2` revoked only `play2`, and a second logout is harmless. Re-login as another user revoked the replaced `r1` token; re-login as the same user revoked nothing. `account rename` kept the tokens, and `account remove` revoked and deleted only that account ([oa-cli3](captures/oa-cli3.txt)). The browser callback flow was not exercised because it would open the host's browser. |
| B4 | The default login (`auth.json`) is never read or changed by a named account. | **FAIL** (high, [#570](https://github.com/CorbanuCore/CorbanuTerminal/issues/570)) | Normal use passes: the `auth.json` hash is unchanged everywhere, and a named session still works with `auth.json` at `chmod 000`. But under `forced_login_method` / `forced_chatgpt_workspace_id` that rule out the default, a named-account session **deletes the default `auth.json`**. That happens even when the named session is allowed and answered (cases 1 and 5). The baseline does the same only for default sessions ([oa-forced](captures/oa-forced.txt)). Under those restrictions, the default login and env keys also decide whether the named session runs at all: in oa-cli, the `CODEX_API_KEY` decoy triggered "ChatGPT login is required … Logging out." with exit 1, and case 7 refused a named API-key session because of the default login's workspace. |
| B5 | Display: kind, email hint, plan and fingerprint; `/status` names the sign-in. | PASS | `ChatGPT sign-in · w…@example.com · Plus · fp 03c266ec0285`, and `OpenAI API key · fp …`. `/status` shows `Account: ChatGPT sign-in · d…@example.com · Plus (account devcode)`. After the reply, the `5h limit` is the named account's own (51% left, from its headers) ([tui-4](captures/tui-4-providers.txt), [tui-17](captures/tui-17-openai-devicecode.txt)). |
| B6 | Signing the same ChatGPT user in twice warns, not refuses. | PASS | `twin` warns that it is "the same ChatGPT user as account(s) \`work\`…", and `dup` names `default`. Both were saved ([oa-cli](captures/oa-cli.txt)). |
| B7 | `forced_login_method` and `forced_chatgpt_workspace_id` apply to named accounts. | **PARTIAL** (#570) | The named-account check works. ChatGPT `work` under `api` and `work` under `ws-other` were refused at thread start with recovery text, and `ws-main` is allowed ([oa-cli](captures/oa-cli.txt)). API-key `key` under `chatgpt` was refused at thread start in a home without env keys ([oa-cli2](captures/oa-cli2.txt), [oa-forced](captures/oa-forced.txt) case 2). But the default login's enforcement still runs for named sessions. It deleted `auth.json` in those runs, it refused `key` before the named check when a `CODEX_API_KEY` decoy was set (oa-cli: "…Logging out."), and it refused a named API-key session on the default login's workspace (case 7). |
| B8 | A rejected refresh or key names `corbanu login --account <name>`, and the default is unchanged. | PASS | `bad`: "account \`bad\`: sign in again with \`corbanu login --account bad\`; the default login is unchanged". `fake`: "OpenAI (openai) account \`fake\` was rejected. Sign it in again…". No other credential was sent. |
| B9 | `OPENAI_API_KEY`, `CODEX_API_KEY` and `CODEX_ACCESS_TOKEN` are never used for a named account. | PASS | With the mock and all three decoys, named runs only ever sent their own token or key. **Real OpenAI:** `--account fake`, with the **real** key in `OPENAI_API_KEY` and `CODEX_API_KEY`, got 401 `invalid_api_key` ([real](captures/real.txt)). |
| B10 | A real named OpenAI API-key sign-in works. | PASS | `login --account real --with-api-key` (key on stdin), then `exec --account real -m gpt-5.4` answered `pong`. `auth.json`, which holds a fake default, is unchanged. |
| B11 | TUI `/providers` add, *Use for this session*, `/status` and `/cost`. | PASS on OpenAI sessions; **FAIL** across providers ([#571](https://github.com/CorbanuCore/CorbanuTerminal/issues/571), medium) | On an OpenAI session: *Add another account* → device code saved `devcode`. *Use for this session* gave a turn answered as `devcode`. The add created no default login: home t1 had no `auth.json` afterwards (direct check in tui-17). API-key entry is masked, and `/cost` lists `OpenAI · account work` and `account fake` (401, no charge) ([tui-15](captures/tui-15-openai-add.txt), [tui-17](captures/tui-17-openai-devicecode.txt), [tui-12](captures/tui-12-openai-cost.txt)). From a Z.AI session, selecting the named **ChatGPT** account (`--account openai:work` or *Use for this session*) and then using `/model` → OpenAI sends the request with **no Authorization header**, while `/status` and `/cost` claim `account work`. API-key accounts and direct starts are fine. The default-account switch is fine too, with both an env key (V5) and a ChatGPT `auth.json` login (V6). So the defect belongs to S06's named ChatGPT path ([tui-11](captures/tui-11-openai-turn-status.txt), [tui-13](captures/tui-13-openai-variants.txt)). My mock answers unauthenticated requests with 200; the real API would return 401 and blame `work`. |
| B-live | Live ChatGPT sign-in | NOT VERIFIABLE | There is no disposable ChatGPT account, and Travis's login was not used. |
| C1 | Flag off: behaviour unchanged. | PASS | On a home made by the baseline, the candidate with the flag off uses the same credentials as the baseline (ChatGPT `legacy`, `AKIDDEFAULTPROFILE`). `account add`, `login --account`, `logout --account` and `exec --account work` are refused; the baseline rejects the unknown arguments, and its `exec --account` text is identical ([carry](captures/carry-flagoff.txt)). |
| C2 | An existing single-account home carries over untouched. | PASS | I snapshotted every file except `log/`, `sessions/`, `tmp/`, `*.sqlite*` and `*.lock`, plus `~/.aws`, before and after candidate runs with the flag off, on, and `--account default`. That covers `auth.json`, `config.toml` and the vault files. The snapshots are identical except `models_cache.json` (a model-list cache, refetched on use). The baseline still works afterwards. |
| C3 | A missing or wrong account fails and never falls back. | PASS | Bedrock: A4. OpenAI: `ghost`, `openai:ghost`, a removed account, and the old name after a rename are all refused at thread start, with 0 Responses requests. A real-key `ghost` is refused even with the real key in `OPENAI_API_KEY`. |
| C4 | Isolation with fake keys and real providers. | PASS | Real named accounts answered `pong`: Z.AI `real`, Kimi `alt`, Claude Plan `cp`, OpenAI `real`. Each fake named account got the provider's 401. For Z.AI, Kimi and OpenAI, the **real** key sat in that provider's env var at the time; Claude Plan `cpfake` got 401 too ([real](captures/real.txt)). The `exec --json` 401 on Z.AI and Kimi doesn't name the account (known [#480](https://github.com/CorbanuCore/CorbanuTerminal/issues/480)). |
| C5 | GLM 5.3 Flash coordinator, TUI. | PASS | The coordinator ran `corbanu exec` workers ([workers.sh](scripts/tui_setup.sh)): Bedrock `work` was signed by `AKIDWORKPROFILE`, Bedrock `gone` was refused with no request, OpenAI `fake` got 401 with recovery, and OpenAI `work` was answered as `tuiwork`. The coordinator replied `pong` ([tui-2](captures/tui-2-glm-workers.txt)). An in-process spawn onto Bedrock was refused by the spawn tool (agents inherit the session provider), so it is not verified ([tui-3](captures/tui-3-glm-spawn.txt)). |
| C6 | No credential in logs, rollouts, the state DB or captures. | PASS | [Leak scan](captures/leak-scan.txt) of 17,831 files, re-run after the review with the env-decoy values added. The env decoys appear only in my scripts, fake `HOME` files and launch log. The 4 real keys: 0 files. Fake AWS secrets, the SSO token, the helper stderr and the bearer env canary appear only in my fake `~/.aws`/helper files, scripts and my own launch log, never in a Corbanu home. Named API keys and refresh tokens: 0 files; the only hits are the *default* logins' own `auth.json`. |
| C7 | `/cost` per account (developer-accounting). | PASS (with #571) | `Amazon Bedrock · account work`, `OpenAI · account work`, `OpenAI · account fake` (401, no charge) and `Z.AI · account default` are each grouped separately ([tui-9](captures/tui-9-bedrock-cost.txt), [tui-12](captures/tui-12-openai-cost.txt)). Under #571, an unauthenticated request is filed under `work`. |

### Mock artefacts

- The `5h limit` lines on Bedrock and OpenAI come from the mock sending `x-codex-primary-*` headers. Per account they are
  distinct (`devcode` 51% vs 93%), which is what B5 relies on.
- The mock answers requests that carry no credentials with 200, so the #571 turns looked successful.
- A slice-A fallback would show mainly as `AKIDENVDECOY` or `envbearer`, since env keys and the bearer token come first in
  the default chain. The zero IMDS, ECS and web-identity hits are a secondary signal.

### Not exercised

- **TUI add rules (gate B3):** a TUI add must never overwrite an existing account, and a refused add must revoke its
  tokens.
- **Unreadable stores:** logout and remove on an unreadable stored sign-in.
- **Resume:** resuming a thread recorded on a named Bedrock or OpenAI account.
- **Spawned agents:** a same-provider spawned agent with a named ChatGPT `account`, and spawned agents under login
  restrictions.
- **Fingerprint:** stability after a refresh. The `fp` was not compared before and after.
- **Rename hint:** the note `account rename` prints when `[provider_accounts]` still names the old name. My rename was of
  an unselected account.
- **D3:** Aggressive security requiring approval for an account switch.
- **Flag off in the TUI:** flag-off parity was checked from the CLI only.
- **Same-user warning in the TUI:** the TUI's hiding of ruled-out add methods was seen only as #573.
- **Browser sign-in:** the local-callback flow.
- **Live checks:** live Bedrock and live ChatGPT.
- **Connector/MCP auth:** my mock does not serve MCP. Requests to `/backend-api/codex/plugins/featured` carried no
  `Authorization` header in every named-account home (b1 17/17, t1 19/19). The default-only home hm sent the default
  token (8/8). No default token leaked into named sessions, but whether connectors use the named account (the docs'
  claim) is not verified.

### Other findings (no issue filed)

- **Refresh churn with short-lived tokens.** When a ChatGPT access token expires within seconds, every use refreshes:
  about 30 refreshes per `exec` with 8 s tokens ([oa-cli](captures/oa-cli.txt)). The chain stayed consistent; real tokens
  live far longer. After a re-login, one process refreshed twice in a row with 1 h tokens
  ([oa-cli2](captures/oa-cli2.txt)).
- **Credential calls per request.** A role chain made 3 STS calls for one request, and SSO made 3 portal calls. The gate
  lists "no credentials cache" as a known limit.
- **Display nits.**
  - `exec` prints Bedrock refusals and 401 lines twice.
  - `/vault list` types the non-secret profile label as `[manual-secret]`.
  - `--value -leadingdash` needs `--value=…`. It is then refused correctly.
  - On a Z.AI session, `/providers` marks `OpenAI · default` as "this session".
  - The footer reads `GPT-5.4 via amazon-bedrock default` while `/status` shows `· account work`. S05 says the footer word
    is the reasoning effort.
  - After a default ChatGPT login is used by a switch, `/status` says "OpenAI API key configured (run corbanu login to
    use ChatGPT)" (V6).
- **Sign-in on a non-OpenAI provider.** With `model_provider = zai`, a named ChatGPT sign-in is refused by the CLI and
  hidden in the TUI. This is inherited from the default login: [#573](https://github.com/CorbanuCore/CorbanuTerminal/issues/573), low.

## Recommendation

**Do not accept S06 as complete yet. Slice A can be accepted behind the flag now.**

- **Slice A works.** Only the profile is used. Every bad profile is refused before any request. Decoys never leak in,
  and SSO works on the mock. Its one failure (#572) is a low-severity UX gap.
- **Slice B is solid in its core**: per-account vault stores, refresh write-back, revocation scope, same-user warnings,
  env-key isolation and real OpenAI API-key accounts. It has two defects:
  - [#570](https://github.com/CorbanuCore/CorbanuTerminal/issues/570) (high): a named session can **delete the default
    login** under login restrictions.
  - [#571](https://github.com/CorbanuCore/CorbanuTerminal/issues/571) (medium): a named ChatGPT account chosen across providers in
    the TUI sends **no credentials**, while `/status` and `/cost` claim the account.

  Neither leaks a secret or falls back to another credential. Fix both before accepting slice B.

The live Bedrock, live ChatGPT and browser sign-in checks remain open, as the sprint record says.

## Reproduce

- Scripts are in [`scripts/`](scripts/) and hard-code this host's paths ([env.sh](scripts/env.sh)).
- CLI runs, in order:
  1. `aws_cli.sh`, `aws_cli2.sh`
  2. `oa_cli.sh`, `oa_cli2.sh`, `oa_forced.sh`, `oa_cli3.sh`
  3. `real.sh`
  4. `carry.sh`
- TUI runs:
  1. `source tui_setup.sh init` with `mock.py --port 34631` running.
  2. The interactive steps, in capture order (`tui-1` … `tui-20`).
  3. `tui_variants.sh V1|V2|V3|V4|V5|V6`. V6 needs a default ChatGPT login in t1; the `BIN=$BASE` V6 run did not reach
     a model (picker mismatch) and is not used.
- Finally: `leakscan.py`, with the real keys passed through the environment from the vault helper.
