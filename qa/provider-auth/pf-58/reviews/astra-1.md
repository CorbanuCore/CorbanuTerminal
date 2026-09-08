autoreview findings: 3
[P2] Preserve external credential ownership for OpenAI API-key recovery
codex-rs/tui/src/chatwidget/provider_recovery.rs:63
When OpenAI uses CODEX_API_KEY, discovery reports OpenAiAuthMetadata::ApiKey and resolves it as ManagedByCorbanu. After a 401, this branch consequently offers to replace a saved key instead of showing environment instructions. Saving reports success and clears credential health, but AuthManager::reload still prioritizes CODEX_API_KEY, so the next request uses the same rejected credential. Preserve the effective environment source when selecting recovery and avoid writing managed credentials for this case.

[P2] Permit validation after externally managed credentials are renewed
codex-rs/tui/src/chatwidget/provider_recovery.rs:73
For a command-auth provider that receives a typed 401, the new health overlay sets RecoveryRequired. These instructions tell the user to renew externally and retry, but renewal leaves the metadata identical and never calls credential_changed. ChatWidget::submit_op then blocks every subsequent request through current_requires_recovery, so the renewed credential cannot be tried without restarting. Provide an explicit retry/revalidation path that preserves the selected source and retains the failure until recovery is validated.

[P2] Include the narrow-terminal snapshot asserted by the new test
codex-rs/tui/src/chatwidget/provider_health_tests.rs:84
The test asserts openai_reauthentication_narrow, but the change bundle and snapshot directory contain only openai_reauthentication.snap. With normal snapshot-update settings, this assertion fails because its expected snapshot is missing, including in the new qualification script's provider_health test selection. Add the reviewed narrow snapshot before landing.

overall: patch is incorrect (0.99)
External-source recovery can either replace the wrong storage source or prevent retry after renewal. The new snapshot test also lacks its required baseline. Review used read-only inspection; no tests were run.
