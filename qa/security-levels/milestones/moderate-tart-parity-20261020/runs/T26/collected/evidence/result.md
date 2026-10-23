# Independent functional execution report — group T26

Run UTC started 2026-10-20T18:24:11Z. Product model/header: GLM 5.3 Flash. Connection: `zai`. No human acceptance is claimed.

## Preflight and candidate identity

- 2026-10-20T18:21–18:24Z: Executor ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; exit code was `0`.
- The executor probe reports `"ok": true`.
- Coordinator preflight summary `preflight-summary.json` reports `"ok": true`, `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `failed: []`, `probe_count: 99`. Both beside preflight files reported `"ok": true`.
- SHA-256 `/opt/agms/pkg/bin/corbanu`: `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6` — matches assignment.
- SHA-256 frozen cases file matches `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`.
- SHA-256 amendments file matches `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`.
- SHA-256 amendments-v3 file matches `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3`.
- SHA-256 amendments-v4 file matches `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae`.

## MOD-34 · The person's own broker_model_auth = false still wins under Moderate

### Starting conditions and fixture

- Fresh product home initially contained only `config.toml`; the run config had the milestone feature flags on and no `broker_model_auth` setting.
- 2026-10-20T18:24:46Z, from the tester's own shell, added `broker_model_auth = false` immediately under the existing `[features]` table. Pre-edit copy: `evidence/config-before-mod34.toml`; post-edit value is in `product-home/config.toml`.
- Before actions, `git status --short --branch` showed `## main`, with no changes.
- Before fixture capture 2026-10-20T18:24:22Z (`agms-tui fixture before-mod34`): project `.env` was 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`; `outside-notes.txt` contained `outside-the-project: synthetic file`.
- After the case 2026-10-20T18:25:57Z (`agms-tui fixture after-mod34`): `.env` remained 98 bytes with the same SHA-256; `outside-notes.txt` remained unchanged; project source files remained unchanged.

### Requested versus actual actions

| Time UTC | Requested / case action | Actual action | Result |
| --- | --- | --- | --- |
| 18:24:50 | Start product | `agms-tui start`; executor launcher executed `/opt/agms/bin/agms-product zai glm-5.3-flash` at 18:24:52 | Started; first-start trust prompt appeared (screens `mod34-00-first-start`, `after-start`) |
| 18:24:55 | Neutral navigation to enter the trusted fixture project | Observed trust options `1. Yes, continue` / `2. No, quit`; pressed Enter on displayed option 1 | Trust prompt cleared; main UI showed v0.1.48 and `model: glm-5.3-flash` (`mod34-01-after-trust`) |
| 18:25:01–18:25:11 | `/security`, Down to Moderate, Enter (read review), Enter, `r` | Typed `/security`; separate Enter; pressed Down; separate Enter; read review; separate Enter; pressed `r` | Completed as shown below |
| 18:25:18–18:25:25 | `/security`, `i`; read Model keys and Level rows; scroll | Typed `/security`; separate Enter; pressed `i`; captured top and two scrolled positions | Completed as shown below |
| 18:25:25–18:25:28 | Esc, Esc; send `Reply with exactly: pong` | Pressed Escape twice; typed prompt text; waited; sent Enter separately | Completed as shown below |
| 18:25:51 | Confirm turn completion | Captured screen | Turn showed `• pong`; no Working indicator; no broker error |

No approval prompt appeared. No cancellation or recovery path was offered during the turn. No model decline occurred.

### Positive visible checkpoints

| Time UTC | Screen label | Checkpoint |
| --- | --- | --- |
| 18:25:01 | `mod34-02-security-permissive` | `/security` shows **Active in this session: Permissive** and Permissive `(active)`. Protected-launch wording says agent commands never run outside the sandbox, even when approved. |
| 18:25:05 | `mod34-03-moderate-highlight` | Moderate is highlighted in the picker while Permissive remains active. |
| 18:25:06 | `mod34-04-moderate-review` | Review describes Moderate. Model-keys section states that `broker_model_auth = false` in the person's own config keeps the broker off. Review ends `Your config.toml is not modified.` |
| 18:25:11 | `mod34-05-moderate-saved` | `Saved: Moderate` shown; Core's rules active in session; model-key broker described as starting at restart “unless your own config turns broker_model_auth off”. `r restart now` offered. |
| 18:25:11 | `mod34-06-after-restart` | Product restarted. Startup warning: **“Security level Moderate: broker_model_auth is off (set in your configuration or by policy), so Corbanu reads provider keys itself instead of the credential broker.”** Header shows model and folder. |
| 18:25:18 | `wait-matched:Security level` | After restart `/security` shows **Active in this session: Moderate**, **Core's level: Moderate**, and Moderate `(active)`. |
| 18:25:20 | `mod34-07-inspector-top` | Inspector shows **“◐ Moderate set; off or not observed: Model key broker.”** Level rows show `Saved: Moderate [configured: /security]`, `Core config: Permissive [configured: config layers]`, and `[set] Active this run: Moderate…`. Model-keys row says `[set] Model keys: Corbanu reads provider keys itself (broker_model_auth off)`. |
| 18:25:20 | `mod34-08-inspector-down1`, `mod34-09-inspector-down2` | Inspector scroll observed; the same Model keys and Level rows remain visible. |
| 18:25:27 | `mod34-10-before-prompt` | Inspector/picker closed; startup warning remains visible; prompt entry is available. |
| 18:25:28 | `mod34-11-pong-complete` | Turn accepted and showed `Working (0s…)` after Enter. |
| 18:25:51 | `mod34-11-pong-20s` | Turn completed with visible model output `• pong`; no broker error. |

### Security level shown at transitions

- Before switching: `/security` showed `Active in this session: Permissive`.
- Immediately after confirming: `Saved: Moderate`, with Core's Moderate rules active in session and broker/active level waiting for restart.
- After `r`: startup warning shows `Security level Moderate`; `/security` shows `Active in this session: Moderate`, `Core's level: Moderate`, Moderate `(active)`.
- Inspector: `Saved: Moderate`; `Core config: Permissive [configured: config layers]`; `Active this run: Moderate`; `Live policy: Moderate in force`; status line `◐ Moderate set; off or not observed: Model key broker`.

### Model declines/refusals

- None. The model completed `pong`.

### Expected-result dispositions

- **Moderate is saved and, after restart, active or shown as partial — passed.** `Saved: Moderate` was shown, and after restart `/security` showed Moderate active.
- **After restart neither `/security` nor inspector claims model keys are held by the broker; inspector model-key row says broker off and Corbanu reads provider keys itself — passed.** Both screens say `broker_model_auth` is off / Corbanu reads provider keys itself. Pre-restart review and Saved screens correctly mention the conditional.
- **The turn completes with `pong` and no broker error — passed.** Visible at 18:25:51 (`mod34-11-pong-20s`).
- **Startup warning and `/security` partial reporting — recorded with ambiguity.** The startup warning was shown exactly as expected. `/security` did not show the word “partial”; it showed Moderate as active. The inspector showed a `◐` status and “Moderate set; off or not observed: Model key broker,” but no visible word “partial.” This outcome is therefore recorded as **inconclusive** because the case asks whether partial reporting is shown and the visible wording is ambiguous.
- No product refusal, security decline, or missing required control occurred.
- Fixture and project remained unchanged.

VERDICT MOD-34: passed
