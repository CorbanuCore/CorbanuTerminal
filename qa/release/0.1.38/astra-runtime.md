# Astra native runtime qualification — September 5, 2026

Status: native runtime repair, final-tree regression checks, both live TUI runs,
and the local human-test installation passed. This is not a release authorization.

## Failure and repair

The human's actual TUI request failed with HTTP 400: Astra required a newer
Codex. PF-55-S02 proved selector/persistence and synthetic requests, **not live
inference**. That evidence did not justify calling Astra ready.

Classification: product initiative, unified-provider-auth PF-55-S03. Product
specification: **Shipping MVP — LIVE**, “Multi-provider inference”; **Product
principles**, “Maintain continuous Codex parity without removing Corbanu-specific
behavior.” Worktree: `/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile`,
branch `integration/reconcile-release-0.1.38`, allocated base
`de26f9f3ccff5748b12633b995ada52570a9e161`.

The compatibility boundary failed in two places: discovery and inference both
advertised `0.144.1`, while the bundled Astra entry came from public API metadata,
not the native Codex harness. The repair shares one `0.153.0` compatibility value
between discovery and inference and installs the native Astra runtime metadata:
Responses Lite, Code Mode Only, unified execution, model instructions, verbosity,
image detail, and native context limits (272,000 default / 872,000 maximum).
Corbanu's product version and user agent remain truthful and unchanged.

The generic personality override now preserves complete model instructions when
a newer remote catalog supplies only an instruction template and an empty legacy
base. A personality toggle must not erase the model's operating instructions.

## Upstream integration disposition

Audited baseline `ba6cf9c69277caec51a4c12c5b7401a9920930e0` against official
[`rust-v0.153.4`](https://github.com/openai/codex/tree/rust-v0.153.4), peeled commit
`3d2ee51ca2d5db578f328aa75e20aa22c0197c9a`. The official
[native catalog](https://github.com/openai/codex/blob/rust-v0.153.4/codex-rs/models-manager/models.json)
declares Astra's exact minimum client version as `0.153.0`. The public
[API reference](https://developers.openai.com/api/docs/models/gpt-6-astra)
describes a different context-window configuration and is not a substitute for
that native harness contract.

- Adapted the native instruction template, with Corbanu branding. P0-size manual
  context review completed: approximately 21 KB / 5K tokens, bounded fixed
  source, below the 10K-token fragment limit; authorization and secret-handling
  guidance retained. It is the existing base-instruction fragment, not a new
  repeated history injection.
- Kept Sol as the application default and Astra's existing manual-selection
  reasoning choices/medium default and disabled automatic allocation economics.
  Did not introduce Ultra orchestration, new billing defaults, or Fast defaults.
- Existing Corbanu Code Mode host, Responses Lite serializer, native auth,
  streaming, cancellation, and persisted history already implement the required
  operating path. Real tool and resume checks validate this, not only headers.
- Did not import the entire 1,517-file upstream delta. Optional cyber access
  programs, guardian endpoints, expanded usage telemetry, persistent reasoning,
  and model-specific Ultra orchestration are separate features, not needed for
  this manual Astra path. No authorization boundary was weakened to gain access.
- Retained existing prefix item identity behavior; upstream deterministic prefix
  UUIDs are a separate caching improvement. Live multi-turn and same-thread
  restart/resume work without that change.
- Did not activate upstream Node REPL or token-budget/history-notes guidance.
  This fork uses its existing Code Mode host and does not register Node REPL;
  the upstream token-budget object explicitly disables that optional feature.
  Native approval defaults are retained; credentials are resolved natively.

## Repeatable live harness

`scripts/astra_tui_acceptance.py` requires explicit `--allow-live`, a built
candidate, an existing authenticated profile, and a disposable linked git
worktree. It creates its own private tmux server; it never controls another
session or reads/copies tokens. Text and Enter are separate PTY writes.

It asserts structured provider response identity/model/provider, matching
tool-call/output IDs, execution results, independent unchanged fixture tests,
Escape abort, subsequent recovery, process exit/restart, and same-thread resume
with new execution evidence. Provider errors, reroutes, unmatched response
turns, missing outputs, and prompt echoes cannot count as success. Code Mode
exec outputs are parsed as typed JSON blocks; no regex is used.

Live runs use `RUST_LOG=warn` and a private explicit log directory. This is a
deliberate security exception to test-tui's trace guidance: the existing profile
is real, and trace has a known secret-keystroke logging risk. No credentials are
entered during qualification. Synthetic isolated TUI checks retain trace.
Final-candidate runs execute the exact hashed binary instead of rebuilding via
`just codex`, so qualification and the installed human-test binary can match.

Example (paths are explicit operator inputs, not repository defaults):

```sh
python3 scripts/astra_tui_acceptance.py --allow-live \
  --binary /path/to/candidate/bin/corbanu --home /path/to/existing/profile \
  --worktree /path/to/disposable/worktree --evidence /path/to/new/private/evidence \
  --expected-sha256 CANDIDATE_SHA256
python3 -m unittest discover -s scripts -p test_astra_tui_acceptance.py
```

First diagnostic: `/tmp/corbanu-astra-live-sj8Gtg/tensorcash-diagnostic-02/`.
Binary SHA-256 `749de8682e2a0628ff3bb1c3bebf9bb859a385aa4938382d4d26e594c07d687f`.
Thread `01a06f79-9773-7e93-90fc-3f2fe47a230d`: nine real Astra responses, six
paired tool calls, three successful turns and one deliberate cancellation;
file edit/tests, recovery, restart and resume all passed. The earlier diagnostic
stopped because nested exec UI events are not persisted in Code Mode; the
harness now verifies the actual structured tool outputs instead. Final-tree
runs below supersede these preliminary results.

## Final-tree evidence

Runtime commit: `6df2f2e2ed545506057e9e1aa7a76b9375aaea73`.
Final executable/harness source: `f848954d7739da8eaa4962f0866c612dedbaf5bb`.
Subsequent commits change only documentation/evidence. Final `just fix` and
`just fmt` preceded all qualifying checks; existing warnings remain warnings.

- Rust: **445/445 passed**, nextest `c1d74cf3-7837-489a-8f83-c0b17c20695d`,
  34.378 seconds. Scope: complete model-manager, provider-info and protocol
  tests, login default-client tests, selected model/reasoning TUI snapshots,
  and native Astra TMUX request/selection/cancel/restart test. The 5,784 filtered
  tests were not run and are not claimed as passes. No new snapshot acceptance
  was required. Log: `/tmp/corbanu-astra-runtime-final-tests-02.log`.
- Harness: **12/12 passed**, including provider errors, reroutes, wrong model,
  unmatched turns/tool outputs, echoed prompts, Code Mode JSON output parsing,
  partial recorder writes, and no regex dependency.
- Additional native TMUX repetition: **1/1 passed**, run
  `8b431d4c-a6ed-4d03-a201-89872d5077bf`. Its existing Rust fixture resolves the
  test-build binary at `codex-rs/target/debug/codex`, SHA-256
  `dc541dd404100fa75eab7d2f3646a5984133bd6b0d2ba15ed319c38efeef729d`;
  it does not honor `CARGO_BIN_EXE_corbanu`. This synthetic check is separate
  from the exact installed-binary live runs below. Captures:
  `/tmp/corbanu-astra-live-sj8Gtg/mock-artifacts-final-repeat/`.
- No Core production adapter or serialized protocol schema was changed; a full
  Core/workspace run was not required for this metadata/negotiation repair.
  Existing Code Mode and streaming adapters were exercised by live TUI tools.
- Governance, portable skill mirrors, diff checks and final source-tree identity
  checks passed. No source changes occurred after the tested source commit.

Final Rust command (from the recorded worktree):

```sh
CARGO_TARGET_DIR=/tmp/corbanu-astra-review-phGuUE/codex-rs/target \
CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_TMUX_REQUIRED=1 \
just test -p codex-wallet -p codex-wallet-daemon -p codex-tasknode-session \
  -p codex-model-provider-info -p codex-provider-auth -p codex-keyring-store \
  -p codex-models-manager -p codex-tui -p codex-cli -p codex-protocol -p codex-login \
  --locked --offline --retries 0 --test-threads 4 \
  -E 'package(codex-models-manager) | package(codex-model-provider-info) | package(codex-protocol) | (package(codex-login) & test(default_client)) | (package(codex-tui) & kind(lib) & (test(model_selection_popup) | test(model_picker) | test(astra) | test(model_reasoning))) | test(tmux_astra_selection_cancel_restart_and_request)'
```

Preceded by `just fix -p codex-models-manager -p codex-model-provider-info
-p codex-protocol -p codex-tui --locked --offline` and `just fmt`.
Fix/build logs: `/tmp/corbanu-astra-runtime-fix-final.log` and
`/tmp/corbanu-astra-runtime-final-build.log`.

### Exact installed-binary live matrix

Both tests execute the installed `bin/corbanu`, SHA-256
`749de8682e2a0628ff3bb1c3bebf9bb859a385aa4938382d4d26e594c07d687f`, with native
authentication from `/home/pfrpc/.corbanu`, OpenAI / `gpt-6-astra` / medium / YOLO.
The hash is checked before and after each run. The bundled Code Mode helper is
the unchanged, working helper with SHA-256
`43a88e88a3cf6728332d196bae4ffef9142f72ce81140e55a7aef0639aa68afb`.

| Repository | Pinned base | Evidence | Result |
| --- | --- | --- | --- |
| TensorCash | `dd6e92024254090de0f596b090bd5c74c4d97b90` | `/tmp/corbanu-astra-live-sj8Gtg/tensorcash-final/` | pass: 9 responses, 6 paired tool calls, 3 successful turns, 1 deliberate abort |
| Isometric Game | `59821b7a85524f186f946c4670480c7ee96483cb` | `/tmp/corbanu-astra-live-sj8Gtg/isometricgame-final/` | pass: 9 responses, 6 paired tool calls, 3 successful turns, 1 deliberate abort |

Disposable worktrees are sibling `tensorcash/` and `isometricgame/` directories
under that same evidence parent. Both read the real project's README and
reported a concrete project fact, fixed only a synthetic Unicode/whitespace
normalization fixture, and executed its unchanged seven-case test. Both then
ran a benign delayed command, cancelled with Escape, recovered with another
test run, exited via `/exit`, restarted with the same thread ID and successfully
executed another history-dependent test request. No trading/backtests, external
project changes or credential copying occurred.

Durable [structured results](astra-runtime-results.json) retain response and
tool IDs, exact repository paths/origins, thread IDs and fixture hashes. Raw
terminal captures, keys and private logs remain in the evidence directories.
TensorCash's configured origin is `postfiatorg/tensorcash`; the policy's
`agtico/tensorcash` URL and configured URL both resolved to HEAD `9325ed67d23355170d6ad38ad58ea776d049ae4e`
when checked. The recorded pinned base, not that newer HEAD, was tested.

Summary hashes: TensorCash
`28a6403976abc0c955e3069008b8723a5dca76ac9e348ff61ed880a8b05d09da`;
Isometric Game `6c95b309fefec6d3ac377a4e0e6bc6bab1706418f39bbeec9d2b987251ff2220`.

### Human handoff

`corbanu-debug --yolo` now resolves to
`/home/pfrpc/.local/share/corbanu-debug/0.1.38-astra-runtime-f848954d77/bin/corbanu`.
Default-server session `corbanu-test`, pane `%20`, was restarted and visibly
shows `gpt-6-astra medium`, YOLO, and `~/repos`. `/proc/2593387/exe` confirmed
the exact installed executable. It uses the approved normal profile and did
not show credential onboarding. Attach with `tmux attach -t corbanu-test`.

Previous launcher is recoverable at
`/home/pfrpc/.local/share/corbanu-debug/launcher-backup-yYKxdG/corbanu-debug`.
The normal `corbanu` launcher and other tmux sessions are unchanged.
No Linux/macOS/Windows release or competitor benchmark qualification is implied.
No new tag/workflow/publication was triggered; named-human acceptance is still
pending the user's test. Real account inference is tested here; a separate live
API-key billing-route run and long-duration soak are not claimed.
