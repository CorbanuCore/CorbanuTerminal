# Named accounts per provider

Status: behind the default-off `named_accounts` feature (PF-84, in development).

Product linkage: **Shipping MVP — LIVE**, "Encrypted `/vault`, masked entry,
metadata-only inspection, and operational credential use without placing raw
values in chat."

## The pain

One API key or subscription per provider is not enough when you want a second
Z.AI or Kimi key, a second Claude Plan login, or a worker that should bill a
different account than its coordinator. The old workaround, one Corbanu home per
account, splits your vault, sessions and usage.

## What works today

Your existing credentials are the `default` account of each provider; nothing
about them changes. A named account lives in the same encrypted vault and adds no
keychain item.

```sh
corbanu --enable named_accounts account add zai work            # key on stdin
corbanu --enable named_accounts account add claude-plan alt --kind claude-token
corbanu --enable named_accounts account list                    # names and kinds only
corbanu --enable named_accounts account remove zai work
```

Choose the account a provider uses in `config.toml`, or for one run with `-c`:

```toml
[features]
named_accounts = true

[provider_accounts]
zai = "work"
```

```sh
corbanu exec -c provider_accounts.zai=work "..."
```

- A named account uses only its own credential. It never reads the provider's
  environment variable and never falls back to `default` or another account; a
  missing or wrong credential fails with a message that names the account.
- Kinds: `api-key` (providers with an API key), `claude-token` and
  `claude-config-dir` (Claude Plan: a `claude setup-token` token, or a Claude
  Code login in its own `CLAUDE_CONFIG_DIR`), `aws-profile` (Bedrock) and
  `command` (custom `auth.command` providers, which receive the name in
  `CORBANU_PROVIDER_ACCOUNT`).
- Secrets are read from stdin only. Claude tokens are refused by generic
  `/vault` reveal and `vault auth-helper`; named API keys behave like today's
  provider keys.
- Adding or removing accounts is refused under the Aggressive security level.
- `corbanu logout --all` removes named accounts with the other provider keys.
