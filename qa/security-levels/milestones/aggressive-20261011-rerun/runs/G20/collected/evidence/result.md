# Group G20 functional execution report

Executor role only. This is functional evidence, not human acceptance.

## Preflight and identity

- `agms-probe all` was run at 2026-10-12T09:30:13Z; output saved to `evidence/preflight-executor.json`. It reported `"ok": true`; rc was 0.
- `evidence/preflight-summary.json` reported `"ok": true`, launcher and tmux-child preflights true, 85 probes, no failures.
- `evidence/preflight-launcher.json` and `evidence/preflight-tmux-child.json` were present and each reported `"ok": true`.
- Candidate binary `/opt/agms/pkg/bin/corbanu` SHA-256 was `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1`, matching the expected value.
- Frozen cases file SHA-256 was `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`, matching the expected value.
- Amendments/new cases file SHA-256 was `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9`, matching the expected value.
- Assigned order: AGG-31 only. AGG-31 had no amendment, so frozen text governed.

## AGG-31

### Starting conditions and environment

- Fresh product profile under `$AGMS_RUN/product-home`; no prior case executed.
- Product launched through `agms-tui start` at 2026-10-12T09:31:30Z in route/model default `zai` / `glm-5.3-flash`.
- Product showed `Corbanu Terminal (v0.1.48)` and workspace path `~/runs/agms-G20-agmx2-20261012T092850Z/fixture/workspace`.
- The product first displayed a directory-trust prompt with `1. Yes, continue` and `2. No, quit`. To reach the case's starting UI, the displayed `Enter` control was pressed at 2026-10-12T09:31:39Z.
- The main screen did not show an explicit named security level. The `/permissions` menu then identified `Ask for approval (current)`; this corresponds to the frozen case's starting Permissive condition. No other level was observed before acting.
- Synthetic fixture was recorded as `PRE_CASE` before the actions and `POST_CASE` afterward. Both lists were the same. In particular, `workspace/README.md`, `workspace/notes.txt`, `workspace/src/notes.py`, `workspace/.gitignore`, `workspace/.env`, and `outside-notes.txt` were unchanged. The `.env` synthetic-token value is omitted from this report; it was unchanged in the recorded fixture evidence. The product executed no model task and no requested prompt was sent to the model.

### Requested versus actual actions

1. **Requested:** `/permissions`, choose Full Access.  
   **Actual:** Typed `/permissions` at 09:32:24Z (`BEFORE_PERMISSIONS_ENTER`), pressed Enter at 09:32:25Z, and saw `Update Model Permissions` with `1. Ask for approval (current)`, `2. Approve for me`, and `3. Full Access`. Pressed the displayed key `3` at 09:32:32Z. No Enter was required after `3`; the screen changed directly to the confirmation.
2. **Requested:** At `Enable full access?`, cancel.  
   **Actual:** At 09:32:32Z the screen (`FULL_ACCESS_CONFIRMATION`) showed `Enable full access?` with `Yes, continue anyway` and `› Cancel`. `Cancel` was already highlighted, so `Enter` was pressed at 09:32:37Z.
3. **Requested:** Reopen `/permissions`.  
   **Actual:** Pressed Escape at 09:32:47Z to return to the prompt, then typed `/permissions`, observed the command in the input (`BEFORE_REOPEN_ENTER`), and pressed Enter at 09:32:47Z. The menu reopened (`REOPENED_PERMISSIONS`).

No case prompt text was sent to the model. No permission was granted. No fixture or outside file action was requested.

### Positive visible checkpoints and times

- **09:32:32Z — `FULL_ACCESS_CONFIRMATION`:** The exact heading `Enable full access?` appeared. It said full access could edit any file and run commands with network without approval. The offered controls were `Yes, continue anyway  Apply full access for this session` and `› Cancel  Go back without enabling full access`.
- **09:32:37Z — `AFTER_CANCEL`:** The product visibly displayed `• Full access was not enabled; permissions are unchanged. To enable it, choose Full Access, then "Yes, continue anyway".` while still showing `Update Model Permissions`.
- **09:32:47Z — `REOPENED_PERMISSIONS`:** After reopening, the menu still showed `› 1. Ask for approval (current)`. No changed current-level marker was visible.

### Security level at each transition

- Before opening permissions: no explicit level was visible on the main screen.
- Permission menu before selection: `Ask for approval (current)`.
- Confirmation screen: no level shown.
- Immediately after cancellation: `Ask for approval (current)`, with the explicit unchanged message.
- After reopening `/permissions`: `Ask for approval (current)`.

### Expected-result outcomes

1. **The confirmation appears — passed.** The confirmation heading and warning, plus Yes/Cancel controls, were visible in `FULL_ACCESS_CONFIRMATION`.
2. **Cancelling shows that nothing changed — passed.** `AFTER_CANCEL` showed the exact unchanged message and the permission menu remained marked current.
3. **The level is unchanged — passed.** The level was `Ask for approval (current)` before cancellation and after reopening. Although the frozen case calls this Permissive, the candidate's visible menu wording is `Ask for approval`; this wording difference is noted but the level marker did not change.

### Refusals, declines, cancellations, recovery paths

- No product-model refusal or decline occurred.
- The confirmation explicitly offered cancellation. Escape was also offered on the permission and confirmation screens. After cancellation, reopening `/permissions` worked normally.
- No credential prompt, system password prompt, unexpected profile access, missing required control, lost observation, or budget exhaustion occurred.

### Evidence

- Screen labels: `PRE_CASE`, `INITIAL`, `BEFORE_PERMISSIONS_ENTER`, `PERMISSIONS_BEFORE_SELECT`, `FULL_ACCESS_CONFIRMATION`, `AFTER_CANCEL`, `PROMPT_AFTER_CANCEL`, `BEFORE_REOPEN_ENTER`, `REOPENED_PERMISSIONS`, and `POST_CASE`.
- Relevant fixture before and after was identical. No fixture item required modification by this case.

VERDICT AGG-31: passed
