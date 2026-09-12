# September 8 candidate repair record

User authority: “fix the product so that the failing tests pass.”

Product citation: **Shipping MVP — LIVE** — “operational credential use without
placing raw values in chat”; “model-aware delegation, durable mailboxes,
supervision, resume, and recovery”; and the shipped runtime, wallet and MCP
workspace capabilities in that same heading.

PF-58-S01 continues the credential recovery initiative in its allocated
`feat/provider-reauth-health` worktree. Explicit manager recovery with no selected
Claude source must present the existing explicit replacement choice, not cancel
silently. Known and externally owned sources retain their existing restrictions.

Adjacent bounded repairs restore already-authorized shipping behavior without
changing authorization, custody, financial actions or persisted formats:

- Stage matching Code Mode and wallet companions; correct package discovery for
  the existing executable brands. Scope: `scripts/codex_package/`,
  `codex-rs/wallet-daemon/src/client.rs`, existing PF-58 candidate staging.
- Bind directly created native panes to their live parent's existing effective
  policy before returning the new thread. Scope: `core/src/thread_manager.rs`
  and `core/src/agent/control_tests.rs`; no policy relaxation or detached-child
  authority is allowed. Ordinary agent-spawn inheritance remains unchanged.
- Install a pinned Node runtime on the RTX and include it in the candidate
  launcher environment; verify the two actually installed MCP servers without
  reading or recording credentials. Do not disable plugins to pass readiness.
- The extra live Fable check reproduced a permission-profile reload using the
  saved Astra model with the CLI-selected Claude provider. Preserve the active
  or explicitly requested model/provider pair while loading permission settings.
  Scope: `app-server/src/request_processors/turn_processor.rs` and its public
  turn-start regression suite. Permission restrictions are still enforced; no
  fallback, provider change, schema or new authorization is introduced.
- Apply the same runtime preservation to live-thread configuration rebuilds
  used by `/mcp`; the public-RPC regression also requests inventory after the
  turn. Fresh requirements and installed plugin configuration still reload.
- Resolve the built-in Claude helper to all supported CLI executable names,
  including the packaged `codex` candidate, and retain that executable during
  permission reload. Do not rewrite arbitrary external auth commands or invoke
  an app-server executable as a CLI. Scope: existing core config helper and tests.

Builds and final affected tests run on RTX. Preserve the user's current session
and auth stores. Run the complete 45-journey TMUX matrix on the staged package,
plus focused inheritance, credential and package tests. Record failures without
weakening assertions or claiming human acceptance. PF-58's five-review cap is
already exhausted; this repair does not reset it. No publication is authorized.

## Results

- Permission reload regression reproduced the incompatible saved model/provider
  error before the fix. After the fix, that public-RPC regression and both
  permission rejection/workspace-root regression tests pass (3/3).
- Extending the same RPC journey to MCP inventory reproduced `failed to reload
  config` with the incompatible saved pair before the live-thread rebuild fix.
- Two intermediate pinned-package matrices passed 45/45, including native-child,
  all wallet onboarding and all Claude recovery cases. They do not supersede
  final-package qualification after the additional live-discovered fixes.
- **Final package: 45/45 TMUX tests pass**, with no failing or skipped selected
  cases. Provider management 10, reauthentication 4, convergence 13 (including
  active-runtime `/mcp`), wallet/multi-provider 10, Claude auth 3, stream 1,
  security 2 and memory/restart 2. All executable aliases were pinned to the
  same immutable packaged CLI, not stale development binaries.
- Final focused core checks pass 7/7 (including fresh requirements/plugins,
  helper resolution and native-pane admission); public-RPC checks pass 3/3.
  Additional affected checks during repair: wallet 13/13, provider-auth 71/71,
  focused TUI intent 2/2, inheritance/fail-closed controls 4/4. Package Python
  tests pass 28/28 and gate regressions 10/10. Scoped `just fix`, formatting and
  `git diff --check` pass; existing warnings remain.
- On the exact final package, the tool smoke executes host/shell/MCP calls four
  times, three concurrent sessions, restart, stream cancellation/recovery and
  narrow guidance. Actual launcher dependency checks pass. Live Fable 5.1 Max
  returns the requested marker, executes the requested `printf` shell tool,
  and successfully lists `codex_apps` and both installed local MCPs via `/mcp`.
- Candidate: `candidate-repaired-qualified/bin/codex` under the RTX evidence
  root. CLI SHA-256:
  `fa2b710733a57787cb0cfbd250be903ef9d2fe1f08b981dd49196697eae14fb8`.
  [Metadata-only final reports and matching source manifest](evidence/repairs-20260908/)
  are retained locally; no credentials or raw live trace logs were exported.
- The full handoff gate intentionally remains **blocked on 11 explicit
  platform/live/full-flow evidence gaps**, not failed automated tests. All 26
  human checks remain mapped and unchecked. PF-58 remains `in_progress`; human
  sign-off, live TensorCash/Isometric qualification, benchmarks and release/main
  integration are not claimed. The plan checker passes; unrelated pre-existing
  PF43–45 sprint-ID/backlink/order errors remain documented.
- No sixth independent review, main merge or push was performed. Existing user
  terminals and stored credentials were left intact. Changes remain uncommitted
  in the allocated branch and mirrored on RTX.
