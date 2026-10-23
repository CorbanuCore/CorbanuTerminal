# G17 functional execution report

Scope: assigned cases AGG-27, AGG-A01 only. This is executor observation, not human acceptance.

## Preflight and identity

- 2026-10-13: ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=0`; rc was `0`.
- Executor preflight JSON reports `"ok": true`.
- Coordinator `preflight-summary.json` reports `"ok": true`; `preflight-launcher.json` and `preflight-tmux-child.json` also report `"ok": true` (85 probes total per summary).
- Candidate binary SHA-256 observed: `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`.
- Frozen cases SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Amendments/new-cases SHA-256 observed: `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9`.
- Package version was independently observed in preflight as `corbanu 0.1.48`.

## Case setup notes

- Used only the supplied fixture under `$AGMS_RUN/fixture/workspace`.
- Product home for this run is `$AGMS_RUN/product-home`; it contained only the supplied `config.toml` before the first launch. The case workspace had the supplied project files and `.git`. No prior case history or source was read.

## AGG-27

### Requested versus actual actions

| Requested | Actual |
|---|---|
| Start fresh profile and project; make Aggressive active. | At 2026-10-13T20:53:20Z launched package `0.1.48` via `agms-tui start` in the supplied workspace. Trusted the displayed directory prompt with Enter. `/security` showed `Active in this session: Permissive` (S27-security-menu, 20:53:34Z). Selected Aggressive with Down, Down, Enter, confirmed with Enter; product said `Saved: Aggressive (nested agents: refuse)` and offered `r restart now` (S27-aggressive-confirm). Pressed `r`; it restarted (20:53:52Z). |
| On `zai` ask `Reply with exactly: pong`. | Typed prompt at 20:53:59Z, pressed Enter separately. Completion `• pong` was visible by 20:54:08Z (S27-zai-pong-complete). |
| `/model`, choose `GLM 5.3 Flash · Z.AI (Claude-style)` (`zai-anthropic`), ask same. | `/model` offered that option. Selected it with Down + Enter (S27-model-option2-selected). The following reasoning menu displayed Low/High/More reasoning; selected Low with Up, Up, Enter. Product said `Model changed to GLM 5.3 Flash · Z.AI (Claude-style) low`; status line showed `via zai-anthropic low` (S27-after-claude-reasoning). Typed same prompt at 20:54:38Z and pressed Enter separately. Completion `• pong` was visible by 20:54:48Z (S27-claude-pong-t5). |
| Restart and repeat on each route. | Stopped/quit with `agms-tui stop`, relaunched `agms-tui start --route zai` at 20:54:58Z. Status line showed `via zai low`. Typed the same prompt at 20:55:04Z and pressed Enter separately; `• pong` was visible by 20:55:09Z (S27-restart-zai-pong). Stopped again and relaunched `agms-tui start --route zai-anthropic` at 20:55:12Z. Status line showed `via zai-anthropic low`. Typed same prompt at 20:55:18Z and pressed Enter separately; `• pong` was visible by 20:55:23Z (S27-restart-anthropic-pong). |
| Record wall-clock time of each pong. | `zai`: 20:54:08Z. `zai-anthropic` after `/model`: 20:54:48Z. Restarted `zai`: 20:55:09Z. Restarted `zai-anthropic`: 20:55:23Z. These are screen-observation times, not inferred relay times. |
| Verify level after restarts. | Opened `/security` after the final restarted turn: `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive (active)` (S27-security-after-restart, 20:55:23Z). |

### Positive visible checkpoints

- S27-security-menu: initial active level Permissive.
- S27-aggressive-confirm: Aggressive saved confirmation and restart control.
- S27-zai-pong-complete: exact `pong`, provider status `Z.AI GLM 5.3 Flash via zai`.
- S27-after-claude-reasoning: provider status `via zai-anthropic low`; model-change notice.
- S27-claude-pong-t5: exact `pong` on `zai-anthropic`.
- S27-restart-zai-pong: exact `pong` after restart on `zai`.
- S27-restart-anthropic-pong: exact `pong` after restart on `zai-anthropic`.
- S27-security-after-restart: active and core level Aggressive after restart.

### Fixture observations

- No case-specific project mutation was requested or performed. The visible turns produced no command approval prompt or file-change notice.

### Expected-result outcomes

1. **Every turn completes with `pong` — passed.** All four requested model turns visibly produced `• pong` (see table/times).
2. **No broker or refusal error appears — passed.** No broker or refusal error was visible on any completion screen. The added route-evidence expectation depends on the independent relay check, not directly visible here.
3. **Model and provider choices are unchanged by the level — passed.** The `/model` screen offered OpenAI-style and Claude-style Z.AI connections while Aggressive was saved but before restart. The status line remained `glm-5.3-flash` / `zai` or `zai-anthropic` through the turns and restarts. The added relay-path check depends on the independent evidence check.
4. **Added: relay shows `/messages` for zai-anthropic turns and `/chat/completions` for zai turns — not exercised by executor.** The relay log is not visible; recorded UTC pong times above are supplied for the independent check.

### Ambiguities/notes

- During the `/model` switch, a reasoning-level menu appeared for Claude-style. The case did not specify a reasoning level, so Low was selected because it was the first displayed choice; this preserved a visible completion without changing model or provider.
- The two restart relaunches inherited the previously selected `low` reasoning indicator. The model remained `glm-5.3-flash`; no model/provider change was made except the requested `/model` switch to zai-anthropic.

## AGG-A01

### Starting-state setup

- At 2026-10-13T20:56:28Z, stopped the AGG-27 session and made a fresh product profile by moving the prior `product-home` aside inside the run directory and recreating it with only the supplied `config.toml`.
- Launched with `agms-tui start --route zai-anthropic --model glm-5.2`.
- The workspace was unchanged from the supplied copy: `git status --short` produced no output. Fixture hashes were saved before and after (`a01-fixture-before.sha256`, `a01-fixture-after.sha256`); the diff was empty.
- No trust prompt appeared on this relaunch; the main screen appeared directly (S-A01-initial).

### Requested versus actual actions

| Requested | Actual |
|---|---|
| Fresh profile, Permissive, launched on `zai-anthropic` with `glm-5.2`. | Initial screen showed `model: glm-5.2 low` and status `Z.AI GLM 5.2 via zai-anthropic standard` (S-A01-initial, 20:56:28Z). `/security` showed `Active in this session: Permissive` and `Permissive (active)` (S-A01-security-menu, 20:56:42Z). |
| Ask `Reply with exactly: pong`. | Typed at 20:56:37Z and pressed Enter separately. Completion `• pong` was visible by 20:56:39Z (S-A01-permissive-pong). |
| Open `/security`, choose Aggressive, confirm with Enter, restart with `r`. | `/security` showed Permissive. Selected Aggressive with Down, Down, Enter; confirmation screen appeared (S-A01-aggressive-confirm). Pressed Enter; product showed `Saved: Aggressive (nested agents: refuse)` and `Press r to restart now` (S-A01-save-confirmation, 20:56:54Z). Pressed `r`; new main screen appeared by 20:56:56Z (S-A01-restarted-initial). |
| Ask same prompt after restart. | Typed at 20:57:00Z and pressed Enter separately. Completion `• pong` was visible by 20:57:03Z (S-A01-aggressive-pong). |
| Observe level after restart. | Opened `/security` after the second turn: `Active in this session: Aggressive`, `Core's level: Aggressive`, `Aggressive (active)` (S-A01-security-after-restart, 20:57:03Z). |

### Positive visible checkpoints

- S-A01-initial: requested launch route/model.
- S-A01-security-menu: active level Permissive before the first turn.
- S-A01-permissive-pong: exact `pong` before restart.
- S-A01-aggressive-confirm: Aggressive confirmation prompt.
- S-A01-save-confirmation: Aggressive saved and restart control shown.
- S-A01-restarted-initial: same model/route after restart.
- S-A01-aggressive-pong: exact `pong` after restart.
- S-A01-security-after-restart: active/core level Aggressive after restart.

### Fixture observations

- Before and after, `git status --short` was empty and the fixture SHA-256 inventory was unchanged. No project files were edited and no command approval was requested.

### Expected-result outcomes

1. **Both turns complete with `pong` — passed.** Both requested turns visibly produced `• pong`.
2. **No error about a missing output limit and no refused request — passed.** No such error, refusal, broker error, or declined request appeared on either completion screen.

### Ambiguities/notes

- The fresh launch showed the model effort as `low` in the header and `standard` in the status line. Both remained unchanged through the case; neither phrase was treated as an error.

## Final verdicts

VERDICT AGG-27: passed
VERDICT AGG-A01: passed
