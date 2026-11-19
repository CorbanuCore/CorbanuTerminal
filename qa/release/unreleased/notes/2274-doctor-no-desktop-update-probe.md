## Fixes

- `corbanu doctor` no longer checks for OpenAI desktop-app updates. Those rows contacted `persistent.oaistatic.com` and `chatgpt.com` whenever the OpenAI desktop app was installed (D620). The desktop app is still reported when present.
