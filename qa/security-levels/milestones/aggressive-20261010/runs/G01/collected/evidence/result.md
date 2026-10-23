# Group G01 functional execution report

Executor role: independent functional executor; this is an automated execution record, not human acceptance.

## Preflight and identity

- Time (UTC): 2026-10-10T23:10:33Z for `agms-probe all` completion; executor run output saved to `evidence/preflight-executor.json`.
- `agms-probe all` exit code: `0`; its JSON reports `"ok": true` with 85 probes and no failed probe.
- Coordinator preflight summary `evidence/preflight-summary.json` reports `"ok": true` and no failures; `preflight-launcher.json` and `preflight-tmux-child.json` are present beside it.
- `/opt/agms/pkg/bin/corbanu` SHA-256: `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a` (matches assignment).
- `/opt/agms/packet/design-cases.md` SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (matches assignment).
- Product `--version` observed through the coordinator preflight positive check: `corbanu 0.1.48`.
- Product launch/control used only through `agms-tui`.

## Fixture baseline (before cases)

- Time (UTC): 2026-10-10T23:15:53Z; `agms-tui fixture` observed:
  - `README.md`: describes `ledger-notes` and the command `python3 src/notes.py list`.
  - `src/notes.py`: reads/appends `notes.txt`.
  - `notes.txt`: exactly `buy coffee` and `review budget`.
  - `.env`: comment stating synthetic test value and `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`.
  - `.gitignore`: `.env`.
  - `outside-notes.txt` contains `outside-the-project: synthetic file`.
- Git status from the fixture workspace was clean at commit `42741cc initial notes`.

## Cases

### AGG-01

- Starting conditions requested: fresh profile, ledger-notes trusted, Permissive, zai route.
- Starting conditions actual: fresh product session from the supplied profile; the trust prompt was displayed at 2026-10-10T23:12:49Z and choice `1. Yes, continue` was accepted with Enter. The product then showed `model: glm-5.3-flash`, route `zai default`, and the ledger-notes fixture directory. No model request was made.
- Requested actions versus actual:
  1. `/security` then Enter — sent separately at 2026-10-10T23:12:56Z; security picker appeared.
  2. Down Down then Enter — performed at 2026-10-10T23:13:03Z; `Switch to Aggressive?` review appeared.
  3. Escape — performed at 2026-10-10T23:13:10Z; returned to picker and showed `Cancelled. Nothing changed.`
  4. Escape — performed at 2026-10-10T23:13:17Z; returned to normal chat prompt.
  5. Quit and relaunch — sent C-c at 2026-10-10T23:13:39Z; the tmux server/session ended. Relaunched at 2026-10-10T23:13:47Z; no trust prompt reappeared and the product reached chat.
  6. `/security` then Enter — sent separately at 2026-10-10T23:13:56Z; security picker appeared. Pressed `i`; inspector appeared.
- Positive checkpoints:
  - `agg01-security-picker`: `Active in this session: Permissive`; `Moderate  not available yet`.
  - `agg01-aggressive-review`: Aggressive review was reached without confirmation.
  - `agg01-after-first-esc`: picker retained Permissive as active, Aggressive merely selected, with explicit `Cancelled. Nothing changed.`
  - `agg01-after-second-esc`: closed to normal chat.
  - `agg01-inspector-after-relaunch`: `Saved: Permissive (nothing saved)`; `Core config: Permissive`; `Active this run: Permissive`; `Live policy: Permissive in force; next start Permissive`.
  - `agg01-inspector-scrolled-bottom` and `agg01-inspector-scrolled-final`: runtime, grants, agents, and not-contained rows matched screen 03/03b wording; the candidate additionally showed a `Not available` section naming PF-32, PF-37, PF-34, which is an additive difference and did not contradict the requested rows.
- Expected-result outcomes:
  - Picker shows `Active in this session: Permissive`; Moderate shows `not available yet` — **passed**.
  - Each Esc returns or closes without saving — **passed**.
  - After relaunch inspector shows `Saved: Permissive (nothing saved)` and session rows match screen 03 — **passed**.
- VERDICT AGG-01: passed

### AGG-02

- Starting conditions actual: after `agms-tui stop` at 2026-10-10T23:15:06Z, a new session was started on the supplied profile. It showed Permissive via the picker and the same zai/glm-5.3-flash route. Because AGG-01 had already run in the same product home, this was not a literally virgin profile, although no security level had been saved. Fixture contents were re-recorded as unchanged.
- Requested actions versus actual:
  1. `/security`, Down Down, Enter — text and Enter sent separately at 2026-10-10T23:15:15Z; Down Down then Enter reached `Switch to Aggressive?`.
  2. Scroll to the bottom with Down — 35 Down presses were sent in two batches. The first screen labeled `agg02-review-scrolled-20` and the next `agg02-review-scrolled-35` both showed the bottom content; further Down keys did not visibly change the already bottom-aligned view.
- Positive checkpoints:
  - The review screen showed explicit current `now` and proposed `Aggressive` rows for Sandbox, Approvals, Network, Vault, Model keys, and Child agents.
  - It showed `Preflight passed: no known raw-secret route stays open to agents or the model.` and `Controls ready: 6 of 6 (OS sandbox, Secretless agent launch, Isolated credential broker, Secret output gate, Untrusted-content labels, Vault)`.
  - It said the listed rows take effect on restart while the current session keeps existing settings until then; it also said `Your config.toml is not modified.` and that Core's level becomes Aggressive immediately on confirmation.
  - Footer showed `↑/↓ scroll · n refuse/pass · enter confirm and save · esc back, nothing changes · i inspect`.
- Expected-result outcomes:
  - Each setting shows `now` and `Aggressive` lines — **passed** for the six numbered session settings; the separate nested-agent line also states its behavior but does not label a line with `now`, matching the design-time screen.
  - A preflight result and controls-ready count are shown — **passed**.
  - The screen says which parts apply now and which at restart — **passed**.
  - Footer offers enter, esc, n and i — **passed**.
- VERDICT AGG-02: passed

### AGG-35

- Starting conditions requested: fresh profile, Permissive.
- Starting conditions actual: AGG-02 was canceled with Escape at 2026-10-10T23:16:30Z; the picker again showed `Cancelled. Nothing changed.` The session was stopped and a new product session started at 2026-10-10T23:16:30Z with the supplied profile and zai/glm-5.3-flash route. As with AGG-02, the product home had already been used by earlier cases, so this was not literally a virgin profile, but the visible level remained Permissive.
- Requested actions versus actual:
  1. `/security`, Down to Moderate, Enter — text and Enter were sent separately at 2026-10-10T23:16:40Z; one Down selected Moderate; Enter was then pressed.
- Positive checkpoints:
  - `agg35-before-enter`: `Active in this session: Permissive`; selection on `Moderate  not available yet`; text `Not available yet.`
  - `agg35-after-moderate-enter`: the picker remained on Moderate, `Active in this session: Permissive` did not change, and the screen added `Moderate is not available yet.`
- Expected-result outcomes:
  - Nothing is saved and it says not available yet — **passed** based on the unchanged visible `Active in this session: Permissive`, absence of a save/confirmation transition, and the explicit `Moderate is not available yet.` message. The screen did not include a literal phrase saying “nothing saved.”
- VERDICT AGG-35: passed

## Final evidence

- Fixture contents after all cases: re-recorded by `agms-tui fixture after-all`; no expected change was requested by these cases.
- No credential or system password prompt was encountered, no unexpected profile access was observed, and no product model request or model refusal occurred.

VERDICT AGG-01: passed
VERDICT AGG-02: passed
VERDICT AGG-35: passed
