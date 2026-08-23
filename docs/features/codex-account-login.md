# OpenAI Codex account login

## The pain

Account login and provider API keys have different lifecycles, but a careless
logout flow can erase both. Corbanu Terminal keeps OpenAI Codex account auth
separate from vault-backed provider credentials.

## Product contract

> **Product specification — “Shipping MVP — LIVE”**
>
> “Multi-provider inference: OpenAI, Anthropic/Claude Plan, Kimi, Z.AI,
> DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama,
> LM Studio, Corbanu Plan, and custom providers.”

## Log in

1. Open `/providers`.
2. Select **OpenAI Codex Account**.
3. Start device-code login.
4. Complete the browser verification.
5. Return to Corbanu Terminal and select an OpenAI model with `/model`.

First-run onboarding exposes the same account option, so initial setup and later
credential maintenance use one flow.

## Credential separation

OpenAI account authentication is stored through the inherited Codex account
manager. Provider API keys remain in Corbanu Terminal's encrypted vault.

Normal logout removes OpenAI account authentication without deleting vault
credentials:

```bash
corbanu logout
```

Deleting all provider credentials is intentionally explicit and destructive:

```bash
corbanu logout --all
```

Review the confirmation carefully before using `--all`.

## Main implementation

- `codex-rs/tui/src/chatwidget/provider_credentials.rs`
- `codex-rs/login/src/auth/manager.rs`
- `codex-rs/cli/src/login.rs`
