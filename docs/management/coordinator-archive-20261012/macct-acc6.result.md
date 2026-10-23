I tested S06 from the outside at `origin/main` `a22fdb903d` and can recommend accepting slice A behind the flag, but not slice B yet: two of its defects need fixing first. One code-blind reviewer audited the report (agree with changes); I applied its changes and saved its review as REVIEW.md. The docs-only PR **#574** merged once all 26 required checks passed (merge `4a2762c74c`). Evidence is in `qa/provider-auth/pf-84/independent-acceptance-s06-20261011/`.

I wrote my own local mock (fake Bedrock, AWS credential sources, OpenAI sign-in and responses) and compared against a pre-S06 baseline build (`3a0a38bed2`). Every run used throwaway homes with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, with fake AWS and OpenAI credentials of every other kind set in the environment. Real keys came only from the vault helper, and none appears in any capture or log.

**Slice A — Amazon Bedrock AWS-profile accounts**

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| A1 | Stores only the profile name | PASS | The vault value reads back as just `work`; bad names are refused; no AWS secret is in any Corbanu file. |
| A2 | Signs only with its profile | PASS | Each account signed with its own key (including a helper-process and a role-chain profile); no environment, instance or container credentials were touched. I found no way to set a managed Bedrock key, so that decoy wasn't tested. |
| A3 | Environment/instance/container credential sources refused | PASS | All three are refused, and the mock saw no request. |
| A4 | Missing or broken profile refused first, no fallback | PASS | Missing profile, no region, failing helper, SSO without a cached token, unknown account: all refused with 0 requests (TUI too). |
| A5 | SSO profiles allowed | PASS on the mock | It fetched role credentials and signed with them. A real `aws sso login`: NOT VERIFIABLE. |
| A6 | Region and file rules | PASS | The config region wins, else the profile's; `AWS_REGION` is ignored; alternate config/credentials files are honoured. |
| A7 | Default account unchanged | PASS | Same signing order as before; baseline and candidate signed identically. |
| A8 | Add, list, rename, remove (CLI and TUI) | PASS | |
| A9 | `/status`, `/cost`, restart | PASS | A restart with no `--account` ran on the account chosen as default for new sessions. |
| A10 | Switching onto a Bedrock account mid-session | **FAIL** (low, #572) | "Use for this session" is accepted, but `/model` offers no Bedrock model. Starting the session on Bedrock works. |
| Live | Real Bedrock | NOT VERIFIABLE | No AWS account. |

**Slice B — ChatGPT/OpenAI sign-in accounts**

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| B1 | Own vault store per account, never `auth.json` | PASS | One vault label per account; `auth.json` was byte-identical through 8 sign-ins and all refreshes. Whether a keychain item is made can't be observed under the no-keychain rule. |
| B2 | Refreshed tokens saved to that account only | PASS | A second process reused the refreshed token without refreshing; a 146-step token chain across processes was never rejected; the default login was never refreshed. |
| B3 | `login`/`logout --account`, status, rename, remove | PASS | Logout and remove revoke only that account. The browser sign-in path wasn't exercised. |
| B4 | Default login never read or changed | **FAIL** (high, #570) | When login restrictions (`forced_login_method` or `forced_chatgpt_workspace_id`) rule out the default login, a named-account session deletes the default `auth.json`, even when the named session itself is allowed. |
| B5 | Display and `/status` | PASS | Rows show kind, email hint, plan and fingerprint. |
| B6 | Same user twice warns | PASS | The warning names the other account, including `default`. |
| B7 | Login restrictions apply to named accounts | PARTIAL (#570) | Named accounts are refused correctly, but the default login and environment keys still affect the outcome. |
| B8 | Rejected refresh or key names the recovery command | PASS | |
| B9 | Environment keys never used for named accounts | PASS | Against real OpenAI, the fake named account got 401 while the real key sat in `OPENAI_API_KEY` and `CODEX_API_KEY`. |
| B10 | Real named API-key account | PASS | `login --account real --with-api-key`, then gpt-5.4 answered `pong`. |
| B11 | TUI add, "Use for this session", `/cost` | PASS on OpenAI sessions; **FAIL** across providers (medium, #571) | Choose a named ChatGPT account from another provider's session, then switch with `/model`: the request carries no credentials, yet `/status` and `/cost` claim the account. Default-account switches and API-key accounts are fine. |
| Live | Real ChatGPT sign-in | NOT VERIFIABLE | No disposable ChatGPT account; your login was not used. |

**Common**

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| C1 | Flag off unchanged | PASS | Same credentials as the baseline; named commands refused. |
| C2 | Existing single-account home carries over | PASS | `auth.json`, config, vault and `~/.aws` unchanged; only a model-list cache changed. |
| C3 | Missing or wrong account never falls back | PASS | |
| C4 | Real providers | PASS | Named accounts on Z.AI, Kimi, Claude Plan and OpenAI answered `pong`; each fake one got 401. |
| C5 | GLM 5.3 Flash coordinator | PASS | Its `exec` workers were signed or refused correctly and it replied `pong`. An in-process spawn onto Bedrock was refused by the spawn tool, so that path is untested. |
| C6 | Leak scan | PASS | 17,831 files: 0 real keys. Fake secrets appear only in my own fake files, scripts and launch log, never in a Corbanu home. |
| C7 | `/cost` per account | PASS | Except the #571 misattribution. |

**Issues filed:** #570 (high), #571 (medium), #572 (low), and #573 (low): a named ChatGPT sign-in is refused or hidden when the configured provider isn't OpenAI. Neither medium-or-higher defect leaks a key or falls back to another credential.

**Recommendation for Travis:** accept slice A (AWS profiles) behind the flag now; it does what it promises, and its only gap is minor. Hold slice B (ChatGPT/OpenAI sign-ins) until #570 and #571 are fixed, because one can silently delete your default login and the other makes a ChatGPT account unusable while claiming it's in use. Live Bedrock, live ChatGPT and browser sign-in stay open, as the sprint record already says.

Scratch homes, both build targets and both worktrees have been removed.