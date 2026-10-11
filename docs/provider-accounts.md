# Named provider accounts (`named_accounts`, off by default)

One key per provider is not enough when you keep a work and a personal account,
or want a worker on a different account than its coordinator. Named accounts
let a provider hold several accounts besides `default`, each a named key or
token in the vault (`corbanu account add <provider> <name>`, `corbanu account
list`). Turn the feature on with `--enable named_accounts` or
`[features] named_accounts = true`.

## Choosing an account

For a provider, the first of these wins: an explicit `--account` for that
provider (or a spawn's `account`), the account a resumed thread recorded (or a
worker's parent runs on), `[provider_accounts] <provider> = "<name>"`,
`default`.

- `--account <name>` selects an account of the provider you launch with. On
  `resume`, write `<provider>:<name>` if the thread runs on another provider.
- `--account <provider>:<name>` for **another** provider selects that
  provider's account for whatever in the session runs on it: spawned agents
  and TUI workers given that provider, and a TUI switch to it. The session's
  own provider keeps its own selection. `corbanu exec` cannot switch
  providers, so it refuses such an account when no spawned agent can run on
  that provider (`agents.enabled = false`, or `agents.provider_allowlist`
  excludes it).
- `--account default` always means today's credentials, also with the feature
  off.

## Fail closed

A missing account is an error, never the default account. The session
provider's account and the account `--account` names (of any provider) are
checked before the session starts; another provider's `[provider_accounts]`
entry is checked when something starts on that provider:

```
account `ghost` of provider `zai` is not configured; add it with
`corbanu account add zai ghost` or pick another with `--account`
(see `corbanu account list`)
```

`corbanu exec` and resume fail at thread start. The TUI exits with this text
before any sign-in screen, also when resuming (choose the account with
`--account`); with a remote app server, the server refuses at thread start.
A resumed thread runs on its recorded account; with the feature off, the
resume is refused and names the recorded account (`--account default`
resumes on the default credentials). A rejected key names the account and
its `corbanu account add` recovery; the default credential is unchanged.
