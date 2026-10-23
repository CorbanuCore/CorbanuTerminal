# PF-84-S06 slice B gate: named ChatGPT/OpenAI sign-in accounts

Sprint: [PF-84-S06](../../../docs/sprints/archive/unified-provider-auth/pf-84-s06-aws-and-chatgpt-accounts.md),
slice B (D4 option (a); Travis 2026-10-11: the default login stays in place, the same ChatGPT user under two accounts
is warned, not refused). Behind the default-off `named_accounts` flag. Slice A is [s06a-gate.md](s06a-gate.md) (#543).

## What changed

- **Gate lifted for OpenAI sign-in.** `supports_named_accounts` accepts OpenAI-auth providers without an env key,
  command or AWS config; core stamping limits named sign-ins to the built-in `openai` provider.
- **B1 (storage).** A named account holds its whole sign-in (`auth.json` contents: ChatGPT tokens or an OpenAI API
  key) in the vault under `provider/openai/accounts/<name>/chatgpt_auth` (`VaultAccountAuthStorage`). No keychain
  item, never `auth.json`. Parse errors never quote the value.
- **B2 (refresh).** One `AuthManager` per (home, account) per process (`for_named_account`, a weak registry), so one
  refresh lock per account. It loads, refreshes, saves and logs out only that account's store; it never reads the
  default login, the ephemeral/external store, `OPENAI_API_KEY`, `CODEX_API_KEY` or `CODEX_ACCESS_TOKEN`. Other
  processes' refreshes are picked up by the guarded reload. A rejected refresh says `sign in again with corbanu login
  --account <name>`. A session on a named account uses that manager for the whole session; a provider on the default
  account is handed back the default manager.
- **B3 (sign-in).** `corbanu login --account <name>` (browser, `--device-auth`, `--with-api-key`), `corbanu login
  --account <name> status`, `corbanu logout --account <name>` and `/providers` *Add another account* (device code or
  masked OpenAI API key). A re-login keeps the old sign-in until the new one is saved, then revokes only the replaced
  tokens (not when it is the same ChatGPT user). A TUI add never overwrites an existing account; a refused add revokes
  the tokens it could not save. Logout or remove revokes only that account's tokens and deletes only its store, also
  when the stored sign-in is unreadable.
- **B4 (default).** The default login is untouched: read in place, never copied, moved, refreshed or logged out by a
  named account (`auth.json` bytes compared before and after in every run).
- **B5 (display).** Rows, `account list`, `login status` and `/status` show only the kind, an email hint
  (`w…@example.com`), the plan and a fingerprint. The fingerprint hashes the sign-in's identity (workspace and ChatGPT
  user, or the API key), so a refresh keeps it. Signing the same ChatGPT user in under a second account prints a
  warning naming the other account(s), the default included.
- **Restrictions.** `forced_login_method` and `forced_chatgpt_workspace_id` apply: a sign-in of the other kind or
  another workspace is refused at session start (sessions and spawned agents) and never loads; the TUI hides the
  ruled-out add method.
- A rejected named OpenAI account (`exec`, TUI health) names `corbanu login --account <name>`; a Bedrock account
  points at its AWS profile.

## Tests

Final head `4faac3c1ef` (slice A and `origin/main` merged in).

Linux on glitch (`-j10`, the isolated `just test` runner, `CORBANU_TEST_NO_NATIVE_KEYRING=1`):

| Run | Result |
| --- | --- |
| `cargo clippy --tests -D warnings` on model-provider-info, exec, vault, aws-auth, login, model-provider, core, cli, tui | clean, plain and `developer-accounting` |
| `-p codex-vault -p codex-aws-auth -p codex-model-provider -p codex-login -p codex-model-provider-info` | 471 passed |
| `-p codex-cli --test provider_accounts` | 4 passed |
| `-p codex-core --lib -E 'test(provider_accounts) \| test(account)'` | 121 passed; 126 with `developer-accounting` |
| `-p codex-tui --lib` (named accounts, provider manager, status, startup provider) | 440 passed, plain and `developer-accounting` |

macOS (arm64): `cargo clippy --tests -D warnings` on the same crates clean; debug build used for the video and the
GLM run.

New tests: vault storage per account (no `auth.json`, delete, unreadable value never quoted); summaries (email hint,
plan) and the same-user warning (default and other accounts); create-only adds; login restrictions (kind, workspace)
at the manager and before start; concurrent refresh of two accounts writing back to their own stores with the
default `auth.json` byte-identical; another process's refresh picked up by the guarded reload; a rejected refresh
naming `corbanu login --account play`; logout revoking only `work` (mock revoke) and missing accounts never falling
back; environment keys ignored by a named account; a device-code sign-in landing in the vault only, a second add
refused and its tokens revoked; replaced sign-ins revoked only for another user; named and default OpenAI providers
never mixing (including a default provider handed a session's named manager); the stable sign-in fingerprint; CLI
`login --account --with-api-key`, `account list`, `login status`, `logout --account`; TUI add methods, API-key and
ChatGPT rows; rejection messages per kind.

## Functional check (tmux, disposable homes)

Script: [slice_b_check.sh](s06/scripts/slice_b_check.sh) with the mock issuer and Responses endpoint
[mock_openai.py](s06/scripts/mock_openai.py) (fake tokens; access tokens expire after 60 s; refresh tokens rotate and
a reused one is rejected; `bad` users get expired tokens), Linux debug build of `4faac3c1ef`. The default login is an
API key in `auth.json`; a decoy `OPENAI_API_KEY` is set throughout. Captures: [s06/slice-b/](s06/slice-b/).

- `login --account work|play|twin|bad --device-auth` signed each account in; `twin` (same ChatGPT user as `work`)
  printed the warning; `account list` shows kind, email hint and plan only ([cli.txt](s06/slice-b/cli.txt)).
- Two `exec` workers on `work` and `play` at the same time each refreshed their own tokens (the mock accepted every
  refresh, so no rotated token was reused) and were answered as their own user; a second run needed no refresh; the
  default was answered as `api key default`; `auth.json` kept the same SHA-256 throughout
  ([mock-events.jsonl](s06/slice-b/mock-events.jsonl)).
- `bad`: the refresh was rejected; the worker failed with `Sign it in again with corbanu login --account bad` and the
  default was unaffected. `nope` was refused at thread start. `logout --account play` revoked only `play`.
- A home with no default login: a named account alone works, and the default still uses the environment key as today.
- TUI: `/status` shows `· account work` and `Account: ChatGPT sign-in · w…@example.com · Plus (account work)`; a turn
  was answered as `work`; `/providers` rows; *Add another account* → *Sign in with ChatGPT (device code)* saved `tui`
  ([tui-6](s06/slice-b/tui-6-signed-in.txt)).
- Leak scan of captures, logs and rollouts for refresh tokens, fake keys and the fake JWT header: 0 files.

## GLM run, video, review

- GLM 5.3 Flash coordinator (`zai`, real default key from the installed vault helper, never printed), macOS debug
  build: it ran a `corbanu exec` worker on `--account fake` (OpenAI, a fake API-key sign-in). OpenAI answered 401; the
  worker named the account and `corbanu login --account fake`, did not fall back, and the coordinator replied `pong`
  ([glm-run.txt](s06/slice-b/glm/glm-run.txt), [script](s06/scripts/glm_run_b.sh)). Real-key scan: 0 files.
- Video `pf84-chatgpt-account` ([mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-chatgpt-account-4faac3c1ef78-2026-10-11.mp4),
  [index](../../demos/index/PF-84-S06.md)): `/providers` lists `OpenAI · work` as a ChatGPT sign-in with email hint,
  plan and fingerprint; *Use for this session*; the turn is answered as `work`; `/status` names the account and its
  sign-in.
- Independent review (Opus 5.5 High, read-only, installed `corbanu exec`): round 1 REQUEST CHANGES (login
  restrictions bypassed; missing tests; re-login destroying the old sign-in; corrupt sign-ins not removable; account
  lending on a provider switch; `/status` identity; lazy load; overwrite on add), round 2 APPROVE WITH NITS, round 3
  APPROVE WITH NITS; the round-3 nits (revoke tests against the mock, same-user guard on an unsaved sign-in, API-key
  hint, recording the known limits) are addressed in `4faac3c1ef` and this file.

## Not verified

- **Live ChatGPT: pending.** No disposable ChatGPT account is available; Travis's own login was not used (brief).
  Only the mock issuer and fake tokens were exercised. A live check needs a second, disposable ChatGPT account signed
  in by device code in a disposable home.
- The browser (local callback server) sign-in into a named account was not exercised end to end; device code and API
  key were.

## Known limits (record before the flag is removed)

- ~~A session started on a named account keeps that account for MCP connectors, plugin install and accounting even
  after a mid-session switch to a default-account OpenAI provider.~~ Fixed in the PF-84 follow-up (below): they
  follow every mid-session switch, named to default, default to named and between named accounts.
- ~~The model list is fetched with the account the process started on.~~ Fixed in the same follow-up: it follows the
  account the user's session uses. ~~Still open: `/status` rate limits before the first reply come from the default
  login, and the app server's `/apps` directory listing (`app/list`) always uses the default login.~~ Fixed by #658:
  the app server's own ChatGPT backend requests (also plugin listings, settings and analytics) use the session's
  account.
- The registry keeps each named manager's first base settings (`forced_login_method`, workspace, route) until it is
  dropped; a config reload does not update a live manager (the session-start check does run again).
- `for_named_account` reads the vault while holding the registry lock; the `/status` card reads the vault on the UI
  thread.
- A same-user re-login leaves the replaced refresh token unrevoked (by design, to keep the new session alive).
- Not done: zeroizing every in-memory copy of an API key; a keychain-mode test of the default login; the legacy
  pfterminal default store in the same-user check; a narrow race between a refresh save and a concurrent remove.

## Follow-up: mid-session account switch (fix/pf84-switch-follow)

- **What follows the switch.** Every path that changes the session's provider (`update_settings`, and a turn whose
  settings change it) moves the session's sign-in (`SessionServices::active_auth`) to that provider's named account or
  the default login, under the state lock and before the model client is rebuilt. Connectors (MCP, Apps auth included)
  are marked for reconnection; the prewarm worker re-subscribes to the new manager's refreshes, also when the old
  manager is gone. Plugin installs read the active sign-in per use. Accounting prices each request under its own
  provider's account (`active_auth.for_provider`), turn-level reads use the turn's manager, so a switch mid-turn does
  not reprice the running turn. A session starting on a provider without a named account uses the default login, also
  when spawned from a session on a named account.
- **Model list.** When the process starts on an OpenAI sign-in provider, its list (`model/list`) is an
  `AccountModelsManager` that follows the account of a user's session (not sub-agents) at session start and on a
  switch; each named account has its own cache (`models-cache/<provider>/<name>/`). A switch to an account whose
  cache is fresh (the check refreshing uses: this client version, under 5 minutes) shows it at once and refetches in
  the background; otherwise the switch waits for the fetch (at most 5 s; API-key sign-ins fetch nothing). If the fetch
  takes longer, the TUI shows the bundled list until its next refetch (a later switch or restart); core has the
  fetched list once it arrives. Only responses from that provider (same base URL) and account refresh it. With two
  user sessions switching at once, one may check the other's list (last one wins, as above). The TUI sends an account-changing model selection with `confirm` (falling back to an
  unconfirmed update if confirmation is unavailable); only once core confirms the switch does it fetch `model/list`
  again and say *Connectors, usage and the model list now use …*.
- **Limits.** The process has one list: with several user sessions on different accounts (an app-server client with
  many threads) the last one to start or switch decides it. A process started on a provider without OpenAI sign-in
  (e.g. `zai`) keeps that provider's list. Analytics events follow the session's account (#658). A turn running
  across a switch is priced under its own provider's account but recorded under the session's current provider id
  (`accounting_binding`, unchanged). The switch reads a named account's ChatGPT or API-key sign-in from the vault under
  the session state lock (other kinds load on first use); accounting resolves the request's account per request, which
  reads the vault only if nothing holds that account's manager. Runtime model entries added for other providers keep
  their metadata across a list refresh, as before.
- **Tests (fail before, pass after).** Core `session::account_switch_tests`: connectors, the prewarm worker across
  default → work → default → play, a turn that changes the provider, accounting per request provider, the model list,
  no reconnect when the account is unchanged; `account_models_manager::tests`. TUI: catalog replacement and which
  switches refresh the list.
- **tmux check** (disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`; `work` signed in with the `openai-api-key`
  vault label, GLM 5.3 Flash on the default Z.AI key, both fetched inline, never printed). Evidence:
  [s06/switch-follow/](s06/switch-follow/).
  - Run 1 started on `openai · work`, so the process list follows: `/model` → GLM → back to OpenAI.
  - Run 2 started on GLM: `/providers` *Use for this session* `work` → `/model` OpenAI (`work-ok`) → GLM
    (`default-ok`). It shows connectors and accounting following and the TUI refetching; its process list is Z.AI's.
  - Runs 3 and 4 (review and final heads) start on `openai · work` and switch GLM → OpenAI → GLM.
  - Core logged each switch (`default` / `openai:work`) before the TUI's `model/list`. The ledger attributes each
    attempt to the account that served it. Leak scan for both real keys: 0 files.
- **Video:** [pf84-switch-follow](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-switch-follow-40cbb034ec7e-2026-10-12.mp4)
  (final head, leak-scanned): `/providers` *Use for this session* `work`, `/model` OpenAI answered via `work`, back to
  GLM answered via the default account, each switch announced.
- **Seen, not caused here (#578):** after GLM → OpenAI in the same session, OpenAI rejected the turn with 404 for a
  GLM reasoning item id (`rs_…`, `store=false`) carried in the history; the request was authenticated as `work`.
