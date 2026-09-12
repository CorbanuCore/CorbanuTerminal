# Provider/model picker regression repair — September 10

User-authorized PF-58 follow-up in `feat/provider-reauth-health`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health`, on HEAD
`e8e9b92ae012045358d7ae0916ed092869bb80ad` plus preserved in-progress repairs.
Bounded picker repair within the active provider-auth initiative; the plan and
PF-58-S01 remain active/in_progress. Product heading: **Shipping MVP — LIVE**,
“OpenAI, Anthropic/Claude Plan ... and custom providers” and “operational
credential use without placing raw values in chat.”

## Defects and corrections

- Opening the manager seeded the current model into every configured runtime,
  including unrelated local/custom providers. Curated provider models now stay
  on the exact current route; other built-ins use their own defaults. Idle local
  servers are not seeded. The established shared opaque-model flow for explicitly
  configured custom gateways and discovered catalog routes is preserved.
- Current-row markers compared only the model slug. They now compare the
  provider/model pair; the session's provider also determines the initial tab.
- Asynchronous manager metadata settled after the model picker cached provider
  health. The settled snapshot now updates both surfaces without another
  credential-store read, preserving generation and in-flight-auth guards.
  A late result also no longer reopens a dismissed manager over the model picker;
  a deterministic app-level test covers stale and valid completion after Escape.
- A missing Anthropic API credential remains separate from Claude subscription
  authentication. Management still exposes setup; the model picker offers the
  API route only when eligible. No credential/billing fallback is introduced.

## Evidence and limitations

- Focused provider-auth regression suite: 72/72 passed on RTX.
- Focused final TUI catalog/provider/status suite: 58/58 passed; two new snapshots
  reviewed. Older test fixtures with mismatched model/provider pairs were
  corrected rather than weakening the exact-identity contract.
- The expanded package harness fails on the previous signed Mac package:
  `baseline-mac-2/result.json` and final pane capture show the missing Claude
  Plan tab after setup. The initial `baseline-mac/` run instead found a harness
  variable initialization error, corrected before the A/B run.
  `baseline-mac-visible` also fails with viewport-only checkpoints: a late
  manager refresh intercepts the attempt to open `/model` after setup.
  Final same-driver A/B in `baseline-mac-settled` waits for completed setup and
  reproduces the invented `Other` provider; the new Mac package passes.
- Replacement Linux package passed the expanded actual-key TMUX harness:
  Claude setup, all three subscription models, Fable 5.1 selection, repeated
  manager opening, same-home restart, and exactly one current model. Anthropic
  remains in management; its independent masked API-key setup then enables all
  three API models without changing the Claude session. Actual host/shell/MCP
  calls, three processes, Ctrl+C cancel/recovery, and narrow token guidance pass.
  Final artifacts: `evidence/picker-20260910/rtx/tmux-linux-settled/`.
- Final Linux candidate: `/home/travis/security-round5/evidence/picker-20260910/candidate-final/bin/codex`;
  SHA-256 `5ecd62f90574e1b6208625220abf93e431d347c1f2a91c9111c3512e81448062`.
  Source mirror: `/home/travis/security-round5/picker-repair-20260910`.
- Full final provider-management/convergence/Claude selection: **32/32 passed**
  on the pinned Linux package (30 true-TMUX journeys and two supporting tests).
  The earlier broad run's custom-model failure is corrected and passes on the
  final package. The final signed native Mac package also passes the expanded
  actual-key harness: `evidence/picker-20260910/tmux-mac-settled/result.json`.
  This record is not human sign-off.
- The expanded harness initially left the manager before credential persistence
  had completed. It now requires current-viewport checkpoints and the actual
  Claude/Anthropic configured rows before opening `/model`; old scrollback is never
  treated as current UI. These driver failures are retained on RTX, not counted
  as final product passes or hidden by retries. The final run passes first try
  with those positive completion assertions.
- Native Mac candidate: `macos-candidate-picker-final-20260910/bin/corbanu`
  under the external PF-58 working directory; CLI SHA-256
  `8275923c4ee9c0bfc7e52109f742564c5a429631d765e15b6f0643b4cb7d3669`.
  All four signed helpers/CLI passed strict signature verification. The four
  stable Applications-launcher links now resolve to this tested package.
  [Package manifest](evidence/picker-20260910/macos-package.md).
- Existing-profile Mac startup and provider/model menus passed twice without
  password input (3.11s and 1.68s). Targeted metadata logs show seven successful
  Keychain loads per process and no failures; configuration is byte-for-byte
  unchanged. This is not a live model request or named-human UI sign-off.
- Handoff-gate Python regressions: 10/10 passed. Final lint/build/unit/TMUX logs
  and success captures are copied under `evidence/picker-20260910/rtx/`; every
  copied Rust success capture records the final CLI hash above.
- The prior human Keychain test asked twice; metadata logs show successful reads but
  do not identify the two dialogs. Preserve the same Developer ID identifiers;
  do not conflate initial per-item permission with a proven no-loop experience.
- Escape cancellation remains a separate reproduced baseline failure. Ctrl+C
  evidence does not qualify that check. Full live-account/live-repository,
  human, benchmark and release qualification are not claimed.
- The sprint checker still reports unrelated PF-43/44/45 duplicate IDs and
  security-plan links/order errors; PF-58's line-length issue was corrected.
  An unscoped Clippy run also encountered an existing core `expect_used` lint;
  final scoped `cargo clippy --locked --no-deps -p codex-tui -p codex-provider-auth --lib`
  passes (existing warnings retained); two avoidable `expect()` calls in the
  affected provider-status paths were removed. No unrelated source was changed.

Final Rust source identity is recorded in
[`evidence/picker-20260910/source-identity.md`](evidence/picker-20260910/source-identity.md).

## Repeatable regression coverage

- `model_catalog_tests`: repeated manager synchronization does not invent
  Claude copies for idle local/custom runtimes; legitimate shared opaque custom
  models remain available.
- `chatwidget/tests/popups_and_settings`: exact provider/model current marker;
  settled Claude metadata restores the three-model subscription catalog; an
  independently configured Anthropic API key enables its separate catalog.
- `app/tests/provider_picker_refresh`: stale generations are inert; a valid late
  refresh updates eligibility without reopening a dismissed manager.
- `provider-auth/status_tests`: settled snapshots update only registered exact
  identities, without adding a provider or reading its credential.
- `tui/tests/suite/claude_auth`: actual-key setup/cancel/failure/recovery/resume
  includes model-catalog assertions after setup and restart.
- `handoff_tmux.py`: run against the staged package, not an unbundled executable;
  the first successful run must include the new catalog/restart result fields.
- `live_picker_startup.py`: opt-in, bounded two-launch check with an explicit
  existing profile. It sends no model request, enters no credential, leaves
  configuration unchanged, and never completes a Keychain dialog. Targeted
  keyring metadata logging is not global tracing. Human acceptance stays false.

On the RTX mirror, the focused TUI invocation is:

```sh
INSTA_WORKSPACE_ROOT=/home/travis/security-round5/picker-repair-20260910/codex-rs \
  just test -p codex-tui --lib -E \
  'test(model_catalog) | test(model_picker) | test(provider_manager) | test(provider_management) | test(provider_model_policy) | test(provider_health) | test(provider_status_host)' \
  --retries 0 --test-threads 4
```

Set the existing RTX Cargo target/tool PATH and synthetic keyring isolation as
in the handoff runner. Run the 32-test integration selection with
`--test all -E 'test(claude_auth) | test(provider_management) | test(provider_convergence)'`,
`CORBANU_TMUX_REQUIRED=1`, and all three `CARGO_BIN_EXE_*` aliases pinned to the
same final package. Snapshot workspace override is necessary because the shared
target's cached workspace path otherwise points at an older checkout.

Earlier PF-58 review cap is retained; no additional independent review, main
merge, release or push is claimed by this follow-up. Tests use synthetic
credentials except any explicitly recorded metadata-only real-profile startup
check. No Keychain ACL changes, secret exports, or credential deletions.
