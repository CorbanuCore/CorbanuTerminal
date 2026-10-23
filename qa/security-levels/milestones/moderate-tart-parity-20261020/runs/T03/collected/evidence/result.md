# Independent functional execution — group T03

- Candidate: `/opt/agms/pkg/bin/corbanu`, Corbanu Terminal 0.1.48.
- Candidate SHA-256: `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6` — verified.
- Candidate `--version`: `corbanu 0.1.48`.
- Frozen packet hashes verified: design cases, amendments, amendments-v3, amendments-v4 all match assignment.
- Coordinator isolation preflight files (`preflight-summary.json`, `preflight-launcher.json`, `preflight-tmux-child.json`) report `ok: true`; executor `agms-probe all` returned `ok: true`, rc 0.
- Only assigned cases run, in order. This report is functional observation only; it is not human acceptance.

## General setup observations

- Fresh-profile setup used for each case: reset `$AGMS_RUN/product-home`, preserving the supplied `config.toml` with all milestone feature flags and Permissive starting behavior; reset `$AGMS_RUN/fixture/workspace` from a fixture copy made before the first case.
- Screen text and paths are recorded from `agms-tui screen`; prompt text and Enter are sent separately.
- Time source: UTC wall clock via `date -u`.

## Case MOD-05 — Moderate survives quitting and starting again

- Status: complete (observations below; outcome at end).
- In force: frozen MOD-05 plus amendments-v2 action 2 relaunch (`agms-tui start`); later amendments do not amend MOD-05. v3/v4 dispositions do not apply to this run.
- Reset timestamp: 2026-10-20T12:24:59Z (profile/project reset; fixture and config initially unchanged).
- Fixture before: git status clean; HEAD `42741cc initial notes`; supplied files present including synthetic `.env`.
- Security level at start: Permissive; at first Moderate confirmation the review says Core becomes Moderate immediately, broker at restart; after restart shown as Moderate.

### Actions: requested versus actual

1. **Requested:** launch fresh product from test project. **Actual:** `agms-tui start` at 12:25:43Z; trust prompt appeared and Enter was sent separately on “1. Yes, continue” (12:25:52Z). Fresh Permissive session started, model `glm-5.3-flash`.
2. **Requested:** establish Moderate (restarted). **Actual:** `/security` sent and Enter sent separately (12:26:13Z); Down then Enter on Moderate (12:26:22Z); review read; Enter confirmed (12:26:30Z); “Saved: Moderate” appeared; `r` pressed (12:26:39Z) and app restarted in same folder/model.
3. **Requested:** `/quit`. **Actual:** prompt text and Enter sent separately at 12:27:02Z; product session closed and the tmux server ended.
4. **Requested:** relaunch with `agms-tui start`. **Actual:** relaunch at 12:27:12Z, same folder/model; no trust prompt.
5. **Requested:** `/security`, then `i`. **Actual:** `/security` text and Enter sent separately at 12:27:20Z; `i` pressed at 12:27:28Z.
6. **Requested:** send `Reply with exactly: pong`. **Actual:** first prompt text was typed while inspector/security navigation was still active and did not create a model turn; I then used displayed Escape controls to return to the main prompt. The literal prompt text was typed and Enter sent separately at 12:28:48Z, producing a real model turn. No model refusal/decline occurred.
7. End-of-case fixture check and session stop after recording.

### Positive visible checkpoints

- `MOD-05-00-start` (12:25:43Z): project trust prompt, option 1 highlighted.
- `MOD-05-01-fresh-start` (12:25:52Z): fresh session, v0.1.48, `glm-5.3-flash`, workspace.
- `MOD-05-02-security-picker-permissive` (12:26:13Z): “Active in this session: Permissive”.
- `MOD-05-03-picker-moderate-highlight` (12:26:22Z): Moderate highlighted.
- `MOD-05-04-moderate-review` (12:26:22Z): full Moderate review; Enter confirms.
- `MOD-05-05-moderate-saved` (12:26:30Z): “Saved: Moderate”; broker keys start at restart; `r` offered.
- `MOD-05-06-after-restart` (12:26:39Z): restarted session, same v0.1.48/model/folder.
- `MOD-05-09-security-after-relaunch` (12:27:20Z): “Active in this session: Moderate”, “Core's level: Moderate”, `Moderate (active)`.
- `MOD-05-10-inspector-top` (12:27:28Z): “Saved: Moderate”; “[set] Active this run: Moderate”; “Model keys: held by the isolated credential broker (broker_model_auth on)”.
- `MOD-05-16-pong-sent` (12:28:48Z): main prompt shows “Reply with exactly: pong” and working state.
- `MOD-05-17-pong-result` (12:28:55Z): reply “pong”; no product/security/broker error.

### Expected-result evaluation

- Moderate active after relaunch: **passed** — security picker showed “Active in this session: Moderate” and `Moderate (active)`.
- Inspector shows Saved: Moderate, Active this run: Moderate, model keys held by credential broker: **passed** — exact visible rows quoted above.
- Turn completes: **passed** — visible reply was “pong”, no error/refusal.
- Offered recovery/continuation: trust “No, quit” at first start; `/security` Esc; Moderate review “esc back, nothing changes”; saved screen offers `r` and close via Enter/Esc; model turn offers Esc to interrupt. No unexpected cancellation needed.
- Fixture after: `agms-tui fixture MOD-05-after` recorded the supplied fixture set; project remained clean/unchanged except normal product/session state outside the project. No test-data mutation required by this case.

## Case MOD-25 — Both Z.AI connections work under Moderate

- Status: complete (observations below; outcome at end).
- In force: frozen MOD-25; no v2/v3/v4 amendment to MOD-25.
- Reset timestamp: 2026-10-20T12:30:02Z (fresh profile/project; supplied config unchanged).
- Fixture before: git status clean; HEAD `42741cc initial notes`; supplied files present including synthetic `.env`.
- Security levels shown: Permissive before selection; immediately after confirming Moderate the review shows Core Moderate and broker keys starting at restart; after `r`, the restarted session was Moderate for both model-connection turns.

### Actions: requested versus actual

1. **Requested:** fresh start and establish Moderate (restarted). **Actual:** launched via `agms-tui start` at 12:30:11Z; trust prompt answered with Enter on “1. Yes, continue” (12:30:18Z). `/security` text and Enter sent separately (12:30:27Z); Down then Enter on Moderate (12:30:36Z); Enter confirmed the review (12:30:44Z); `r` restarted the app (12:30:46Z).
2. **Requested:** `/model`, select `GLM 5.3 Flash · Z.AI (Claude-style)`, Enter. **Actual:** `/model` text and Enter sent separately (12:30:52Z); Down highlighted the named option; Enter selected it (12:31:04Z). The product then required a displayed Reasoning Level choice. Up/Up selected “Low”, and Enter confirmed it (12:31:23Z); the product showed “Model changed to GLM 5.3 Flash · Z.AI (Claude-style) low” and route `zai-anthropic`.
3. **Requested:** send `Reply with exactly: pong`. **Actual:** prompt text and Enter sent separately at 12:31:32Z; result at 12:31:38Z.
4. **Requested:** `/model`, select option 1, Enter. **Actual:** `/model` text and Enter sent separately (12:31:49Z); Up highlighted option 1 and Enter selected it (12:31:58Z); the product again required the displayed Reasoning Level choice. Enter confirmed “Low” (12:32:06Z); product showed “Model changed to GLM 5.3 Flash · Z.AI (OpenAI-style) low” and route `zai`.
5. **Requested:** send `Reply with exactly: pong`. **Actual:** prompt text and Enter sent separately at 12:32:13Z; result at 12:32:20Z.
6. End-of-case fixture check and session stop after recording.

### Positive visible checkpoints

- `MOD-25-00-start` (12:30:11Z): project trust prompt, option 1 highlighted.
- `MOD-25-01-fresh-start` (12:30:18Z): fresh Permissive session, v0.1.48, `glm-5.3-flash`, workspace.
- `MOD-25-02-security-picker-permissive` (12:30:27Z): “Active in this session: Permissive”.
- `MOD-25-04-moderate-review` (12:30:36Z): Moderate review shown and confirmed.
- `MOD-25-05-moderate-saved` (12:30:44Z): “Saved: Moderate”; broker keys start at restart.
- `MOD-25-06-after-restart` (12:30:46Z): restarted session, same v0.1.48/model/folder.
- `MOD-25-07-model-picker` (12:30:52Z): model list displayed.
- `MOD-25-08-claude-style-highlight` (12:31:04Z): `2. GLM 5.3 Flash · Z.AI (Claude-style)` highlighted.
- `MOD-25-11-model-chosen` (12:31:23Z): model changed to Claude-style low; footer “via zai-anthropic low”.
- `MOD-25-13-pong1-result` (12:31:38Z): reply “pong”; no security/broker error.
- `MOD-25-15-openai-style-highlight` (12:31:58Z): option 1, `GLM 5.3 Flash · Z.AI (OpenAI-style)`, highlighted.
- `MOD-25-17-openai-style-selected` (12:32:06Z): model changed to OpenAI-style low; footer “via zai low”.
- `MOD-25-19-pong2-result` (12:32:20Z): second reply “pong”; no security/broker error.

### Expected-result evaluation

- First turn completes via Claude-style Z.AI connection: **passed** — Claude-style route `zai-anthropic` visibly selected; reply “pong”; no error/refusal.
- Second turn completes via option 1/OpenAI-style Z.AI connection: **passed** — route `zai` visibly selected; reply “pong”; no error/refusal.
- No security or broker error: **passed** — both result screens contain no product warning or refusal.
- No model refusal/decline occurred in either turn.
- Reasoning-level submenu is recorded as an unavoidable displayed step because the product requires choosing Low/High/More after either model selection; “Low” was chosen for both, and the case does not judge reasoning level.
- Offered recovery/continuation: trust “No, quit”; `/security` Esc; saved-Moderate `r`; model-picker Esc; reasoning-picker Esc; turn Esc to interrupt. None unexpectedly needed.
- Fixture after: `agms-tui fixture MOD-25-after` recorded the supplied fixture set; project remained clean/unchanged. No test-data mutation required by this case.

## Final verdicts

- `VERDICT MOD-05: passed`
- `VERDICT MOD-25: passed`

Functional result only; not a claim of human acceptance.

## Evidence state

- MOD-05 session stopped at 2026-10-20T12:29:44Z; MOD-25 session stopped at 2026-10-20T12:32:29Z.
- This report is stored at `$AGMS_RUN/evidence/result.md`; screen labels are recorded by `agms-tui`; fixture records are at `$AGMS_RUN/evidence/MOD-05-fixture-after.txt` and `$AGMS_RUN/evidence/MOD-25-fixture-after.txt`.
- Executor preflight is at `$AGMS_RUN/evidence/preflight-executor.json`; coordinator preflight files were beside it and each reported `ok: true`.
