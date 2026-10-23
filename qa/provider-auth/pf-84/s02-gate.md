# PF-84-S02 gate evidence (named account registry and resolution)

Candidate: branch `feat/pf-84-s02-account-registry`, code commit `dcbe51a94e`, base
`051f9747225776a5d85ad00c2e5d0a8f5f4036bf` (merged with main `44b527f11f`). Behind the
default-off `named_accounts` feature. macOS arm64 debug build; every run used disposable
homes and `CORBANU_TEST_NO_NATIVE_KEYRING=1`.

## What the slice delivers

- Names `[a-z0-9][a-z0-9-]{0,31}` (`default` reserved); labels
  `provider/<provider id>/accounts/<name>/<kind>` in the existing vault; the label set is the
  encrypted registry. No keychain item per account. Today's credentials are the `default` account.
- Kinds: `api_key` (env-key providers, resolved inside the credential broker or the non-broker
  path; never from environment variables), `claude_oauth_token` / `claude_config_dir`
  (`internal-claude-oauth-token --account`), `command` (`auth.command` providers receive
  `CORBANU_PROVIDER_ACCOUNT`; an unenrolled account never runs the command).
- `[provider_accounts]` config selector (so `-c provider_accounts.<id>=<name>` works);
  `corbanu account list|add|remove` (secrets from piped stdin only; refused under Aggressive).
- Fail closed: unknown, missing or invalid accounts error with recovery text; OpenAI sign-in
  and AWS providers are refused at config load (moved to PF-84-S04); a model correction to a
  sibling route keeps the account only when both routes use the same key.

## Checks

| Check | Result |
| --- | --- |
| `just test -p codex-vault -p codex-login -p codex-network-proxy -p codex-model-provider-info -p codex-model-provider -p codex-features -p codex-config -p codex-cli` | 2556 passed |
| `just test -p codex-core -E 'test(/provider_accounts\|model_broker_auth\|schema\|multi_agents/)'` | 159 passed |
| same core filter with `--features codex-core/developer-accounting` | 35 passed |
| Isolation canaries | vault, login and broker tests: per-account canaries never cross; a missing account never yields `default` or another account; named labels never surface as default key ids |
| Migration | a single-account home resolves identically and its `local.age` bytes are unchanged until the first named write |
| Linux clippy `-D warnings` + `cargo check --workspace --tests` (RTX) | `CLIPPY_EXIT 0`, `CLIPPY_FEAT_EXIT 0` (developer-accounting) and `cargo check --workspace --tests` `EXIT 0` at `7e635b1aa`; rerun at `dcbe51a94e`: see the PR |
| tmux + GLM 5.3 Flash (Z.AI default key via the vault helper) | `pf84-account-isolation`: the GLM agent lists accounts and runs `corbanu exec` children: Z.AI account `main` (real key) replies `pong`, account `fake` gets `401 Unauthorized` |
| Second provider (Kimi) | `pf84-account-isolation-kimi`: same flow on `kimi-code` with `provider/kimi_api_key` (`pong`, then 401 for `fake`) |
| Videos | [Z.AI](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s02-pf84-account-isolation-dcbe51a94e33-2026-10-10.mp4), [Kimi](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s02-pf84-account-isolation-kimi-dcbe51a94e33-2026-10-10.mp4); leak scan passed; disposable homes holding real keys were deleted after recording |

## Independent review (Opus 5.5 High, read-only `corbanu exec`)

Three passes (initial, follow-up, scoped confirmation). Fixed: Linux cfg on `account_cmd`;
schema regenerated; unsupported providers fail closed; `auth.command` accounts gated by the
registry inside the account-bound manager (so no path, including realtime, runs the command or
falls back to session auth); named subscription labels refused by generic reveal, `vault
auth-helper` and the scoped broker; typed reads and typed registry listing; vault errors logged;
metadata-only checks; sibling-route correction rules; secrets only from piped stdin; errors do
not echo input; the unshipped feature doc removed. Accepted with reasons:
- Out-of-process `corbanu account` edits are seen by a running session after restart (the S04
  UI will bump the in-process revision).
- `internal-claude-oauth-token` honours an inherited `CORBANU_PROVIDER_ACCOUNT`; every parent
  sets or removes it for provider commands.
- The TUI worker preflight (`tui/src/spawn_orchestration.rs`) still validates the default
  account's command; it never passes the unenrolled name (S03 owns spawn).
- A vault error on the API-key path is logged and surfaces as "no stored API key".

## Open

- Independent code-blind functional design and execution: acceptance step, not the implementer.
