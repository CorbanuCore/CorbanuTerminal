# Picker repair source identity

Base HEAD: `e8e9b92ae012045358d7ae0916ed092869bb80ad`.
Branch: `feat/provider-reauth-health`. This is a dirty-tree candidate including
the preserved September 8 runtime and September 10 Keychain repairs, not a
claim that HEAD alone identifies its source.

SHA-256 of `git diff --binary -- codex-rs` after final formatting:
`861d51815e4d80d4e8d4049e27ed3b2e506d3bbac684995b145a9f1d7a8930d3`.

Additional untracked Rust/snapshot source files (relative to `codex-rs/`):

| File | SHA-256 |
| --- | --- |
| keyring-store/src/macos_interaction.rs | bc9656571df078ad15eacffdb6679cf8d9d32eb9c51d39e33360be506cd0a465 |
| keyring-store/src/macos_interaction_tests.rs | 26885e7e2919f2d187cdf474d4a0d107ae6ccd8dd7e6fa3274abb79bdc52da73 |
| keyring-store/src/prompt_budget.rs | 0b753ca658531fdf11bc2326e3b416662400d94e58175abec34bd1c829f8519a |
| keyring-store/src/prompt_budget_tests.rs | 3324442b6f49fd56d0d13a476a81fefd9fbd351c825bd22bb9386fae35e6d4b6 |
| tui/src/app/tests/provider_picker_refresh.rs | 9cf229a1c2e88435e0cca6c33ec26368ecda207b6d4882a5434ae59ff4ed09be |
| tui/src/chatwidget/tests/snapshots/codex_tui__chatwidget__tests__popups_and_settings__model_picker_exact_provider_current.snap | 0a876b0ba0312ab724e738faed4fab692ccb70fdfea48b43c747b06737711fbb |
| tui/src/chatwidget/tests/snapshots/codex_tui__chatwidget__tests__popups_and_settings__model_picker_recovered_claude_subscription.snap | acb2ac94b4ddd51e8fbf7116b5ef6c81f42f4c2dfddce00f112176849e18af47 |

The RTX source mirror is `/home/travis/security-round5/picker-repair-20260910`.
Local native builds use the allocated Mac worktree. Final package hashes and
test outcomes are recorded in the parent picker repair document and result JSON.
