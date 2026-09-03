# Authentication and Vault

## The pain

Provider credentials become unsafe when users must paste secrets into chat or
cannot tell account authentication from stored API keys. Corbanu Terminal keeps
credential entry masked, storage encrypted, and metadata inspectable without
revealing raw values.

For the user-facing vault workflow, start with
[`/vault` and credentials](features/vault.md). This page covers the deeper
account, storage, migration, and logout behavior.

## Product contract

| Field                      | Value                                                                                                                                                                                        |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Status                     | **LIVE**                                                                                                                                                                                     |
| Exact product-spec heading | **Shipping MVP — LIVE**                                                                                                                                                                      |
| Vault excerpt              | “Vault and credentials: Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.”                                      |
| Provider excerpt           | “Multi-provider inference: OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers.” |

## Credential surfaces

Corbanu Terminal has three credential surfaces:

1. account-backed provider login for OpenAI Codex Account and Claude Plan;
2. provider API keys for Anthropic, Ambient, Kimi Code, Z.AI, DeepSeek,
   OpenRouter, Meta, Baseten, Vercel, and compatible providers; and
3. the encrypted `/vault` credential store for provider keys and other
   user-managed secrets.

`/providers` is the common front door. OpenAI uses device authentication;
Claude Plan uses Claude Code's native subscription login; supported API-key
entries use masked entry and are written to the vault. See
[`/providers`, account login, and `/model`](features/model-providers.md) for
the complete user workflow.

## Account-backed providers

### OpenAI Codex Account

Use `/providers` and select:

```text
Provider: OpenAI Codex Account
```

Corbanu Terminal starts a device-code login, shows the verification URL and one-time
code, and stores the resulting Codex/OpenAI account auth in the configured
Corbanu Terminal home.

Installed `corbanu` launchers and the source-built `corbanu` binary use the
same deterministic state resolution as the installer. To override the product
home explicitly, set:

```bash
export CORBANU_HOME="$HOME/.corbanu"
```

`PFTERMINAL_HOME` remains supported for existing automation, and an explicit
`CODEX_HOME` remains authoritative when neither product-specific override is
set. With no override, Corbanu Terminal prefers an existing `$HOME/.corbanu`,
reuses a lone `$HOME/.pfterminal` in place, and otherwise creates
`$HOME/.corbanu`. This keeps account auth, vault data, sessions, and logs
separate from a stock Codex install using `$HOME/.codex`.

### Claude Plan

On first run, choose **Provider: Anthropic Claude Account** to enter this flow
directly; that option is the default when provider-account onboarding is
allowed, and Corbanu persists `claude-plan` only after authentication succeeds.
For an existing installation, use `/providers` and select:

```text
Provider: Claude Code Plan
```

Corbanu Terminal presents an explicit choice. The recommended option runs
`claude setup-token`, then stores the approximately one-year subscription token
through masked entry in the encrypted vault. The compatibility option selects
Claude Code's normal rotating platform login. A successful choice persists;
Corbanu never silently falls back to another source, account, or billing path.
Claude Code must be installed, and neither option uses an Anthropic API key.

OpenAI and Claude account state have separate owners and lifecycles. Signing in
or out of one does not authenticate or erase the other.

See [Reliable Claude Plan authentication](features/claude-plan-authentication.md)
for eligibility, platform-store precedence, replacement, and recovery.

## Provider Keys

Built-in providers use these key names:

| Provider   | Provider id  | Key name             | Vault label                   |
| ---------- | ------------ | -------------------- | ----------------------------- |
| Anthropic  | `anthropic`  | `ANTHROPIC_API_KEY`  | `provider/anthropic_api_key`  |
| Ambient    | `ambient`    | `AMBIENT_API_KEY`    | `provider/ambient_api_key`    |
| Kimi Code  | `kimi-code`  | `KIMI_API_KEY`       | `provider/kimi_api_key`       |
| Z.AI       | `zai`        | `ZAI_API_KEY`        | `provider/zai_api_key`        |
| DeepSeek   | `deepseek`   | `DEEPSEEK_API_KEY`   | `provider/deepseek_api_key`   |
| OpenRouter | `openrouter` | `OPENROUTER_API_KEY` | `provider/openrouter_api_key` |
| Meta       | `meta`       | `MODEL_API_KEY`      | `provider/model_api_key`      |
| Baseten    | `baseten`    | `BASETEN_API_KEY`    | `provider/baseten_api_key`    |
| Vercel     | `vercel`     | `AI_GATEWAY_API_KEY` | `provider/ai_gateway_api_key` |

Provider key resolution uses one exact provider-key identity across onboarding,
`/providers`, startup, requests, restarts, and resumed sessions. A present
environment variable is authoritative for that process, including an invalid
value that requires recovery; when it is absent, Corbanu checks encrypted
managed storage and then the legacy migration source. Legacy
`provider_auth.json` remains readable for migration compatibility, and a
successful vault write removes the migrated plaintext key when possible.

Newly configured providers become active by default. Deactivation changes only
eligibility and preserves the credential in its existing owner. Reactivation
therefore does not copy, reveal, or rewrite the key. Metadata and status can
cross the UI event boundary; raw credential values cannot.

Environment variables are still supported for temporary shells and automation:

```bash
export ANTHROPIC_API_KEY="..."
export AMBIENT_API_KEY="..."
export KIMI_API_KEY="..."
export ZAI_API_KEY="..."
export DEEPSEEK_API_KEY="..."
export OPENROUTER_API_KEY="..."
export MODEL_API_KEY="..."
export BASETEN_API_KEY="..."
export AI_GATEWAY_API_KEY="..."
```

For normal interactive use, store keys through onboarding or `/vault` so they
are encrypted at rest.

## Vault Storage

The vault is backed by the Codex managed-secrets store:

- encrypted file: `$CODEX_HOME/secrets/local.age`;
- passphrase storage: OS keyring when available;
- fallback: local `0600` keyring fallback file only for the vault passphrase on
  keyring-less hosts;
- metadata: labels, types, providers, and timestamps are listable without
  revealing raw secrets.

The vault is global to the Corbanu Terminal home directory, so stored credentials are
available from any working directory that uses the same `CODEX_HOME`.

## Using `/vault`

Open the vault action menu:

```text
/vault
```

Useful commands:

```text
/vault list
/vault show provider/zai_api_key
/vault credential add
/vault credential delete provider/openrouter_api_key
```

`/vault credential add` opens a masked entry view. Do not type raw secrets as
chat text. The secure entry path keeps secrets out of prompt history, transcript
history, and model context.

`/vault show <label>` displays metadata only. Raw reveal/export is intentionally
handled through secure UI, not chat output.

## Login And Logout Commands

Corbanu Terminal includes inherited Codex login commands:

```bash
corbanu login
corbanu login --with-api-key
corbanu login status
corbanu logout
corbanu logout --all
```

`corbanu logout` removes Codex/OpenAI account auth and preserves provider
API keys in the vault. Use `corbanu logout --all` only when you also want to
remove provider API keys from the vault and legacy provider auth storage.
Neither command owns Claude Code's separate subscription account state.

For API-key providers, use the onboarding picker, `/providers`, `/vault`, or
the provider environment variables above. Claude Plan remains an account route.
Corbanu API is wallet-linked and stores its generated API credential through the
same encrypted provider-key boundary; `/providers` reports both routes separately.
