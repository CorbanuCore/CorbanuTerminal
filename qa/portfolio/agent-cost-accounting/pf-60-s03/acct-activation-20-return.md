# RETURN — acct-activation-20

Allocation digest: b8fa1c277f95843a32f2019791240b16ef496f7e4595d635ade610666f48ec27.
Claim: 40a27351-cbac-4620-934c-8dcae94258f2.
Worker: gpt-6-astra / high.
Verified frozen brief SHA-256: def4b01bd757dada700efb4b015d09820059168fafe4d58e049a8ef469ecd3c7.
Base: e45b9d77d2bbd1eed35913753c7c73ad3bf970d8.
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/acct-activation-20260916.
Branch: bootstrap/acct-activation-20260916.

## Authorization and scope

Product initiative, PF-60-S03 (in_progress), active plan `docs/plans/active/portfolio-agent-cost-accounting.md`.
Product-spec heading: **Product measurement**; excerpt: “No commercial performance numbers have been supplied.”
The sprint records Travis's September 16 developer-only activation decision.
This frozen action explicitly supplies the activation worktree, base and literal scope.
The plan/sprint coordinates still name the older inspector worktree; manager reconciliation is outstanding, and those records are outside this worker's write scope.
Sprint checker passes: 115 current, 127 archived.

## Blockers found before editing

1. `AccountingMode` defaults to `Disabled`; production config loading hard-codes that value.
   The four source test modules named in the brief are not the complete set: integration-test support/recovery files also assign accounting modes.
2. A read-only inspector requires an installed ledger, but the claim that core never calls installation is false.
   The real public install entry point is `codex_state::accounting::AccountingStore::open` in `state/src/runtime/accounting_store.rs`.
   It calls private `install_on_connection`, runs `accounting_migrator().run_direct` when wholly absent, validates, and performs retention maintenance.
   Existing `Sampling::start_request` already calls `AccountingStore::open` before admission.
   Disabled collection prevents reaching that call. A second installer would duplicate existing behavior.

## Implementation

Non-default Cargo feature: `codex-core/developer-accounting`.
The feature compiles `developer_accounting_mode` and its config-loader call site.
The selector binds existing direct Anthropic and built-in OpenAI Responses/Chat collector modes to their provider endpoints.
Existing collector eligibility/authentication/endpoint checks remain responsible for actual sampling; unsupported provider IDs remain disabled.
Installation stays lazy, on the first eligible request through the existing collector.
No state code or dependency was changed. No new accounting TOML key, environment variable or slash command was added.
`AccountingMode::default()` remains `Disabled`; default-feature config loading still sets it to `Disabled`.
The doc comment was deliberately updated to describe build-only activation accurately; it is **not unchanged**.
The existing inspector and its “Unavailable — accounting ledger not installed. Collection remains off.” message are untouched.
No user shipment, release, human acceptance or independent functional qualification is claimed. Nothing was pushed.

## Mechanical compile-time proof

The copied default native core library contains **0** selector-symbol matches; the copied feature-enabled library contains **4** (definitions/references, including its closure).
Both contain **14** existing collector-control symbol matches.
Default library SHA-256: `dbc16934d49996b3704b9fda425374425e4ecb83f7adb6a3cdee758c2576c002`.
Enabled library SHA-256: `17aa86eb34acdffd9146cff6ab3553ae5f807eac1e37bd0b3657033deef54119`.
Cargo fingerprint feature sets were respectively `[]` and `["developer-accounting"]`; both declared the new feature.
See [default scan](acct-activation-20-default-symbols.json), [enabled scan](acct-activation-20-enabled-symbols.json), [artifact origins](acct-activation-20-artifact-origins.json) and [reproducible scanner](acct-activation-20-symbols.py).
Local binary artifacts are preserved under the ignored `acct-activation-20-artifacts/` directory.

This proves discrimination for those unstripped native library artifacts, not every packaged release, optimization level or target.
It does not catch an accidental release with the feature enabled/unified (including `--all-features`), differently named equivalent code, or code whose symbols were stripped/inlined.
Existing `AccountingMode` variants, collector code and explicit store APIs remain compiled in default builds; the newly introduced selector and loader call are excluded.

## Validation

| Run | Result |
| --- | --- |
| `just test -p codex-core accounting` | Exit 100; 124 run, 54 passed (2 flaky, 3 leaky), 70 failed, 3529 skipped |
| `just test -p codex-tui usage` | Exit 0; 91 run, 91 passed, 4071 skipped |
| First feature accounting attempt | Exit 101; zero tests executed; E0433: new fixture referenced undeclared `reqwest` |
| Feature accounting replay | Exit 100; 126 run, 54 passed (2 flaky, 2 leaky), 34 failed, 38 timed out, 3529 skipped |
| Final activation-only replay | Exit 0; 3 run, 3 passed, 2470 skipped; all three added cases passed |
| Feature CLI build | Exit 0; `cargo build --locked --offline -p codex-cli --bin codex --features codex-core/developer-accounting` |
| PTY attempt 01 | Exit 1: populated inspector rendered, but harness incorrectly required the below-viewport exact subtotal without scrolling |
| PTY replay 02 | Exit 0: one sampled Responses request, populated overview, and exact subtotal/input rows reached with real Down keys |

Exact default failure names and flaky/leaky cases: [machine-readable result](acct-activation-20-core-default-result.json).
Exact feature failure and timeout names: [machine-readable result](acct-activation-20-core-enabled-result.json).
Raw attempts are retained in the correspondingly named `.txt` files; no failure is attributed to the base.
The feature replay includes the new HTTP fixture failing because its initial SSE contained usage but no model output. Output events were added, and the three activation cases are replayed separately on the final source; the broad failing run is not relabeled as a final-tree pass.
The compilation error was fixed with the existing `HttpClientBuilder`/transport API, without adding dependencies.
The fixture was also corrected because built-in Anthropic does not accept a TOML endpoint override.
The sampler fixture binds loopback through the developer selector; the PTY fixture uses the existing OpenAI `openai_base_url` setting.
Only changed Rust files were formatted with `rustfmt --edition 2024 --config skip_children=true`; no workspace formatter/fixer ran.
A concurrent untracked `tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_auto_review_permissions.snap.new` appeared outside scope.
That test is absent from this worker's usage-gate log. The file was neither edited nor removed.

## End-to-end evidence and line counts

Candidate: Corbanu Terminal v0.1.42, assigned base plus the four-file working diff.
Built CLI SHA-256: `9c9dec0a44b6b33787f9fb548005ea4775dc8cdf3b4745127eedfedd2c77ac76`.
The copied executable contains 4 selector-symbol matches and 12 collector-control matches: [CLI scan](acct-activation-20-cli-symbols.json).
The PTY replay's independently recorded executable hash matches this value.

The [PTY driver](acct-activation-20-pty.py) starts an empty HOME/profile, sets the existing debug native-keyring denial, and uses only synthetic API-key data against a loopback Responses server.
It sends prompt text and Enter separately, receives “Activation fixture complete.”, opens `/usage requests` with separate Enter, and sends ten Down keys to reveal detail.
[Server receipt](acct-activation-20-pty-run-02/requests.json): exactly one POST to `/v1/responses`, model `gpt-5.6-sol`.
This is a real request through the CLI's native sampler to a synthetic provider, **not live paid inference** and not manually inserted ledger rows.

Actual rendered excerpts from the [overview](acct-activation-20-pty-run-02/overview.txt) and [scrolled inspector](acct-activation-20-pty-run-02/inspector.txt):

```text
Recorded requests — root and descendants
Requested UTC day: 2026-09-16
Estimated token cost for recorded attempts: $0.000125
Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.
Billed cost: unavailable — no settlement evidence
Snapshot is not current; newer activity is unverified
Known subtotal exact USD: 0.000125
Input: 7 known + unknown in 0 attempts
Noncached input (derived for inclusive input): 7 known + unknown in 0 attempts
```

The replay reports a 2047 ms maintenance lag; no freshness warning was suppressed.
Both PTY attempts retain raw terminal bytes, screen captures, keys, request receipts and executable hashes in their separate directories.
Attempt 01's failure was the harness's viewport assumption, not an Absent result; it is preserved rather than overwritten.
The copied main executable lacked the `codex-code-mode-host` sidecar and visibly warned that Code Mode was unavailable. This smoke exercised no tool execution.
No OS-enforced code-blind executor, negative probes, independent evidence review, cancel/recovery/resume qualification, live-repository suite or named-human acceptance is claimed.
Those gates remain with the manager; this return is not an unqualified human-test handoff.

Source budget: **46 changed production/config lines outside tests** (45 additions, 1 deletion); **177 Rust test lines added**, retaining all existing tests.
QA test programs add **185 lines** (158-line PTY driver and 27-line symbol scanner), for **362 added test-program lines** including Rust.
Documentation, raw logs, terminal captures and artifact metadata are evidence, reported separately in [change counts](acct-activation-20-change-counts.json); they are not hidden in the 46-line production figure.
No state source, state manifest, protected state files, or TUI source was edited.
All owned changes are confined to the frozen scope. The unrelated generated snapshot noted above remains untouched.
Nothing was committed or pushed.
