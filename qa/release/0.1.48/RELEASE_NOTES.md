# Corbanu Terminal 0.1.48

- **Busy providers no longer end your turn.** When OpenRouter reports that a
  model's shared upstream pool is briefly rate-limited and asks callers to retry,
  Corbanu now waits and retries (up to four times, honoring Retry-After) instead
  of stopping the task on the first 429. Your own quota and usage limits still
  stop immediately.
- **MiMo V2.6 Pro on OpenRouter.** `xiaomi/mimo-v2.6-pro` is now in `/model`.
- **Better GLM 5.3 Flash results.** GLM 5.3 Flash on Z.AI, OpenRouter and Vercel
  gets a short work pattern: batch independent calls, write complete
  implementations, and verify by running code before finishing.

Carries forward the 0.1.44 limitation that cancelling a long streamed answer can
leave later requests stalled; start a fresh session if this happens.

Released under explicit operator authorization. Full cross-platform
qualification and the competitive benchmark cycle remain incomplete.
[Release record](https://github.com/CorbanuCore/CorbanuTerminal/blob/rust-v0.1.48/qa/release/0.1.48/RELEASE.md).
