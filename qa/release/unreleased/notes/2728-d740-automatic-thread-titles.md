## Upstream Codex rust-v0.167.0

- Naming a new conversation automatically from its first message costs one
  extra model request, so Corbanu now does it only on plan or subscription
  accounts (a ChatGPT plan, Claude Plan, Kimi Code or the Z.AI coding plan).
  It is off on pay-per-token API keys, wallet-funded providers (including the
  Corbanu Plan balance), local models and a ChatGPT personal access token. Set
  `automatic_thread_titles = true` in the `[tui]` table of `config.toml` to
  turn it on everywhere, or `false` to turn it off.
