# Corbanu Terminal 0.1.44

Updated models and restored GPT-6 access through existing ChatGPT sign-in.

- Fixes OpenAI compatibility negotiation: an outdated client-version header could reject GPT-6 Sol and Luna even when the same login worked in official Codex.
- Adds Claude Opus 5.5 Plan with exact model routing, separate from Anthropic API billing.
- Adds GLM 5.3 Flash to Z.AI Coding Plan and fixes persistence of its supported reasoning efforts.
- Adds OpenRouter Grok 4.7, DeepSeek V4.1 Flash, GLM 5.3 Flash, and `nvidia/nemotron-3-ultra-550b-a55b:free`.
- Removes legacy OpenRouter DeepSeek V4 Pro/Flash, Grok 4.5/4.6, and Owl Alpha from normal selection and automatic allocation. Explicit legacy IDs remain compatible.
- Hides GPT-5.5 from the selector while preserving explicit-ID compatibility; includes GPT-6 Luna.
- Preserves the provider-credential replacement fix shipped in 0.1.43. No account or billing fallback is introduced.

**Known limitation:** cancelling a long streamed answer can leave subsequent requests stalled. Start a fresh session if this happens. Normal Sol chat, tool use, and restart passed local checks; cancellation recovery did not.

FlashX is not included: Z.AI currently excludes it from Coding Plan, and a separate pay-as-you-go route was not authorized.

Restart Corbanu after updating. The operator explicitly authorized this release with the disclosed limitations. Competitive/model benchmarks and full cross-platform interactive qualification remain incomplete; no benchmark pass is claimed. [Release record](https://github.com/CorbanuCore/CorbanuTerminal/blob/rust-v0.1.44/qa/release/0.1.44/RELEASE.md).
