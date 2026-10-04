# Corbanu Terminal 0.1.43

The repository operator instructed on 2026-09-12 that the local provider credential replacement hotfix be "fixed properly": promoted from the debug candidate `0.1.42-provider-replacement-94b042de5b` (which blocked `corbanu update` on production hosts) into a published release. Classification: bounded reliability repair; no trading behavior, authorization, or credential-storage boundary changes.

Release branch `release/corbanu-0.1.43`, based on public `main` at `406aa3c5f5` (267 commits past the hotfix base `30436889d5`). PR: https://github.com/CorbanuCore/CorbanuTerminal/pull/122.

Included changes:

- `dd8b901696` (cherry-pick of `94b042de5b`): Configured providers retain activation controls and expose their existing account/API-key replacement methods in the Providers menu; "Replace" verb for configured providers.
- `693173fcd1` (cherry-pick of `bdce8ada90`): production recovery record for the 2026-09-12 hotfix (`qa/reliability/2026-09-12-provider-replacement/`).
- `4da8d793f1`: six provider-replace menu snapshots refreshed for main's status copy ("Enabled · configured" plus the verification hint); the "Replace with …" items are unchanged.
- `f41847a6ff`: version 0.1.43 in `codex-rs/Cargo.toml` and lockfile.

Conflict resolution: the hotfix forced `ProviderConfigurationState::Configured` to `ClaudeAccountIntent::Replace` unconditionally. `main` already ships credential-scoped reauthentication (`0215dc704f`), where Configured with a known unauthorized source renews that source and otherwise starts Replace. `main`'s behaviour was kept in `provider_management.rs` and its test; it satisfies the hotfix goal (a saved invalid Claude token is reachable for correction) without dropping the scoped path.

Validation on this host:

- `cargo test -p codex-tui --lib -- configured_credentials_can_be_replaced provider_manager claude_intent`: 11 passed.
- `cargo fmt --check -p codex-tui`: clean.
- Snapshot diff limited to status copy; every `provider_replace_*` snapshot still lists "Replace with …".

Not claimed: cross-platform interactive acceptance, the competitor/model benchmark cycle, and a fresh keyboard-driven PTY run on the 0.1.43 binary. The hotfix's own PTY recovery evidence is retained under `qa/reliability/2026-09-12-provider-replacement/`.

Release run: workflow `corbanu-terminal-release` dispatched with `release_version=0.1.43`, `publish_release=true`, `make_latest=true`. Publication is confirmed only by the workflow result.
