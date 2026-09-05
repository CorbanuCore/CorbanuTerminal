# Astra native runtime qualification — September 5, 2026

Status: implementation and first live diagnostic passed; final-tree qualification
and installation are in progress. This is not a release authorization.

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

Pending final affected Rust/TUI tests, two-repository live rerun, exact installed
binary, and separate human test session. Twelve harness regression tests pass
before final formatting. No Linux/macOS/Windows release or competitor benchmark
qualification is implied, and no named-human acceptance has yet been received.
