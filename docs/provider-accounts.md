# Named provider accounts (`named_accounts`, off by default)

A provider can hold several accounts besides `default`, each a named key or
token in the vault (`corbanu account add <provider> <name>`, `corbanu account
list`). Turn the feature on with `--enable named_accounts` or
`[features] named_accounts = true`.

## Choosing an account

Precedence: `--account` (or a spawn's `account`) > the account a resumed thread
recorded > `[provider_accounts] <provider> = "<name>"` > `default`.

- `--account <name>` selects an account of the session's provider.
- `--account <provider>:<name>` for **another** provider selects that
  provider's account for whatever in the session runs on it: spawned agents
  and TUI workers given that provider, and a TUI switch to it. The session
  itself stays on its own provider's selection. `corbanu exec` cannot switch
  providers, so it refuses such an account when no spawned agent can run on
  that provider (spawned agents off, or `agents.provider_allowlist` excludes
  it).
- `--account default` always means today's credentials, also with the feature
  off.

## Fail closed

A missing account is an error, never the default account. Every selected
account, including another provider's, is checked before the session starts:

```
account `ghost` of provider `zai` is not configured; add it with
`corbanu account add zai ghost` or pick another with `--account`
(see `corbanu account list`)
```

`corbanu exec` and resume fail at thread start; the TUI exits with this text
before any sign-in screen. A resumed thread runs on its recorded account; if
the feature is off, the resume is refused and says the recorded account needs
it (`--account default` resumes on the default credentials). A rejected key
names the account and its `corbanu account add` recovery; the default
credential is unchanged.
