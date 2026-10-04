# Corbanu Terminal 0.1.46

Fixes several provider and agent-loop problems found in benchmarks against
Hermes.

- **Runaway tool-call loops are stopped.** A model that repeats the same call
  over and over (seen with GLM 5.3 Flash on Vercel, up to 250 times) now has
  repeats refused after three in a row, and the turn stops after eight.
- **GPT-6 Sol works over OpenRouter.** Its code-execution calls were rejected
  before; it now passes the benchmark tasks.
- **No more runaway whitespace output.** One malformed tool call could make a
  model emit tens of thousands of blank tokens; the tool format no longer allows
  it.
- **GLM 5.3 and GLM 5.3 Flash work on Vercel accounts that require zero data
  retention.**
- **Fewer wasted Vercel requests.** A rejected server-side continuation is no
  longer retried every turn, and Corbanu no longer adds a fake "Continue."
  message the model mistook for the user.

Known limitation: a model that cycles through several calls without making
progress is not yet stopped; a fix is in progress. Carries forward the 0.1.44
limitation that cancelling a long streamed answer can leave later requests
stalled; start a fresh session if this happens.

Restart Corbanu after updating. Released under explicit operator authorization.
Full cross-platform qualification and the competitive benchmark cycle remain
incomplete. [Release record](https://github.com/CorbanuCore/CorbanuTerminal/blob/rust-v0.1.46/qa/release/0.1.46/RELEASE.md).
