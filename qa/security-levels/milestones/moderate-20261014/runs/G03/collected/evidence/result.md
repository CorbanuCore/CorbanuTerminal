# G03 functional execution report

I am the independent functional executor. This is observed product execution, not human acceptance.

## Identity, isolation, and preflight

- Candidate: Corbanu Terminal 0.1.48 at `/opt/agms/pkg/bin/corbanu`.
- Observed SHA-256: `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a` (matched assignment).
- Frozen cases SHA-256: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` (matched).
- Amendments SHA-256: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` (matched).
- Coordinator preflight summary and sidecars were checked before any case. Summary reported `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, 85 probes, `failed: []`, `ok: true`.
- At `2026-10-15T01:29:02Z`, I ran the required `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`. It exited `rc=0` and reported `"ok": true` with no failed probes. Output retained at `preflight-executor.json`.

## Group setup

- Test profile started with only the provided product config (`config.toml`) in `$AGMS_RUN/product-home`.
- Provided fixture project was clean (`git status --short` returned no output) and outside-notes fixture existed.
- Prompt text and Enter were separate `agms-tui` actions throughout. Approval option letters were not typed; Enter activated the highlighted/displayed option.

## MOD-05 · Moderate survives quitting and starting again

### Starting condition setup

Requested by case plus amendment:

1. Start at a fresh profile/project under Permissive.
2. `/security`, choose Moderate, confirm it.
3. Press `r` so Moderate is restarted.

Actual:

| UTC | Action | Positive checkpoint |
|---:|---|---|
| 2026-10-15T01:30:21Z | `agms-tui start` | Product showed the directory trust prompt with option 1 highlighted. Security level not shown on the header. |
| 2026-10-15T01:30:30Z | Enter on displayed trust choice | Main prompt appeared (`agms-tui wait` timed out only because the screen did not contain the searched level text; the main screen itself was visible). |
| 2026-10-15T01:31:49Z | Typed `/security`, then Enter | Picker showed `Active in this session: Permissive`; `Permissive (active)`. |
| 2026-10-15T01:31:56Z | Down to Moderate, then Enter | Review `Switch to Moderate?` appeared. |
| 2026-10-15T01:32:02Z | Enter on `enter confirm and save` | Screen showed `Saved: Moderate`, active protection now, broker keys on next start, and `r restart now`. |
| 2026-10-15T01:32:11Z | Pressed `r` | New empty main screen appeared at `01:32:14Z`; same model/folder. This establishes "Moderate (restarted)" as required by MOD-05. |

Security shown at transitions: Permissive in picker before change; `Saved: Moderate` after confirm; new screen after restart showed no explicit security level in header. No trust prompt on the `r` restart.

### MOD-05 case execution

Requested actions (amended action 2 governs):

1. `/quit`.
2. Relaunch with `agms-tui start` (same profile/project; key from launch environment).
3. `/security`, then `i`.
4. Send `Reply with exactly: pong`.

Actual:

- [2026-10-15T01:32:26Z] Typed `/quit`, observed command listing, then pressed Enter at [2026-10-15T01:32:27Z]. Product session exited.
- [2026-10-15T01:32:47Z] Started `agms-tui start`.
- [2026-10-15T01:33:13Z] Typed `/security` separately and pressed Enter.
- Positive checkpoint [2026-10-15T01:33:14Z, `07-picker-moderate-active`]: picker showed `Active in this session: Moderate`, `Core's level: Moderate`, `Moderate (active)`, and the `g` key.
- [2026-10-15T01:33:23Z] Pressed `i`. Positive checkpoint [2026-10-15T01:33:24Z, `08-inspector-moderate`]: inspector showed `Saved: Moderate [configured: /security]`, `[set] Active this run: Moderate`, and `[set] Model keys: held by the isolated credential broker (broker_model_auth on)`.
- Ambiguity preserved: the inspector's top status simultaneously said `◐ Moderate set; off or not observed: Credential broker`, while the same inspector said model keys were held by the isolated credential broker. I did not infer which top-level status meaning applies.
- [2026-10-15T01:33:33Z] Escape from inspector; [2026-10-15T01:33:34Z] Escape from picker returned to main prompt.
- [2026-10-15T01:33:34Z] Typed `Reply with exactly: pong` separately; [2026-10-15T01:33:35Z] pressed Enter. Product showed `Working (0s • esc to interrupt)`.
- Positive checkpoint [2026-10-15T01:35:23Z, `mod05-pong-poll`]: turn completed and displayed `• pong`; no product security or broker error was visible.

### MOD-05 expected-result status

- Moderate is active: **passed** (picker and inspector both showed it after `/quit` plus `agms-tui start`).
- Inspector shows Saved Moderate, Active this run Moderate, and model keys held by credential broker: **passed** (exact texts above). The conflicting top status is recorded as ambiguity but did not displace the specific required inspector texts.
- The turn completes: **passed** (`pong` displayed).

VERDICT MOD-05: passed

## MOD-25 · Both Z.AI connections work under Moderate

### Starting condition

The immediately preceding MOD-05 flow established Moderate after a real `/quit` and fresh `agms-tui start`; after MOD-05 the session was still Moderate and idle. This satisfies MOD-25's `Moderate (restarted)` starting condition without repeating setup in the same fresh group profile/project.

### Execution

Requested actions:

1. `/model`, select `GLM 5.3 Flash · Z.AI (Claude-style)`, Enter.
2. Send `Reply with exactly: pong`.
3. `/model`, select option 1, Enter.
4. Send `Reply with exactly: pong`.

Actual actions and checkpoints:

| UTC | Action | Observation |
|---:|---|---|
| 2026-10-15T01:35:53Z | Typed `/model`; Enter at 01:35:53Z | Picker visible by 01:35:54Z (`19-model-connections`). |
| 2026-10-15T01:36:02Z | Down to option 2, Enter | Product showed an extra reasoning-level submenu for the Claude-style connection. This was not named by the case. |
| 2026-10-15T01:36:18Z | Up Up to displayed `Low`, Enter | Selection completed. Positive checkpoint at 01:36:19Z: `Model changed to GLM 5.3 Flash · Z.AI (Claude-style) low`; footer showed `via zai-anthropic low`. |
| 2026-10-15T01:36:23Z | Typed pong; Enter at 01:36:24Z | Turn showed `Working`, completed by [2026-10-15T01:38:28Z, `mod25-claude-poll`] with `• pong`; no security or broker error. |
| 2026-10-15T01:38:41Z | Typed `/model`; Enter at 01:38:42Z | Picker showed option 1 `GLM 5.3 Flash · Z.AI (OpenAI-style)` and current option 2 Claude-style. |
| 2026-10-15T01:38:49Z | Up to option 1, Enter at 01:38:50Z | Product again showed reasoning-level submenu. |
| 2026-10-15T01:38:56Z | Enter on displayed `Low` | Positive checkpoint at 01:38:56Z: `Model changed to GLM 5.3 Flash · Z.AI (OpenAI-style) low`; footer showed `via zai low`. |
| 2026-10-15T01:38:56Z | Typed pong; Enter at 01:38:57Z | Turn showed `Working`, completed by [2026-10-15T01:40:20Z, `mod25-openai-poll`] with `• pong`; no security or broker error. |

Navigation ambiguity/deviation: both requested model selections triggered an unrequested reasoning submenu. To complete the literal model selection using displayed controls, I chose `Low` both times. This is recorded as product-required navigation, not as a changed expectation.

No model refusals or declines occurred. No case action changed fixture files. At 2026-10-15T01:40:32Z, `git status --short` was empty and `agms-tui fixture group-end` listed the same non-credential file contents as the supplied fixture; `.env` remained represented only by 98-byte size and SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`.

### MOD-25 expected-result status

- Claude-style turn completes with no security or broker error: **passed**.
- OpenAI-style turn completes with no security or broker error: **passed**.

VERDICT MOD-25: passed

## Final verdicts

VERDICT MOD-05: passed
VERDICT MOD-25: passed
