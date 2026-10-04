# Corbanu Terminal 0.1.45

Fixes prompt caching for Claude and other explicitly cached models over
OpenRouter and Vercel.

- Cache markers now follow the newest turn in agent loops. Before, every tool
  turn re-billed the whole conversation at the full input price when Claude,
  MiniMax, or some Qwen models were used through the `openrouter` or `vercel`
  providers. In an Opus 5.5 test on OpenRouter, a debugging task dropped from
  $0.53 to $0.30 and cached input rose from 38% to 79%.
- Claude Plan, direct Anthropic, and GLM, Kimi, and DeepSeek V4 routes were not
  affected.

Carries forward 0.1.44 unchanged otherwise, including its known limitation:
cancelling a long streamed answer can leave subsequent requests stalled; start a
fresh session if this happens.

Restart Corbanu after updating. Released under explicit operator authorization.
Full cross-platform qualification and the competitive benchmark cycle remain
incomplete. [Release record](https://github.com/CorbanuCore/CorbanuTerminal/blob/rust-v0.1.45/qa/release/0.1.45/RELEASE.md).
