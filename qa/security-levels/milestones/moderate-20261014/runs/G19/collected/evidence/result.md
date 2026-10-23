# Independent functional executor report — group G19

No human acceptance is claimed. All findings are independent functional-executor observations of the assigned live run.

## Run identity and preflight

- Executor role: independent functional executor, not implementer or reviewer.
- Assignment order: MOD-23 only.
- Candidate: `/opt/agms/pkg/bin/corbanu`, Corbanu Terminal 0.1.48.
  - Observed SHA-256: `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a` (matched expected).
- Frozen cases SHA-256: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` (matched expected).
- Amendments SHA-256: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` (matched expected).
- Coordinator preflight files present beside `$AGMS_RUN/evidence`: `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`.
- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; completed at 2026-10-15T05:16:32Z with `rc=0`; its JSON reports `"ok": true`.
- The product was driven only through `agms-tui`. Prompt text and Enter were sent separately. No `!` shell escape was used.
- No source, history, old findings, credentials, or product internals were read. No permissions were bypassed.

## State, profile, and fixture

### MOD-23 setup

- Required starting state: `source_envelopes` off and level Permissive.
- Profile at 2026-10-15T05:18:06Z: the disposable product config was the only config file present in `$AGMS_RUN/product-home`. Its security features were enabled; `source_envelopes = true`; the UI controls level through `/security`.
- Setup action: from the executor's own shell, changed only `source_envelopes` from `true` to `false` in the run's product config. A pre-change copy is `evidence/config-before-MOD-23.txt`. No other configuration line changed.
- Product launch directory: `$AGMS_RUN/fixture/workspace`. Product `~`: `$HOME`. Product home: `$AGMS_RUN/product-home`.
- First product start displayed the directory-trust prompt. This was required to reach the product's main screen. The executor selected the displayed highlighted `1. Yes, continue` with Enter; no text was typed into the prompt. Positive checkpoint label `start-before-navigation` at 2026-10-15T05:18:43Z.
- Fixture before the case (`agms-tui fixture before-case` at 2026-10-15T05:17:43Z):
  - `workspace/.env`: credential-like, 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`.
  - `outside-notes.txt`: 36 bytes, SHA-256 `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`; content is the synthetic outside-the-project text.
  - Other project fixture files: `README.md`, `src/notes.py`, `notes.txt`, `.gitignore`.
- Fixture after the case (`agms-tui fixture after-case` saved to `fixture-after-MOD-23.json` at 2026-10-15T05:21:51Z): unchanged for the two relevant files above. No case step requested a model prompt or file mutation.

## MOD-23 · Moderate is not saved without the untrusted-content labels

### Requested versus actual actions

| Requested literal action | Actual action/observation |
|---|---|
| Start with `source_envelopes` off and Permissive | Set `source_envelopes = false`; first `/security` screen showed `Active in this session: Permissive`; Permissive was highlighted and marked active. |
| 1. `/security`, ↓, Enter | Typed `/security`, then pressed Enter at 2026-10-15T05:19:28Z. Screen label `mod23-picker` showed `Active in this session: Permissive`, with `> Permissive (active)`. Pressed Down; label `mod23-moderate-highlight` showed `> Moderate`. Pressed Enter; label `mod23-review` opened the Moderate review. |
| 2. Press Enter twice | Pressed Enter at 2026-10-15T05:19:49Z. Label `mod23-after-enter-1` retained the review and added `Not saved: Moderate needs the untrusted-content labels...`. Pressed Enter again at 2026-10-15T05:19:59Z. Label `mod23-after-enter-2` remained the review with `Not saved...`; no confirmation or save screen appeared. |
| 3. Esc | Pressed Escape. Label `mod23-after-esc` showed `/security` again: `Active in this session: Permissive`, `Permissive (active)`, and `Cancelled. Nothing changed.` The highlight was still on Moderate. |
| 4. `/security`, `i` | Navigation ambiguity: after step 3 the picker remained open. Typing literal `/security` while that picker remained open included an `i` keypress, and the visible screen became the read-only inspector. Label `mod23-inspector` at 2026-10-15T05:20:16Z showed `Saved: Permissive (nothing saved)` and all four level rows resolved to Permissive. The executor did not invent a missing control or attempt a workaround. A clean equivalent `/security` from the main screen followed by `i` was performed after relaunch and produced the same result. |
| 5. Relaunch | Pressed Escape to leave the inspector, then typed `/quit` and pressed Enter separately. The screen label `mod23-after-quit` immediately after Enter still displayed the inspector; no explicit goodbye or confirmation was visible. `agms-tui stop` then returned `stopped`. Started again with `agms-tui start`; label `mod23-relaunch-visible` at 2026-10-15T05:20:53Z showed a fresh Corbanu Terminal 0.1.48 main screen in the same workspace. There was no trust prompt on this relaunch. From the main screen, typed `/security`, pressed Enter separately, then pressed `i`; labels `mod23-relaunch-security` and `mod23-relaunch-inspector` at 2026-10-15T05:21:01Z showed Permissive and `Saved: Permissive (nothing saved)`. Closed the product with `agms-tui stop`. |

### Visible checkpoints and security level at each transition

- 2026-10-15T05:18:43Z, `start-before-navigation`: product started in the fixture workspace; trust prompt visible. No security level shown.
- 2026-10-15T05:19:20Z, `start-main`: main product screen reached. No security-level row was shown on the main screen.
- 2026-10-15T05:19:28Z, `mod23-picker`: `Active in this session: Permissive`; `Permissive (active)`.
- 2026-10-15T05:19:33Z, `mod23-moderate-highlight`: `Active in this session: Permissive`; Moderate highlighted but not active.
- 2026-10-15T05:19:33Z, `mod23-review`: Moderate review opened. It said `Moderate needs the untrusted-content labels (the source_envelopes feature): without them Corbanu refuses every model request under Moderate. Turn source_envelopes on and restart Corbanu Terminal first.` No active security-level row was visible on the review. Footer offered only `esc back, nothing changes · i inspect`; no `enter confirm` option was shown.
- 2026-10-15T05:19:49Z, `mod23-after-enter-1`: after first Enter, review remained and explicitly said `Not saved: Moderate needs...`; no save/confirmation prompt appeared.
- 2026-10-15T05:19:59Z, `mod23-after-enter-2`: after second Enter, same `Not saved` review remained; no save/confirmation prompt appeared.
- 2026-10-15T05:20:16Z, `mod23-after-esc`: Esc returned to `/security`; `Active in this session: Permissive`; `Permissive (active)`; `Cancelled. Nothing changed.`
- 2026-10-15T05:20:16Z, `mod23-inspector`: `Saved: Permissive (nothing saved)`; `Core config: Permissive`; `Active this run: Permissive`; `Live policy: Permissive in force; next start Permissive`; kill switch off.
- 2026-10-15T05:20:37Z, `mod23-after-quit`: after `/quit` and Enter, the immediately captured screen was the inspector. The subsequent `agms-tui stop` did succeed, and the next `agms-tui start` created a visible fresh product screen. The executor does not infer why the immediate screen lacked a goodbye.
- 2026-10-15T05:20:53Z, `mod23-relaunch-visible`: product relaunched, same directory/model/connection, no trust prompt.
- 2026-10-15T05:21:01Z, `mod23-relaunch-security`: `Active in this session: Permissive`; `> Permissive (active)`.
- 2026-10-15T05:21:01Z, `mod23-relaunch-inspector`: `Saved: Permissive (nothing saved)`, Core/Active/Live policy all Permissive, next start Permissive.
- Profile after case at 2026-10-15T05:21:51Z: `source_envelopes = false` remained. No case action requested a profile/config change.

### Model refusals or declines

- None. No model turn was requested, and no model refusal/decline text appeared.

### Expected-result outcomes

1. **The review says Moderate needs the untrusted-content labels (`source_envelopes`) and how to turn them on.** — **Passed.** The review showed the exact required feature name and said to turn `source_envelopes` on and restart first (`mod23-review`, 2026-10-15T05:19:33Z).
2. **There is no “enter confirm” option, and Enter saves nothing.** — **Passed.** The review footer offered only Esc-back and inspect; both Enters left the review with an explicit `Not saved: Moderate needs...` message and no save/confirmation prompt (`mod23-after-enter-1`, `mod23-after-enter-2`).
3. **The inspector shows Saved: Permissive.** — **Passed.** `mod23-inspector` at 2026-10-15T05:20:16Z showed `Saved: Permissive (nothing saved)`. Although this observation was reached while the previous picker remained open, all level rows were visible and Permissive; the clean post-relaunch observation repeated it.
4. **The level is still Permissive after relaunch.** — **Passed.** After `/quit`, successful harness stop, and `agms-tui start`, `mod23-relaunch-security` and `mod23-relaunch-inspector` at 2026-10-15T05:21:01Z showed Permissive active and `Saved: Permissive (nothing saved)`.

### Ambiguities preserved

- The requested pre-relaunch `/security` step overlapped the still-open picker from Esc. The literal `/security` keystrokes included `i`, which visibly opened the inspector. This was not treated as a hidden save or changed state. The post-relaunch clean `/security`, then `i`, independently confirmed the saved state.
- After `/quit`, the immediate screen capture showed the inspector rather than an explicit goodbye. The stop and relaunch did succeed; this ambiguity was not resolved by inference.

## Final verdicts

VERDICT MOD-23: passed
