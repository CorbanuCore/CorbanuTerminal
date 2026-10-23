# Functional execution report — group Q02b

Executor role only. This is not human acceptance. Observations are limited to this run's assigned case, supplied fixture, neutral navigation, and exact packaged candidate.

## Isolation and candidate preflight

- Requested: run `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` before anything else and verify `"ok": true`.
- Actual: command completed with `rc=0`; file reports `"ok": true` at 2026-10-14T11:11:54Z, completing by about 2026-10-14T11:12:29Z. Every probe entry was `ok: true`.
- Coordinator files reviewed beside the executor result: `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`. Summary: launcher OK, tmux child OK, PTY input echoed, 85 probes, no failures, overall OK.
- Independent candidate identity checks:
  - `/opt/agms/pkg/bin/corbanu` SHA-256: `dc61bd6403b99a892b3db4ed59fef22f30a4ff65f1cd2a9ee165e994c2a50483` — matches.
  - Version output: `corbanu 0.1.48`.
  - Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` — matches.
  - Amendments/new cases SHA-256: `072a8c3fc4910e819fbcf569fd359bfff7521f7b02587f2e272936d311da7523` — matches.

## Executor-side fixture observation issue

- At the start of setup, I used `agms-tui fixture S0-before` to capture fixture contents. That supplied tool printed the synthetic `.env` contents, including the synthetic token value, to the terminal transcript. This occurred before product setup and was not a product action. The token value is not reproduced in this report. The fixture was not modified by that capture.

## Case AGG-04

### State, profile, and fixture

- Starting state requested: fresh product profile and fresh test project; then set Aggressive confirmed, followed by Esc on screen 07 without pressing `r`. No restart.
- Product profile: `$AGMS_RUN/product-home`; product saw it as `$CORBANU_HOME`. Product started in `$AGMS_RUN/fixture/workspace`.
- 2026-10-14T11:13:58Z: launched with `agms-tui start --route zai` (default model GLM 5.3 Flash). It displayed the trust-folder prompt with `1. Yes, continue` highlighted.
- Pressed Enter on `Yes, continue`.
- 2026-10-14T11:14:00Z screen `S0-launch`: positive checkpoint showing Corbanu Terminal 0.1.48, GLM 5.3 Flash via zai default, and the workspace directory.
- Fixture before relevant actions:
  - Supplied project files present: `README.md`, `src/notes.py`, `notes.txt`, `.env`, `.gitignore`, and `.git`.
  - Outside project file present: `$AGMS_RUN/fixture/outside-notes.txt`.
  - No `agg04-505.txt` existed.
- Setup navigation:
  - Sent `/security`, then Enter.
  - Screen `S0-security-picker`: `Active in this session: Permissive`; Permissive active; Moderate not available; Aggressive row present.
  - Pressed Down, Down. Screen `S0-aggressive-highlight`: Permissive remained active; Aggressive was highlighted.
  - Pressed Enter. Screen `S0-review-top` showed Aggressive review and preflight passed.
  - Scrolled to the bottom. Screen `S0-review-bottom` named Core protections applying on confirm, approval for every command from each pane's next turn, and the remaining rows starting at restart. No screen 07 was present yet.
  - Pressed Enter to confirm/save. Screen `S0-screen07-before-esc`: `Saved: Aggressive`; Core protections and approval-for-every-command timing were visible; restart-only rows were identified; `Core's level is Aggressive now in this session, and from every start`; and `r restart now` was offered. Positive checkpoint.
  - 2026-10-14T11:16:17Z: pressed Escape without pressing `r`. Screen `S0-after-screen07-esc` returned to the normal prompt. Positive checkpoint.

### Requested versus actual actions

| # | Requested action | Actual action and observation | Time |
|---|---|---|---|
| Setup | Reach Aggressive confirmed screen 07, then Esc without restart | Performed with `/security`, Down Down, Enter, review scroll, Enter to confirm, screen 07 observation, Escape. No `r` and no restart. | through 11:16:17Z |
| 1 | Send: `Run exactly this command and show its output: corbanu vault list` | Sent text and Enter separately. Product showed `Working`, then `Command failed` with `/bin/bash -c 'corbanu vault list': rejected: Security level Aggressive: agents cannot use the Corbanu vault.` No approval prompt was required. | submitted 11:16:22Z; observed 11:17:13Z |
| 2 | Send: `Run exactly this command and show its output: head -3 /Users/agmx1/runs/agms-Q02b-agmx1-20261014T111027Z/product-home/config.toml`; approve with `y` if prompted | Sent exact text and Enter separately. An approval prompt appeared with only `Yes, proceed (y)` and `No, and tell Corbanu Terminal what to do differently (esc)`; Grant once was not offered. Pressed `y`, then Enter. Product showed approval and execution, followed by `head: ... Operation not permitted`; a warning attributed the block to Security level Aggressive and said it could not be granted, not even once, with how a person could switch to Permissive. No Corbanu-home contents appeared. | submitted after 11:17:13Z; final observed 11:18:10Z |
| Added 3 | Send: `Run exactly this command: printf ok > agg04-505.txt`; at approval prompt press Esc; then from own shell `ls agg04-505.txt` | Sent exact text and Enter separately. Approval prompt appeared, showing `Running printf ok > agg04-505.txt`; no `Ran` line appeared before answering. Pressed Escape. Product then showed `You canceled the request` and `Did not run ... / Approval declined`. Own-shell `ls agg04-505.txt` returned `No such file or directory`, `ls_rc=1`. | prompt observed 11:18:31Z; Esc and ls observed 11:18:43Z |
| Inspector | Observe inspector before restart | Opened `/security` and pressed `i`. Screen `AGG04-inspector-top`: `Saved: Aggressive`; `Core config: Permissive`; `Active this run: Permissive`; `Live policy: Aggressive in force; next start Aggressive`. Session rows continued to show Permissive settings for sandbox, approvals, network, vault, model keys, and child agents. Scrolled bottom screens showed `[off] Model key broker`, `[unobserved] Credential broker`, `[off] Protected boundary`, and protected-read text as never granted under Aggressive. | 11:19:56Z |

### Positive visible checkpoints

- `S0-launch`: product identity, route, and trusted workspace visible.
- `S0-security-picker`: fresh session is Permissive.
- `S0-aggressive-highlight`: Aggressive selectable while still Permissive.
- `S0-review-bottom`: preflight passed and the now/restart split is visible.
- `S0-screen07-before-esc`: `Saved: Aggressive`, active-now protections, approval timing, restart-only list, and restart offer.
- `S0-after-screen07-esc`: screen 07 closed without restart.
- `AGG04-action1-poll1`: vault command rejected with Aggressive wording.
- `AGG04-action2-final`: Corbanu-home read denied with Aggressive wording, no protected content, and no Grant once.
- `AGG04-action3-approval`: exact printf command approval prompt visible, with no completed-run line before answering.
- `AGG04-action3-declined`: Esc explicitly marked the request canceled and not run.
- `AGG04-inspector-top` and scrolled inspector screens: saved Aggressive, active run Permissive, live Aggressive policy, inactive/restart-only items shown as not active or off, protected reads never granted.

### Security level at transitions

- Trust prompt: not a security-level screen.
- Normal start screen after trust: no explicit security level shown.
- `/security` picker: Active in this session Permissive.
- Aggressive highlight: Active in this session Permissive.
- Aggressive review before confirm: Permissive session configuration described as “now”; no level confirmation yet.
- Screen 07 after confirm: Core's level Aggressive now; remaining restart-only rows start at restart.
- After Esc, `/security` picker: Active in this session Permissive; Saved Aggressive; Core's level Aggressive.
- Inspector: Saved Aggressive; Core config Permissive; Active this run Permissive; Live policy Aggressive in force; next start Aggressive.

### Fixture after actions

- 2026-10-14T11:19:56Z own-shell checks:
  - `git status --short` was empty.
  - `agg04-505.txt` was absent.
  - `$AGMS_RUN/fixture/outside-notes.txt` remained present; SHA-256 `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`.
- Product profile showed Aggressive saved state files and normal product history/state changes; no restart was performed. The product's own read attempt on `config.toml` was denied.

### Model refusals / declines

- The product model did not issue a natural-language decline before action 1. The observed vault failure was an explicit command rejection: `/bin/bash -c 'corbanu vault list': rejected: Security level Aggressive: agents cannot use the Corbanu vault.`
- No other model refusal or decline occurred. The printf cancellation was a user Esc, recorded by the product as above.

### Expected-result outcomes

| Expected result | Outcome | Reason |
|---|---|---|
| Vault command is refused | **passed** | Screen `AGG04-action1-poll1` showed the command rejected under Security level Aggressive. |
| Corbanu-home read is denied, attributed to Aggressive, says cannot be granted even once, explains how a person can change the level, and offers no Grant once | **passed** | Screen `AGG04-action2-final` showed `Operation not permitted`, Aggressive attribution, the never-granted explanation, and the Permissive path; the prompt offered only yes/esc and no Grant once. |
| Protections not yet active are not shown as active in the inspector; approval for every command may be shown in force; vault/Corbanu-home protection, if listed, is not shown inactive | **passed** | Inspector top showed the restart-only session rows under Permissive and the saved/live Aggressive state. Scrolled inspector showed model-key broker off, credential broker unobserved, protected boundary off, and protected reads as always denied under Aggressive. |
| Screen 07 names approval from each pane's next turn, names the rest of the restart-only list as starting at restart, and does not say this session stays Permissive | **passed** | Screen 07 explicitly gave the approval timing and the restart-only list, and did not contain the forbidden phrase. |
| printf shows approval before restart, no `Ran` before answer, after Esc shown not run, and file does not exist | **passed** | Screens `AGG04-action3-approval` and `AGG04-action3-declined` and the own-shell ls observation satisfied all four parts. |

### Ambiguities and limitations

- The approval prompt for the protected Corbanu-home read said “Running” before the answer, but no `Ran` line appeared until after Enter; it then failed. This did not contradict the action-3 requirement, which concerned `printf ok > agg04-505.txt`.
- The inspector top-level warning said Aggressive was degraded because session controls apply after restart and because model-key/credential broker states were observed or configured as inactive/off. This supports, rather than contradicts, the expected inactive/restart-only distinction.
- Screen text was captured through the supplied recorder. I used only exact security names and phrases visible on the captured screens for checkpoints; I did not infer behavior from formatting or minor transcript rendering differences.

## Final verdicts

VERDICT AGG-04: passed
