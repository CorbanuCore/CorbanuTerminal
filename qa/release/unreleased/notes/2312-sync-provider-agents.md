## Fixes

- Keep the current provider when resuming with current settings, and hide Vertex AI and Bedrock Runtime onboarding entries until their setup features are enabled.
- Require a fresh confirmation before a held Enter key can save a prefilled provider API key.
- Restore manager tools for all v2 agent models while honoring the native-tools off switch. A sub-agent reopened without its parent remains locked down and explains that the parent task must be opened first.
