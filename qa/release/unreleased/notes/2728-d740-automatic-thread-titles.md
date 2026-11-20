## Upstream Codex rust-v0.167.0

- Naming a new conversation automatically from its first message costs one
  extra model request, so Corbanu now does it only on plan or subscription
  accounts (a ChatGPT plan, Claude Plan or a Corbanu Plan). It is off on
  pay-per-token API keys, wallet-funded providers and local models. Set
  `automatic_thread_titles = true` in the `[tui]` table of `config.toml` to
  turn it on everywhere, or `false` to turn it off.
