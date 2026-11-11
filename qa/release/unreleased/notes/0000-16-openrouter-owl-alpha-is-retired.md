## OpenRouter Owl Alpha is retired

- **`openrouter/owl-alpha` no longer works** (#128, #1486). OpenRouter lists no
  endpoints for it (checked 2026-10-28), so a request to it is expected to fail
  at OpenRouter. A config that saved this ID still resolves to its own hidden
  entry and is never switched to another model; its description now says it is
  retired. Pick a current model with `/model`. The earlier "explicit legacy IDs
  remain compatible" note no longer covers this ID. The other retained IDs
  (Grok 4.6 and 4.5, DeepSeek V4 Pro and V4 Flash 0731 on OpenRouter, and
  GPT-5.5 on OpenAI) are still served and unchanged.
