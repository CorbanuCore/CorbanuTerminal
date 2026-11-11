## Corbanu API with no key: `/status` and first-run setup

- **`/status` no longer claims a missing Corbanu API key.** Under Moderate or
  Aggressive, with Corbanu Plan chosen but no key, `/status` said
  `Corbanu Plan API key configured` while `/providers` said not configured. It
  now says `Corbanu API not configured (set it up in /providers)` (#1563).
- **First-run setup for a chosen Corbanu Plan opens the provider list.** It used
  to open an OpenAI-worded key prompt that ended in
  `Provider API-key selection is no longer available`. The list now starts on
  **Corbanu API**, whose wallet and API-key setup runs after **Done**.
- **The same holds with `forced_login_method = "api"`.** With Corbanu Plan as
  the provider, setup now opens on the **Corbanu API** row (only that row and
  **Done**) instead of the OpenAI key prompt (#1584, #1571, thanks to
  @bradsbots).
