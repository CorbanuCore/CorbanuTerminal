## `corbanu login status`

- **No part of an API key is shown.** With an API-key login it used to print
  the key's first 8 and last 5 characters. It now prints
  `Logged in using an API key - API key configured`, plus
  `(fingerprint <12 hex>)` when the home's vault has its fingerprint salt. That
  is the same per-home fingerprint a named OpenAI account holding the key
  shows (REG-02).
