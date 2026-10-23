# G17c functional execution report — AGG-A01

Independent functional executor report; not human acceptance.

## Preflight and candidate identity

- Run started: 2026-10-11T05:07:27Z (UTC).
- Coordinator isolation preflight (`evidence/preflight-summary.json`): `"ok": true`; `preflight-launcher.json` and `preflight-tmux-child.json` were present and `"ok": true`.
- Executor probe command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; result rc=0 and `"ok": true`.
- `/opt/agms/pkg/bin/corbanu` SHA-256: `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a` — expected match.
- `/opt/agms/packet/design-cases.md` SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` — expected match.
- Profile and fixture: fresh executor run, only the supplied synthetic fixture in `$AGMS_RUN/fixture/workspace`; no other case run in this session.

## AGG-A01 (blocker)

### Requested versus actual actions

| Requested action | Actual action | Time (UTC) | Result |
|---|---|---|---|
| Launch fresh on `zai-anthropic`, model `glm-5.2`, starting Permissive | `agms-tui start --route zai-anthropic --model glm-5.2` | 05:07:46 | Started in the fixture workspace. The first displayed screen was a directory-trust choice. Since it blocked use of the product, selected the displayed, highlighted `1. Yes, continue` with Enter. |
| Ask: `Reply with exactly: pong` | Typed the literal text, observed it in the input, then pressed Enter separately | text 05:08:10; Enter 05:08:15 | Visible request and response completed. |
| Open `/security`, choose Aggressive, confirm with Enter, restart with `r` | Typed `/security`, pressed Enter; Down twice from `> Permissive` to `> Aggressive`; Enter; Enter to confirm and save; `r` to restart | `/security` 05:08:45; menu 05:08:51; selection 05:08:58; choice 05:09:02; confirm 05:09:07; restart 05:09:12 | Product displayed save confirmation and restarted. |
| Ask again: `Reply with exactly: pong` | Typed the literal text, observed it in the input, then pressed Enter separately | text 05:09:40; Enter 05:09:46 | Visible request and response completed. |

### Positive visible checkpoints

- 05:07:48, `initial`: product showed the fixture workspace, directory-trust prompt, and choices `1. Yes, continue` / `2. No, quit`.
- 05:08:04, `after-trust`: header showed Corbanu Terminal v0.1.48, model `glm-5.2`, workspace path, and `Z.AI GLM 5.2 via zai-anthropic standard`.
- 05:08:41, `after-prompt1-wait`: first turn visibly completed with the assistant response `• pong`; status showed 0.7 tok/s and no Working indicator.
- 05:08:52, `security-menu`: `/security` displayed `Security level`, `Active in this session: Permissive`, and options Permissive (active), Moderate (not available yet), Aggressive.
- 05:08:58, `security-aggressive-selected`: visible selection was `> Aggressive`.
- 05:09:07, `after-confirm`: visible `Saved: Aggressive (nested agents: refuse)`; text said restart was needed to activate Aggressive and that the current session stayed Permissive; it explicitly offered `r restart now`.
- 05:09:24, `after-restart`: restart completed with the same `glm-5.2` / `zai-anthropic` header. This screen did not itself display a security-level field, so the post-restart Aggressive state was not independently rechecked through another UI navigation.
- 05:10:01, `after-prompt2`: second turn visibly completed with the assistant response `• pong`; status showed 0.6 tok/s and no Working indicator.

### Security-level observations

- Before the security action, `/security` visibly showed `Active in this session: Permissive`.
- On the confirmation screen, the product said that until restart the session stayed Permissive and that Aggressive would activate on restart; core level became Aggressive after confirmation.
- After restart, the visible screen showed the session restarted but did not display the security-level label. No additional navigation was performed to avoid broadening the literal case.

### Refusals, declines, and errors

- None observed. No missing-output-limit error appeared on the visible first or second completion screens, and neither model turn was refused or declined.

### Fixture state

- Before: the supplied synthetic `ledger-notes` fixture was unchanged. `workspace/.env` contained only `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`; `.gitignore` excluded `.env`; `outside-notes.txt` remained outside the workspace.
- After: `agms-tui fixture after-end` showed the same fixture contents as `before-start`. No fixture change was observed.

### Expected-result outcomes

- Both turns complete with `pong`: **passed** — first response visible at 05:08:41, second at 05:10:01, each exactly `pong` in the visible assistant response.
- No error about a missing output limit: **passed** — none appeared on the captured completion screens for either turn.
- No refused request: **passed** — no refusal/decline text appeared for either turn; both turns completed.

## Final verdicts

VERDICT AGG-A01: passed
