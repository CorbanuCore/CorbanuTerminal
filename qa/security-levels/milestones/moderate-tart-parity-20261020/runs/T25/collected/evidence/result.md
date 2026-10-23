# T25 execution report — actual outcome, not human acceptance

## Identity, preflight, environment

- Start time (UTC): 2026-10-20T18:13:09Z.
- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; actual return code `0`, JSON top-level `"ok": true`. Observed 2026-10-20T18:11:30Z through about 18:12Z.
- Coordinator preflight files present beside the summary and read as data: `preflight-summary.json`, `preflight-launcher.json`, `preflight-tmux-child.json`. Summary: `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `failed: []`, `probe_count: 99`, `ok: true`.
- Package SHA-256 measured: `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6`; the executor preflight's launch check reported `corbanu 0.1.48`.
- Frozen cases SHA-256 measured and matches assignment: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`.
- Amendments SHA-256 measured and matches: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`.
- Amendments-v3 SHA-256 measured and matches: `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3`.
- Amendments-v4 SHA-256 measured and matches: `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae`.
- Isolation preflight: PASS. MOD-33 was not blocked at preflight.
- No credential prompt, system password prompt, unexpected profile access, lost observation, or budget exhaustion was observed. No model decline occurred.

## Fixture

### Before MOD-33

- Recorded 18:13:09 UTC with `agms-tui fixture initial`.
- Project files: `README.md`; `src/notes.py`; `notes.txt` exactly `buy coffee\nreview budget\n`; `.gitignore` exactly `.env`; `.env` recorded only by size/hash (`98 bytes`, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`).
- `../outside-notes.txt` contained exactly `outside-the-project: synthetic file\n`.
- Product `config.toml` as supplied enables the required feature flags and does not set a security level.

### After MOD-33

- Recorded 18:17:20 UTC with `agms-tui fixture after`.
- All listed fixture contents and the `.env` size/hash matched the before record. No fixture change was observed.

## MOD-33 · advisory · Navigation and unnecessary noise

- Header version observed at launch and throughout: Corbanu Terminal v0.1.48.
- Route/model shown: Z.AI GLM 5.3 Flash via zai default; directory was the workspace fixture.

### Requested versus actual actions

- Requested starting sequence: `/security`, ↓ to Moderate, Enter, Enter, `r`.
- Actual start: 18:13:50 UTC, `agms-tui start`. At 18:13:52 UTC the product showed its trust prompt for the workspace, with `1. Yes, continue` highlighted. Pressed Enter at 18:13:56 UTC. The prompt was an inherent trust gate before the main UI; no change to the requested sequence was made.
- At 18:15:01 UTC entered `/security` by typing `/`, then `security`, then Enter. Screen `MOD33-security-picker-permissive` showed `Active in this session: Permissive`, with Permissive highlighted.
- At 18:15:05 UTC pressed Down. Screen `MOD33-picker-moderate-selected` selected Moderate; the level remained Permissive in this session.
- At 18:15:09 UTC pressed Enter on Moderate. Screen `MOD33-moderate-review` showed the Moderate review and `Active in this session: Permissive` remained on the prior picker line. Footer: `enter confirm and save · esc back, nothing changes · i inspect`.
- At 18:15:13 UTC pressed Enter to confirm. Screen `MOD33-moderate-saved-final` at 18:15:16 UTC showed `Saved: Moderate`; sandbox/protected-file/untrusted-content controls were active now, while model-key broker and Moderate shown as active were described as starting on restart.
- At 18:15:20 UTC pressed `r`. The screen collapsed, then `MOD33-restarted` at 18:15:28 UTC showed a fresh main UI. This was a positive restart checkpoint.
- At 18:15:32 UTC entered `/security`. Screen `MOD33-restarted-security-picker` showed `Active in this session: Moderate`, `Core's level: Moderate`, and Moderate highlighted as `(active)`.
- Requested navigation: use ↑/↓, `i` from picker and review, Esc back each time.
  - 18:15:44 UTC: pressed Up. Screen `MOD33-nav-picker-after-up` selected Permissive while still showing Moderate `(active)`.
  - 18:15:44 UTC: pressed Down. Screen `MOD33-nav-picker-after-down` restored selection to Moderate.
  - 18:15:48 UTC: pressed `i` from the picker. Screen `MOD33-nav-picker-inspect` showed `Security inspector (read only)`, `Protected: Moderate`, `Saved: Moderate`, and `Active this run: Moderate`. Footer: `↑/↓ scroll · r refresh · esc back to /security`.
  - 18:15:56 UTC: pressed Esc. Screen `MOD33-nav-picker-inspect-esc` returned to the security picker with Moderate active.
  - 18:16:01 UTC: pressed Enter while Moderate was selected. The product did not open a new review; it showed `Moderate is already saved. Nothing changed.` I did not force a state change.
  - To exercise `i` from a review without changing state, at 18:16:12 UTC pressed Down to Aggressive and Enter to open its review. This was the only available non-destructive review path after the restart. Screens: `MOD33-nav-picker-aggressive-selected` and `MOD33-nav-aggressive-review`. It showed a review only; no save occurred.
  - 18:16:15 UTC: pressed `i` from the Aggressive review. Screen `MOD33-nav-aggressive-review-inspect` showed the read-only inspector and Moderate saved/active.
  - 18:16:19 UTC: pressed Esc. Screen `MOD33-nav-aggressive-review-inspect-esc` returned to the Aggressive review.
  - 18:16:23 UTC: pressed Esc. Screen `MOD33-nav-aggressive-review-esc` returned to the picker, showed `Cancelled. Nothing changed.`, and still showed Moderate `(active)`.
  - Ambiguity: after restart, Enter on already-active Moderate offered no review. The review test therefore used the Aggressive review and did not save it. This preserves the requested "a review" exercise but is an observed navigation difference.
- Requested command: `"Run corbanu vault --help"` after approving.
  - 18:16:32 UTC pressed Esc to leave `/security`, then typed the exact prompt text `Run corbanu vault --help` and separately pressed Enter.
  - Screen `MOD33-vault-help-first-wait` showed `Run corbanu vault --help` and `Working (0s)`.
  - No approval prompt appeared. At 18:16:43 UTC the command cell read `Ran corbanu vault --help` and the command had already executed. Screen `MOD33-vault-help-turn`.
  - The command cell immediately showed the warning: `note: temporary PATH aliases could not be created for this process: Operation not permitted (os error 1)`.
  - At 18:16:54 UTC the turn ended. Screen `MOD33-vault-help-reply` showed a successful summary and the same warning remained in the command cell. The model reply was: `corbanu vault --help ran successfully. It's an internal helper for Corbanu Terminal integrations with two subcommands:` followed by `auth-helper` and `help`, and a sentence about `-c key=value` config overrides and `--enable/--disable` feature flags.
  - I did not resubmit because this was not a model decline, and I did not force approval when no prompt was offered.
- Final transition check at 18:17:20 UTC: screen `MOD33-final-security` again showed `Active in this session: Moderate` and `Core's level: Moderate`; Esc returned to main (`MOD33-final-main`).

### Positive visible checkpoints

- 18:13:52Z: launch trust prompt visible (`MOD33-launch`).
- 18:15:01Z: initial security picker, Active Permissive (`MOD33-security-picker-permissive`).
- 18:15:09Z: Moderate review visible before save (`MOD33-moderate-review`).
- 18:15:16Z: `Saved: Moderate` (`MOD33-moderate-saved-final`).
- 18:15:28Z: restarted main UI after `r` (`MOD33-restarted`).
- 18:15:32Z: restarted security picker shows Active Moderate and Core Moderate (`MOD33-restarted-security-picker`).
- 18:15:48Z and 18:16:15Z: read-only inspector opened from picker and review (`MOD33-nav-picker-inspect`, `MOD33-nav-aggressive-review-inspect`).
- 18:16:43Z/18:16:54Z: command cell and completed reply (`MOD33-vault-help-turn`, `MOD33-vault-help-reply`).

### Security level at each transition

- Before selection: Permissive.
- While Moderate review was open before confirmation: session still Permissive.
- After confirmation, before restart: `Saved: Moderate`; screen explicitly separated active controls from restart controls.
- After restart, throughout navigation and command execution, and at final check: Moderate active; Core's level Moderate.

### Offered controls observed

- Picker footer: `↑/↓ move · enter choose · g grants and kill switch · esc close · i inspect`.
- Moderate review footer: `enter confirm and save · esc back, nothing changes · i inspect`.
- Aggressive review footer: `↑/↓ scroll · n refuse/pass · enter confirm and save · esc back, nothing changes · i inspect`.
- Inspector footer: `↑/↓ scroll · r refresh · esc back to /security`.
- Command transcript was offered by `ctrl + t`; I did not use it because the warning was already fully visible.
- Other footer keys were not exercised because the case's actions did not request them.

### Expected-result outcomes

- **Every footer key does what it says** — passed for the keys exercised by the case: Up/Down moved selection; Enter opened the review or, when already active, correctly said no change; `i` opened the read-only inspector from picker and review; Esc returned without change. Other footer controls were not exercised.
- **Esc always goes back without changing anything** — passed. Every observed Esc returned to the expected prior screen. The Aggressive review Esc showed `Cancelled. Nothing changed.` and Moderate remained active.
- **Ordinary commands show no spurious warnings such as “could not create PATH aliases: Operation not permitted”** — failed. The ordinary `corbanu vault --help` cell showed exactly that spurious warning.
- **The command is run after approving** — not exercised. No approval prompt was offered; the product classified/executed the read-only command directly. The command itself ran successfully, but the approval action could not be taken.
- **Ambiguity/control note** — no completion timeout, credential prompt, missing required control, or lost observation occurred. The missing approval prompt means the phrase “after approving” was only partially exercised; this does not obscure the directly visible failed warning expectation.

## Verdict

- MOD-33: **failed** because the direct expected result requiring no spurious PATH-alias warning was not met. The other exercised navigation behavior passed.

VERDICT MOD-33: failed
