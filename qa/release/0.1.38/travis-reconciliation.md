# Travis / repair reconciliation — 2026-09-05

## Scope and source identity

This is bounded product-initiative integration under PF-57-S03, not a published
release or a whole-product security certification. Product citation:
**Shipping MVP — LIVE** — “Encrypted `/vault`, masked entry, metadata-only
inspection, and operational credential use without placing raw values in chat.”
The repository development skill required an isolated serial sprint, history
preservation and evidence separation; test-tui requires actual interactive input.

| Input | Exact commit | Disposition |
| --- | --- | --- |
| Fork main | `6dd9ad646beb4a7407521439411f436f21ea4af1` | Already included by Travis's branch; no redundant main merge |
| Shared reviewed baseline | `f03e95f7a65609bb442764d6306682d5fe43f6bb` | Common ancestor |
| Our credential fixes | `41794c3ae7de689594b21837c18899945ee75cf5` | Preserved unchanged |
| Our repair evidence | `c9680a41e7940e20c8816201db37b32d001a1a6b` | Preserved unchanged |
| Travis provider UX | `f38dccd8bf39ebbb6fb87b67612a0cb6f2504cc3` | Included |
| Travis Ambient catalog | `07791288b6feeccfaee5a57c12452359cc666957` | Included |
| Combined source | `c37eb277d9f83ebcabe89e41cc81b9d3e92797a2` | History-preserving merge |

Worktree: `/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile`.
Branch: `integration/reconcile-release-0.1.38`. The canonical dirty checkout and
the completed repair worktree were not modified. Local upstream reference
`ba6cf9c69277caec51a4c12c5b7401a9920930e0` was reverified; no upstream Codex
integration is claimed.

The two overlapping source files merged cleanly. `wallet_menu.rs` retains
direct legacy-compatible Lock and full wrapped upgrade guidance while adding
the shared account recovery/token UI. The new alias fixtures in
`multi_provider_onboarding.rs` inherit Travis's native-keyring isolation from
`CommandSpec`; no extra per-fixture workaround was necessary. Scoped Clippy
removed one redundant `clone()` of the Copy visibility enum; no behavior changed.

## Behavior preserved

- Claude's guided `claude setup-token` workflow, masked local enrollment,
  explicit source selection, cancellation, failed-token retry and successful
  save returning to the provider manager. No real subscription token was used;
  synthetic enrollment cannot establish a provider token's real validity period.
- OpenAI typed failure/retry, device-code/browser guidance and source correlation.
- Ambient offers GLM 5.2 only for new model/pane selections. Retired Kimi 2.7
  metadata remains available for saved-session restoration. Other providers'
  catalog choices are unchanged.
- Wallet daemon compatibility preflight before sensitive operations, explicit
  one-time manual legacy restart, and direct Lock for legacy revocation.
- Durable scoped Task Node logout/relink and consistent Corbanu API environment
  aliases across runtime eligibility and read-only account/usage requests.

## Combined-tree qualification

All Rust commands use toolchain 1.95.0 through the repository `just` entry points,
with `CARGO_TARGET_DIR=/tmp/corbanu-astra-review-phGuUE/codex-rs/target`.

| Check | Result |
| --- | --- |
| Scoped `just fix` for provider-auth, keyring-store, models-manager, TUI and CLI | PASS; existing lint warnings remain; log `/tmp/corbanu-pf57s03-fix.log` |
| `just fmt` and staged/unstaged `git diff --check` | PASS |
| Installer contract; packaging helpers | PASS: 4 + 27 Python tests; installer/DMG shell syntax passes |
| Plan and sprint validator tests | PASS: 4 + 19 Python tests |
| Plan/sprint checks and portable-skill mirror | PASS; 25 mirrored files, 3 mirror tests |
| Combined domain/CLI/TUI suite | PASS: 625/625, retries disabled; run `8bfd831d-a368-4a1e-9885-613e3aef960a`, 45 test binaries, 7.773s execution after compilation; 5,129 unselected tests not claimed |
| True-TMUX provider/catalog/alias regression suite | PASS: 16/16, retries disabled; run `c55d855c-d5f9-492c-95e4-71a4438c9f66`, 191.976s; 5,738 unselected tests not claimed |
| Manual `just codex` | PASS: 0.1.38 startup, `/providers`, Claude method guidance, masked token entry, Escape retaining OpenAI current, clean `/exit` |

The combined test command selected all tests in `codex-wallet`,
`codex-wallet-daemon`, `codex-tasknode-session`, `codex-model-provider-info`,
`codex-provider-auth`, `codex-keyring-store` and `codex-models-manager`, plus
focused `codex-cli` Claude OAuth and `codex-tui` account/status, onboarding,
wallet, Claude/panes, masked-entry and model-picker tests. Full command:

```sh
CARGO_TARGET_DIR=/tmp/corbanu-astra-review-phGuUE/codex-rs/target just test \
  -p codex-wallet -p codex-wallet-daemon -p codex-tasknode-session \
  -p codex-model-provider-info -p codex-provider-auth -p codex-keyring-store \
  -p codex-models-manager -p codex-tui -p codex-cli --locked --offline --retries 0 \
  -E 'package(codex-wallet) | package(codex-wallet-daemon) | package(codex-tasknode-session) | package(codex-model-provider-info) | package(codex-provider-auth) | package(codex-keyring-store) | package(codex-models-manager) | test(provider_account) | test(provider_status_host) | test(provider_auth_presentation) | test(onboarding::auth) | test(chatwidget::wallet_) | test(chatwidget::provider_manager) | test(chatwidget::claude_) | test(claude_panes) | test(onboarding::provider_setup) | test(ambient_glm_only) | test(model_selection_popup) | test(claude_oauth) | test(vault_secret_entry)'
```

Output: `/tmp/corbanu-pf57s03-tests.log`. No snapshot-update flag was used.

The true-TMUX run reused the same package selection with `--test-threads 2`,
`--retries 0` and the filter below. It required TMUX, enabled native-keyring
fixture isolation, and selected the fresh `codex` executable through
`CARGO_BIN_EXE_corbanu`. Incoming provider fixtures use the ignored debug-path
symlink to that same executable.

```text
test(suite::multi_provider_onboarding::tmux_corbanu_env_) | test(suite::provider_management::tmux_pf5) | test(suite::provider_management::tmux_ambient_model_picker) | test(suite::multi_provider_onboarding::tmux_configure_many) | test(support::tmux::tests::)
```

All eight real application journeys passed in one run: configure-many with
restart/request, three alias/restart/account-read cases (including legacy
daemon failure/direct Lock/retry), Ambient's GLM-only picker, API-key recovery,
OpenAI cancel/retry and Claude rejected-token retry/successful save. All eight
harness isolation/cleanup/artifact tests passed too. Native credentials and
live inference/payment services were not used.

Log: `/tmp/corbanu-pf57s03-tmux.log`. Success artifacts were moved out of the
source tree to `/tmp/corbanu-pf57s03-evidence-AH6CdM/target/tmux-artifacts/`;
the separate harness artifact root is `/tmp/corbanu-pf57s03-tmux-artifacts`.
All success-bundle binary hashes match the recorded TMUX executable below.

### Interactive build and manual evidence

`just codex` built successfully from the combined source. The wallet daemon was
rebuilt with `cargo build -p codex-wallet-daemon --bins --locked --offline` from
`codex-rs/`; log `/tmp/corbanu-pf57s03-wallet-build.log`. The ignored local
`codex-rs/target/debug` symlink points to the shared cache for incoming fixtures
that require that path. All tested programs are local Linux debug builds,
not release installers.

| Binary | SHA-256 |
| --- | --- |
| Manual `just codex` process, hashed through its running `/proc` executable | `b739e72f8bdac5d44260a926f8960c269e9b21766c6f69b0b8bda9b3430d9b2e` |
| `codex` rebuilt by the selected test-package graph, used by TMUX fixtures | `29725a54e298175567b05a2e7a065e776191041bec864512e467d5b114211389` |
| `corbanu` from the selected test-package graph | `4a30d60220cc38763bae8f45060fea8bd37366584f2d59185496b1a538715970` |
| `pfterminal-walletd` | `3b6f956832415fa627e2dcc30f57d89fc31fc268caf137c174240cab4cb5be0f` |

Manual home/evidence: `/tmp/corbanu-pf57s03-tui-awm0jR`. Run with private
`CODEX_HOME`, `CORBANU_HOME` and `PFTERMINAL_HOME`,
private `CLAUDE_CONFIG_DIR`, native-keyring fixture isolation, synthetic OpenAI
file auth and loopback `OPENAI_BASE_URL`. `RUST_LOG=trace` and explicit
`-c log_dir="/tmp/corbanu-pf57s03-tui-awm0jR/logs"` followed test-tui.
No inference prompt or real subscription token was submitted.

Keys: `/providers` text then separate Enter; Down/Enter for Claude Account;
Enter for setup; inspect long-lived method; Enter for masked token entry;
Escape to the provider list; Escape to chat; `/exit` text then separate Enter.
The private TMUX server exited normally. Captures:

- `claude-method.txt`: `8c68daad986b1e4f01b83df4ef6757daea82b29050a9171fe0f814d028fbafb7`
- `claude-token-entry.txt`: `443e4fb1982f7e1916190d66349b8093a392561b4ac27330478e6e67c60a6327`
- `claude-cancel.txt`: `0d72641b1e10fbe14789b7295a058096342560c28be9843aef74b0627f25b40d`

The first manual fixture incorrectly tried to override the reserved built-in
`openai` provider. Startup correctly rejected it. The fixture was corrected to
use `OPENAI_BASE_URL` and the successful run above; no product code was changed
to bypass that validation.

### Independent Astra check

The user's explicit Astra review request was applied to the final combined
source while the primary agent ran qualification. Astra reported **no actionable
new P0/P1 findings** in the bounded merge-specific review of `c37eb277d9`.
It checked wallet/login overlap, submission sequencing, aliases/eligibility,
Task Node unlink, retired Ambient restoration and debug keyring isolation.
This was read-only source review, not independent test execution or an
exhaustive security audit. No further external model review was commissioned.

## Release and risk boundaries

- No tag or GitHub release was created. Publication was presented as a separate
  user choice; the development target remains 0.1.38.
- Final fetch reconfirmed both `origin/main` and Travis's integration head at
  the input commits above. The candidate is intended as a new remote branch;
  neither shared `main` nor Travis's source branch is rewritten.
- Incoming GitHub CI run `33929290764` passed at `07791288b6`; this is not
  final combined-tree evidence. Historical macOS/native-release results in the
  incoming records are likewise not reruns on this merge.
- TensorCash and Isometric Game are not exercised by this bounded home-level
  auth/catalog reconciliation: no project editing or coding-task behavior was
  changed. Their prior PF-56 passes remain historical. A release-level
  requalification, cross-platform builds, named-human acceptance and the due
  benchmark bootstrap are not claimed here.
- The forwarded report that Travis's Claude login works is user-supplied
  experience on the incoming branch, not a named-human acceptance run of this
  exact merged build and not a one-year-duration test.
- The [incoming privacy observation](provider-ux-parity-audit.md#additional-pre-existing-privacy-observation)
  remains unresolved: debug/trace key-event logging can expose individual secret
  characters. Do not enter real credentials with those logs enabled. Fixture
  full-string canary checks do not establish character-level logging safety.
- The incoming audit also records an unpublished-version recall heuristic that
  can offer 0.1.37 to a 0.1.38 candidate; skip that prompt when testing this
  unpublished build. No updater repair was part of this reconciliation.
- The separate site/API-key review findings remain unchanged; see
  [Astra repair and site evidence](astra-fixes.md). No production key creation,
  payment, signing, credential deletion or site deployment occurred here.
