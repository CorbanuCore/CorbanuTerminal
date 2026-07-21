# Urgent Execution Fixes: Orchestrate and Provider Paths

**Date:** 2026-07-10
**Branch:** `integrate/pfterminal-20260707` at `7d75c15fe`
**Scope:** Defects found by direct code inspection of the shipping execution paths for
`/orchestrate` (whips) and the provider/vault subsystem. This spec lists only defects with
user-visible runtime impact. Process/test-matrix debt from
`PRE_RELEASE_ARCHITECTURE_REVIEW_20260710.md` is out of scope here.

All paths are relative to `codex-rs/` unless noted. Line numbers are at `7d75c15fe`.

---

## Fix 1 (P0): Synchronous vault decryption freezes the TUI thread

### Background

Every `codex_vault::Vault` read on the local-secrets backend decrypts an age/scrypt file
(~5.2 s measured on this machine). `secrets/src/local.rs` has an in-memory decrypted cache
(`cached_file`, lines 80, 107, 167), but it is per-`LocalSecretsBackend` instance and every call
site constructs a fresh `Vault::new(...)`, so the cache never hits. This is the same defect class
as the `/providers` freeze that was already fixed at `7d75c15fe` (status loads moved off-thread).
Four shipping paths still block the TUI event loop:

### 1a. Every turn sent to a bridge-transport Claude pane (worst case)

- `tui/src/claude_panes/command_plan.rs:89` — `build_claude_command_plan` calls
  `reveal_provider_secret` (index decrypt + secret decrypt, ~2 vault accesses) for
  `AmbientChatBridge` and `AnthropicPassthroughBridge` transports.
- Called synchronously from `prepare_turn` (`tui/src/claude_panes/registry.rs:260`), which runs on
  the TUI event path (`tui/src/claude_panes/app_integration.rs:623` and `:693`).
- 4 shipping pane profiles use these transports (`tui/src/claude_panes/provider.rs:73,85,133,145`).
- Impact: the UI freezes for the full decrypt cost on **every user prompt** to those panes, and on
  every **whip auto-fire** into them (`AppEvent::SubmitSpawnClaudePaneTask` →
  `submit_claude_pane_task`, `app_integration.rs:642`) — i.e. periodic freezes with no user input.

### 1b. Opening the `/vault` menu

- `tui/src/chatwidget/vault_menu.rs:10-12,28` — `open_vault_menu` calls
  `sorted_vault_credentials` → `Vault::list()` → index decrypt, synchronously.

### 1c. Copying a vault secret to the clipboard

- `tui/src/chatwidget/vault_menu.rs:71` — `copy_vault_secret_to_clipboard_with` calls
  `vault.reveal(&label)` (index + secret decrypt) synchronously; dispatched from
  `tui/src/app/event_dispatch.rs:1329`.

### 1d. Creating a Claude pane and `/vault` subcommands

- `tui/src/claude_panes/registry.rs:170` — `create_pane_with_role` calls
  `ensure_vault_label_exists` → `Vault::exists()` → index decrypt.
- `tui/src/chatwidget/slash_dispatch.rs:753` — `/vault list|show|...` runs
  `handle_vault_command` synchronously (`tui/src/vault_command.rs:25` constructs the vault).

### Required behavior

- No TUI event-loop callback may perform vault decryption, keyring access, or scrypt work.
- Turn submission to bridge-transport Claude panes must not block the renderer; the secret reveal
  moves into the async turn task that already spawns the child process (mirror how non-bridge
  transports defer to `pfterminal vault auth-helper` at use time, `command_plan.rs:329`).
- Vault-backed UI surfaces (`/vault` menu, copy-to-clipboard, pane creation, `/vault` subcommands)
  open immediately and complete the vault work in the background, following the established
  `/providers` pattern: immediate view + background load + bounded wait + result event
  (`tui/src/chatwidget/provider_credentials.rs:110-131`).

### Implementation notes

- Introduce one long-lived shared vault handle (or share the `LocalSecretsBackend` cache across
  `Vault::new` calls for the same `codex_home`) so at most one decrypt is paid per process per
  file change. Cache invalidation may key on file mtime; writes already refresh the cache
  (`secrets/src/local.rs:234-236`).
- Do not log or persist secret material while refactoring; secrets stay out of events. Reuse the
  masked-secret wrapper pattern (`ProviderApiKeySecret`) if a secret must cross an event boundary,
  or better, reveal inside the consuming task so it never crosses one.

### Acceptance

- With a vault whose decrypt takes ≥5 s (or an injected slow backend): typing remains responsive
  while (a) submitting a turn to an Ambient-bridge pane, (b) opening `/vault`, (c) copying a
  secret, (d) creating a Claude pane. Verify in a real PTY, per the release-process finding.
- A regression test asserts `build_claude_command_plan`'s TUI-thread portion performs no vault
  reveal for bridge transports (e.g. plan carries a deferred-secret handle, not the secret).
- Repeated vault operations in one session decrypt at most once (cache-hit test).

---

## Fix 2 (P1): Claude status check has no timeout; `/providers` can show "Checking..." forever

- `tui/src/chatwidget/provider_credentials.rs:117-121` — the background refresh joins
  `claude_code_login::current_status()` with the vault statuses, but only the vault side is wrapped
  in `PROVIDER_STATUS_TIMEOUT` (10 s, line 27).
- `tui/src/chatwidget/claude_code_login.rs:229-233` — `read_status` runs
  `claude auth status --json` with no timeout. A hung CLI leaves the Claude row (and the
  `ProviderCredentialStatusesReady` event) blocked indefinitely.

### Required behavior

Wrap the Claude status future in the same 10 s bound; on timeout, report a distinct
"Status unavailable" state, never a permanent "Checking...".

### Acceptance

Unit test with a stub executable that sleeps: rows resolve to unavailable within the bound and the
ready event still fires.

---

## Fix 3 (P1): Detached whips are never deleted

- `tui/src/orchestrate.rs:1605` (attach-replace) and `set_whip_state_by_ref`
  (`orchestrate.rs:1630`, used by `/orchestrate detach`) only set `WhipState::Detached`.
- No code path ever removes entries from `orchestrate_whips` (no `remove` in the module).
- Detached whips are persisted with the pane layout
  (`tui/src/claude_panes/app_integration.rs:352`) and restored on every startup
  (`tui/src/app.rs:1305-1320`), and the `/orchestrate` status view lists all whips unfiltered
  (`orchestrate.rs:765`). Every attach-replace or detach permanently grows saved state and the
  status list.

### Required behavior

- Detach removes the whip from the registry (after emitting the existing info message), or at
  minimum detached whips are excluded from persistence and from the status view.
- Attach-replace deletes the superseded whips rather than parking them as `Detached`.
- Also prune `orchestrate_idle_generation_by_target` entries whose target has no live whip
  (currently the map only grows).

### Acceptance

- Attach → detach → persist → restore yields zero whips in state and in `/orchestrate`.
- Attach-replace N times yields exactly one persisted whip.
- Existing tests referencing detached whips (`find_whip_id` filtering, agent replacement refusal)
  updated to the new semantics.

---

## Fix 4 (P1): Whips can fire ~45 s after restart from stale state

- On restore, idle generations are reset to 0 for every whip target (`tui/src/app.rs:1325-1328`)
  while each whip's persisted `last_idle_generation_fired` survives (e.g. `Some(3)`).
- The periodic sweep (`WHIP_SWEEP_INTERVAL` 45 s, `tui/src/app.rs:250`;
  `sweep_orchestrate_whips`, `orchestrate.rs:1433`) then evaluates armed whips with
  `FireTrigger::Tick`: generation `0 != 3` passes the once-per-idle-period check, and if the target
  is idle and cooldown has lapsed, the whip fires — including **auto** whips that execute tasks —
  even though the target may have completed its work before the app was closed.
- Symmetrically, a whip whose last fire happened at generation 0 is wrongly suppressed after
  restart until a fresh edge.

### Required behavior

After restore, a whip may not fire until its target completes at least one turn in the new session
(a fresh idle edge). Concretely: on restore, set each restored whip's
`last_idle_generation_fired = Some(0)` to match the reset generation maps (or persist and restore
the generation map; the former is simpler and safer).

### Acceptance

- Deterministic test: whip armed with `last_idle_generation_fired = Some(3)` restored against a
  fresh generation map; `sweep_orchestrate_whips` does not fire; after the target runs and goes
  idle (generation 1), the whip fires once.
- Manual-fire (`/orchestrate fire`) remains exempt from the gate (existing
  `FireTrigger::Manual` behavior).

---

## Fix 5 (P2): Failing turns re-arm auto whips into a 20-shot error loop

- Claude pane turn errors call `note_whip_target_idle_with_fire_control` with the error text and
  `allow_fire=true` (`tui/src/claude_panes/app_integration.rs:884-889`); the native path
  similarly reports the last result on idle (`tui/src/app/thread_routing.rs:1728`).
- Error text is non-empty, so `pause_spinning_whips_on_empty_output` (`orchestrate.rs:1832`,
  threshold: 2 consecutive empty outputs) never trips.
- Result: a persistently broken pane (Claude CLI missing, expired auth, bad bridge config) is
  auto-whipped every cooldown (default 60 s) until `max_fires` (default 20) — ~20 failed turns and
  associated error spam before the whip exhausts.

### Required behavior

Failed turns count toward the auto-pause protection: track consecutive failed-turn fires per whip
(reuse or parallel the `empty_output_fires` counter) and pause the whip with a visible reason after
2 consecutive failures, matching the empty-output behavior. A successful turn resets the counter.

### Acceptance

Test: auto whip on a pane whose turns error twice consecutively → whip is `Paused` with reason,
fire count ≤ 2; a success between failures resets the streak.

---

## Fix 6 (P3, optional in this pass): Malformed non-base64 `data:` image URLs sent to Anthropic

- `core/src/client.rs:3137-3153` — `anthropic_image_block` falls back to
  `{"type":"url","url":...}` for any `data:` URL lacking `;base64`, sending a data URI where
  Anthropic requires http(s); the request 400s.

### Required behavior

Reject non-HTTP(S), non-base64 image sources before serialization with a clear local error instead
of shipping a doomed request.

### Acceptance

Unit test: `data:image/png,percent-encoded` input produces a local validation error and no request.

---

## Delivery constraints

- Order: Fix 1 first (largest, touches vault plumbing); Fixes 2-5 are independent of it and of
  each other and may proceed in parallel.
- Keep each fix a separate commit with its tests; no drive-by refactors (the `WhipRegistry`
  extraction and provider-adapter work from the architecture review remain follow-up debt, not
  part of this pass).
- No secret values in code, logs, tests, or transcripts; slow-vault tests use injected fake
  backends, never real keys.
- Every TUI-facing fix (1, 2) gets real-PTY verification in addition to unit coverage, per the
  provider-freeze release-process finding.
