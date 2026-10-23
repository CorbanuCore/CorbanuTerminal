# Independent functional execution — group G01

Run UTC started: 2026-10-12T05:08:05Z (executor preflight completion)

## Preflight and identity

- Executor preflight: `agms-probe all > evidence/preflight-executor.json; echo rc=0`; JSON reports `"ok": true`. Launcher/tmux preflight summary reports `"ok": true`.
- Candidate `/opt/agms/pkg/bin/corbanu`: `corbanu 0.1.48`; SHA-256 `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` (matches).
- Frozen cases SHA-256 matches `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Amendments/new-cases SHA-256 matches `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9`.
- Isolation: no coordinator instruction was accepted to broaden action. All product interaction uses `agms-tui`. Fixture is confined to `$AGMS_RUN/fixture/workspace`.

---

## AGG-01

Starting conditions: fresh profile, ledger-notes trusted, Permissive, zai route.

### AGG-01 requested versus actual
- Requested: fresh profile and ledger-notes trust; start Permissive, zai route.
- Actual: launcher started from fixture/workspace; trust chooser displayed at 05:10:17Z. Pressed Enter on displayed `1. Yes, continue` at 05:10:34Z. Main UI then showed version 0.1.48, model glm-5.3-flash, route `zai default`, fixture/workspace (AGG01-initial). No credential/system password prompt appeared.
- Requested: type `/security`, Enter. Actual: text and Enter sent separately (05:12:42Z, 05:12:49Z).
- Requested: Down Down, Enter. Actual: Down at 05:12:58Z selected Moderate (`not available yet`); second Down selected Aggressive; Enter opened review at 05:12:59Z.
- Requested: Esc, Esc. Actual: first Escape at 05:13:10Z returned to picker and printed `Cancelled. Nothing changed.`; second Escape returned to main chat at 05:13:10Z.
- Requested: quit and relaunch. Actual: typed `/quit` and Enter separately; tmux then reported server ended (05:13:19Z). Relaunch at 05:13:28Z; no trust prompt appeared (ledger-notes remained trusted).
- Requested: open `/security`, press i. Actual: text and Enter separately; then i at 05:13:35Z opened read-only inspector.

### AGG-01 checkpoints
- 05:12:49Z, `AGG01-security-open`: Picker shows `Active in this session: Permissive`, `Permissive (active)`, `Moderate not available yet`, and Aggressive. Footer shows enter/esc/i. Security level shown: Permissive. [passed]
- 05:12:58Z, `AGG01-down2`: Aggressive was highlighted; active session remained `Active in this session: Permissive`. [passed]
- 05:13:10Z, `AGG01-esc1`: First Esc returned without saving and displayed `Cancelled. Nothing changed.`; active remained Permissive. [passed]
- 05:13:10Z, `AGG01-esc2`: Second Esc returned to main chat without any saved/changed message. [passed]
- 05:13:35Z, `AGG01-inspector`: Shows `Saved: Permissive (nothing saved)`, `Core config: Permissive`, active/live Permissive. [passed]
- 05:14:06-09Z, `AGG01-inspector-scrolled1/2`: Session rows were broadly in the same order, but the actual Vault session row includes allowlist wording while screen 03 says secret-like environment variables are passed through; additional runtime rows also differ from screen 03. This is not a match. [failed]
- Fixture before (`initial-AGG01`) and after (`after-AGG01`) were unchanged. [passed]

### AGG-01 expected-result outcomes
1. Picker shows Permissive and Moderate unavailable — passed.
2. Each Esc returns/closes without saving — passed.
3. After relaunch, Saved Permissive shown — passed; session rows match screen 03 — failed because Vault row text differs and other visible runtime/inspector rows differ.

---

## AGG-02

Starting conditions: Permissive with no security change saved by AGG-01. Product-home retained ordinary session/log artifacts from AGG01; they were not inspected or reset, and the security profile remained fresh/unsaved. Fixture recorded as `initial-AGG02`.

### AGG-02 requested versus actual
- Requested: `/security`, Down Down, Enter. Actual: `/security` text and Enter separately (05:15:37-38Z); Down/Down/Enter at 05:15:38Z. Aggressive review opened at top (`AGG02-review-top`).
- Requested: scroll to bottom with Down. Actual: 30 Down presses to 05:15:52Z (`AGG02-review-scroll1`), then 30 more; second screen at 05:15:59Z (`AGG02-review-bottom`) was unchanged and showed the end of the review. Cleanup Escape returned to picker with `Cancelled. Nothing changed.` and session remained Permissive; this cleanup was not part of the case.

### AGG-02 checkpoints
- 05:15:52-59Z, `AGG02-review-scroll1/2`: every listed setting shows a `now` line followed by an `Aggressive` line. [passed]
- 05:15:52-59Z: review shows `Preflight passed: no known raw-secret route stays open to agents or the model.` and `Controls ready: 6 of 6 (...)`. [passed]
- 05:15:59Z, `AGG02-review-bottom`: screen explicitly says Core's protections below apply on confirm and the rest of rows above take effect at restart. It names the vault command refusal and protected-path denials applying now, and the restart-only rows include sandbox, approvals, network, environment/model-key/child-agent settings. [passed]
- 05:15:52-59Z: review includes protected reads `never granted, not even once`; credential files including `ones created later`; project `.codex` and `.agents` read-only even with Grant once; and only allowlisted environment variables reaching agent commands. [passed]
- Footer at review bottom offers `↑/↓ scroll · n refuse/pass · enter confirm and save · esc back, nothing changes · i inspect`. [passed]
- Fixture after (`after-AGG02`) remained unchanged. [passed]

### AGG-02 expected-result outcomes
1. Each setting has now/Aggressive lines — passed.
2. Preflight result and controls-ready count shown — passed.
3. Now/restart split shown — passed.
4. Footer offers enter, esc, n and i — passed.
5. Amendment: protected reads never granted/credential files later/.codex/.agents/environment allowlist statements — passed.
6. Amendment: vault refusal and Corbanu-home protection named now, restart-only list applies at restart — passed.

---

## AGG-35

Starting conditions: Permissive with no security change saved by prior cases. Fixture recorded as `initial-AGG35`.

### AGG-35 requested versus actual
- Requested: `/security`, Down to Moderate, Enter. Actual: start at 05:17:03Z; `/security` text and Enter separately (05:17:05Z); one Down selected Moderate at 05:17:05Z; Enter at 05:17:05Z. Cleanup Escape returned to main chat and was not a case action.

### AGG-35 checkpoints
- 05:17:05Z, `AGG35-picker`: Picker showed `Active in this session: Permissive`, `Permissive (active)`, `Moderate not available yet`, Aggressive. [passed]
- 05:17:05Z, `AGG35-moderate`: Moderate was highlighted with `not available yet`. [passed]
- 05:17:05Z, `AGG35-moderate-result`: After Enter, screen showed `Moderate is not available yet.`; active session remained `Active in this session: Permissive`, and `Permissive (active)` remained highlighted as the saved/active level. Nothing was saved. [passed]
- Fixture after (`after-AGG35`) remained unchanged. [passed]

### AGG-35 expected-result outcomes
1. Nothing saved and it says not available yet — passed.

---

## Final verdicts

VERDICT AGG-01: failed
VERDICT AGG-02: passed
VERDICT AGG-35: passed

These are independent functional execution outcomes only, not human acceptance.

### Fixture integrity across all cases

Fixture was byte-for-byte unchanged across the recorded before/after snapshots. SHA-256 after AGG-35:
- workspace/README.md: `5eafd8d8f99086242914e721d88e03d7cef4066402b416d2d6e18cf194a9cc88`
- workspace/notes.txt: `67fbf93a8f22aae1b72212f2a170a86071ccefbdca94408af4bc99727f069dae`
- workspace/.env: `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5` (synthetic fixture; value not quoted)
- workspace/.gitignore: `69aa73cf5bc3377a7d97f2e8d2bb773195815efe225af7f1be6f5b9306bbe14f`
- workspace/src/notes.py: `8cfc17de788ec38f7c04a5916b5b1e5896691d421b32d92bb922257985b131ec`
- outside-notes.txt: `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`
