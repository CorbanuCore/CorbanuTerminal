# 2. Verify startup identity and remembered setup

Wait for the composer, then run /status. Note the current provider and model. Do not configure anything yet.

Pass: startup reaches chat without forcing /providers, your previously selected provider/model is remembered, Code Mode is available, no codex_apps startup warning appears, and Corbanu does not offer an “update” to the older 0.1.37 release.

Fail: provider onboarding appears despite a valid prior setup, the provider silently changes, Code Mode host is missing, MCP startup is interrupted, an update-to-0.1.37 message appears, or the UI stalls.

