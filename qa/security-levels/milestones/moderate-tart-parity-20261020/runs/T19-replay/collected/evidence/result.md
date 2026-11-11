# Functional execution report — group T19

Independent functional executor. I did not claim human acceptance.

## Isolation and candidate preflight

- 2026-10-20T19:30:14Z: ran the required `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json`; rc=0; its top-level `"ok"` was `true`.
- Coordinator preflight files inspected: `preflight-summary.json` reports `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `failed: []`, `probe_count: 99`, `ok: true`; `preflight-launcher.json` and `preflight-tmux-child.json` both report `ok: true`.
- Candidate `/opt/agms/pkg/bin/corbanu` SHA-256 matched `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6`. Probe `--version` output was `corbanu 0.1.48`.
- Frozen packet SHA-256 values matched all four supplied values.

## MOD-23 — Moderate is not saved without the untrusted-content labels

### Starting conditions and state/profile

- 2026-10-20T19:32:25Z, fixture label `before-MOD23`: fixture files were recorded. Relevant state: workspace `.env` was 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`; `outside-notes.txt` contained `outside-the-project: synthetic file`; all listed workspace files were in their supplied state.
- The supplied fresh product profile contained only the supplied `product-home/config.toml` before launch. From the tester shell, I set `source_envelopes = false` under `[features]` as the required starting condition. No security-state file was present before launch.
- Product was launched at 2026-10-20T19:32:28Z with plain `agms-tui start` (route `zai`, model `glm-5.3-flash`). The product showed `Corbanu Terminal (v0.1.48)` and `model: glm-5.3-flash`.
- The product showed the required trust prompt for the fixture workspace. I pressed Enter on the displayed `Yes, continue` option at 2026-10-20T19:32:34Z. The main screen then appeared.

### Requested versus actual actions

- Requested `/security`, ↓, Enter:
  - 2026-10-20T19:32:39Z: typed `/security` without Enter, then observed `security-picker` and pressed Enter separately.
  - 2026-10-20T19:32:49Z: pressed Down while Moderate was highlighted (`moderate-highlight`).
  - 2026-10-20T19:32:50Z: pressed Enter while Moderate was highlighted. This reached the `moderate-review` screen.
- Requested press Enter twice:
  - 2026-10-20T19:32:56Z: pressed Enter once (`after-first-enter`).
  - 2026-10-20T19:33:00Z: pressed Enter again (`after-second-enter`).
- Requested Esc:
  - 2026-10-20T19:33:05Z: pressed Escape (`after-esc`), which returned to `/security` and showed `Cancelled. Nothing changed.`; Active remained Permissive.
- Requested `/security`, `i`:
  - 2026-10-20T19:33:09Z: typed `/security`, pressed Enter separately, observed the Permissive picker, then pressed `i` at 2026-10-20T19:33:10Z (`inspector-permissive`).
- Requested relaunch:
  - I returned to the main screen with Escape and sent `/quit` as separate text and Enter at 2026-10-20T19:33:24Z. The product exited and its tmux server ended before `agms-tui screen after-quit` could run.
  - Ran `agms-tui stop`, then started again with plain `agms-tui start` at 2026-10-20T19:33:29Z.
  - At 2026-10-20T19:33:35Z, typed `/security`, pressed Enter separately, observed Permissive picker, then pressed `i` for `relaunch-inspector`.
- All prompt text was sent separately from Enter. No approval-prompt text was typed. No `!` shell escape was used.

### Positive checkpoints

- `trust-prompt`: the product asked whether to trust the fixture workspace and showed `Yes, continue`.
- `security-picker`: Active in this session was **Permissive**.
- `moderate-highlight`: Moderate was highlighted while the picker still said **Permissive**.
- `moderate-review`: the review said, verbatim in relevant part: `Moderate needs the untrusted-content labels (the source_envelopes feature): without them Corbanu refuses every model request under Moderate. Turn source_envelopes on and restart Corbanu Terminal first.`
- `after-first-enter` and `after-second-enter`: the review remained displayed after both Enter presses and added: `Not saved: Moderate needs the untrusted-content labels (the source_envelopes feature): without them Corbanu refuses every model request under Moderate. Turn source_envelopes on and restart Corbanu Terminal first.` Its visible footer offered `↑/↓ scroll · esc back, nothing changes · i inspect`; there was no Enter-confirm option.
- `after-esc`: `/security` showed `Active in this session: Permissive` and `Cancelled. Nothing changed.`
- `inspector-permissive`: Inspector showed `Saved: Permissive (nothing saved)`, `Core config: Permissive`, `Active this run: Permissive`, and `Live policy: Permissive in force; next start Permissive`.
- After relaunch, `relaunch-security-picker` showed **Permissive**, with `Permissive (active)` selected. `relaunch-inspector` again showed `Saved: Permissive (nothing saved)`, `Core config: Permissive`, `Active this run: Permissive`, and `Live policy: Permissive in force; next start Permissive`.

### Fixture contents before/after

- `after-MOD23` at 2026-10-20T19:33:55Z matched `before-MOD23`: no workspace file changed; `.env` remained 98 bytes with SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`; `git status --short` was empty.
- Product `config.toml` still had `source_envelopes = false` after the case. The product created its own runtime/profile files but no Moderate or Permissive `/security` state file was added; I did not inspect product databases.

### Refusals / declines

- No model refusal or decline occurred; no model turn was requested.

### Expected-result decisions

- The review names `source_envelopes` and says how to proceed (turn it on and restart): **passed**.
- There was no `enter confirm` option, and both Enter presses saved nothing (`Not saved` remained, and Esc showed nothing changed): **passed**.
- The inspector showed `Saved: Permissive`, and it remained Permissive after relaunch: **passed**.
- No required control, fixture, or completion checkpoint was missing. No credential/system password prompt or unexpected profile access occurred.

VERDICT MOD-23: passed
