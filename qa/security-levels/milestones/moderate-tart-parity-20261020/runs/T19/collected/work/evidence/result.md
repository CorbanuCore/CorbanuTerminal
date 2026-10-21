# T19 execution report

Executor: independent functional executor (not implementer or reviewer).  
Candidate: `/opt/agms/pkg/bin/corbanu` (Corbanu Terminal 0.1.48).  
Case order assigned and run: MOD-23 only.

## Isolation and identity preflight

- UTC 2026-10-20T16:51:37Z: ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`.
- Actual: rc=0; JSON reports `"ok": true`; 49 individual probes shown, none failed. Coordinator summary reports `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `failed: []`, `probe_count: 99`, `ok: true`; the two `preflight-*.json` files beside it were present (`preflight-launcher.json`, `preflight-tmux-child.json`).
- Frozen packet and candidate identity:
  - `design-cases.md` SHA-256 `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` (expected and observed match).
  - `amendments.md` SHA-256 `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` (match).
  - `amendments-v3.md` SHA-256 `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3` (match).
  - `amendments-v4.md` SHA-256 `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae` (match).
  - `corbanu` SHA-256 `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6` (match).
  - `corbanu --version` output: `corbanu 0.1.48`; the TUI header showed `v0.1.48`.

## MOD-23 · Moderate is not saved without the untrusted-content labels

### Starting condition setup

- UTC 2026-10-20T16:54:00Z: confirmed a fresh product home containing only `config.toml`; set `[features] source_envelopes = false`; all other supplied feature flags unchanged. No `[security]` level was set, so the configured launch state was Permissive. Later product behavior changed the same config file (see below).
- UTC 2026-10-20T16:54:00Z: fixture `initial-before` recorded. Project and outside file were unchanged from the supplied synthetic fixture; workspace Git status was clean. `.env` was recorded only as 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`; `outside-notes.txt` content was `outside-the-project: synthetic file`.
- Requested start: plain `agms-tui start` (default route `zai`, default model `glm-5.3-flash`). Actual header/model: `glm-5.3-flash`, `zai default`.

### Requested versus actual actions and checkpoints

- Requested: start the product. Actual: at 16:54:34Z, screen `00-first-start-trust` displayed the directory-trust prompt with `› 1. Yes, continue` selected. At 16:54:39Z, pressed Enter and screen `01-initial-permissive` showed the normal v0.1.48 chat UI. Security level shown at the first transition: none on the chat screen; `/security` immediately afterward showed `Active in this session: Permissive`.
- Requested action 1: `/security`, ↓, Enter. Actual: at 16:55:19–20Z, typed `/security`, then pressed Enter separately. Screen `02-security-picker-before-down` showed `Active in this session: Permissive`; `Permissive  (active)` was selected. At 16:55:26Z, pressed Down; screen `03-moderate-highlight-source-envelopes-off` selected `Moderate`. Pressed Enter; at 16:55:27Z, screen `04-moderate-review-no-labels` displayed the full Moderate review.
- Positive checkpoint: The review screen ended with `enter confirm and save · esc back, nothing changes · i inspect`. It did **not** mention missing `source_envelopes`/untrusted-content labels, did not say Moderate needs them, and did not say how to turn them on. This is a failed expected result.
- Requested action 2: press Enter twice. Actual: at 16:55:35Z, pressed the first Enter. Screen `05-after-first-enter` showed `Saved: Moderate`; it did not block the save. At 16:55:42Z, pressed the second Enter; screen `06-after-second-enter` showed the normal chat UI. The first Enter had already closed the save screen.
- Requested action 3: Esc. Actual: at 16:55:45Z, pressed Escape; screen `07-after-escape` remained the normal chat UI.
- Requested action 4: `/security`, `i`. Actual: at 16:55:50–51Z, typed `/security`, pressed Enter separately, and screen `08-security-after-moderate-save` showed `Active in this session: Permissive`, `Saved: Moderate`, and `Core's level: Moderate`. At 16:55:53Z, pressed `i`; screen `09-inspector-after-attempt` showed `Saved: Moderate [configured: /security]`, `Core config: Permissive`, `Active this run: Permissive`, and `Live policy: Moderate in force`. It also showed `Model keys: Corbanu reads provider keys itself (broker_model_auth off)`. This is a failed expected result: the inspector did not show `Saved: Permissive`.
- Requested action 5: relaunch. Actual: at 16:56:09Z, used `agms-tui stop`, then started plain `agms-tui` again. At 16:56:15Z, screen `10-relaunched` showed v0.1.48. At 16:56:21–23Z, `/security` showed screen `11-security-after-relaunch` with `Active in this session: Moderate`, `Core's level: Moderate`, and `Moderate  (active)`. At 16:56:24Z, `i` showed screen `12-inspector-after-relaunch` with `Saved: Moderate`, `Active this run: Moderate`, and `Live policy: Moderate in force`; `Core config: Permissive` remained in the inspector. This is a failed expected result: after relaunch it was not Permissive.
- Fixture/status after: at 16:56:42Z, `agms-tui fixture after-mod23` showed the same visible test-project contents and `.env` size/hash as before; workspace Git status remained clean and `outside-notes.txt` was still `outside-the-project: synthetic file`. The product then stopped cleanly.
- Additional partial observation: after execution, `$AGMS_RUN/product-home/config.toml` contained `source_envelopes = true`, although the tester had set it to `false` before launch and the save screen said `Your config.toml is not modified.` The tester did not alter this line during the case after the initial setup.

### Model refusals / declines

- None. No model turn or model-visible request was sent in this case.

### Prompts / interruptions

- The expected first-start directory-trust prompt appeared; Enter selected the displayed `Yes, continue` option.
- No credential prompt, system password prompt, unexpected profile access, missing required control, lost observation, or budget exhaustion occurred. No cancellation path beyond the displayed `esc back, nothing changes` was offered during the attempted Moderate save.

### Expected-result dispositions

1. **The review says Moderate needs the untrusted-content labels (`source_envelopes`) and how to turn them on.**  
   **FAILED.** The complete review screen at 16:55:27Z (`04-moderate-review-no-labels`) contained no mention of `source_envelopes`, untrusted-content labels, or how to enable them.
2. **There is no "enter confirm" option, and Enter saves nothing.**  
   **FAILED.** The review offered `enter confirm and save`. Enter saved and displayed `Saved: Moderate`.
3. **The inspector shows Saved: Permissive.**  
   **FAILED.** After the attempted save, the inspector showed `Saved: Moderate [configured: /security]`.
4. **It is still Permissive after relaunch.**  
   **FAILED.** After relaunch, `/security` showed `Active in this session: Moderate`, and the inspector showed `Saved: Moderate`, `Active this run: Moderate`, and `Live policy: Moderate in force`.

### Outcome

- VERDICT MOD-23: failed

The tester does not claim human acceptance.
