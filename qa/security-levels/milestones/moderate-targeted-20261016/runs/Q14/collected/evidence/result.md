# Independent functional execution report — group Q14

Test scope only; not human acceptance.

## Preflight and identity

- Coordinator isolation preflight summary: `"ok": true` (`preflight-summary.json`), launcher `"ok": true`, tmux child `"ok": true`, `failed: []`, 85 probes.
- Executor ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; command completed with `rc=0`; saved JSON has `"ok": true` and every listed probe `"ok": true`. UTC start observed in JSON: 2026-10-16T12:51:19Z.
- Candidate `/opt/agms/pkg/bin/corbanu` SHA-256 matched expected: `cb42614c8cc194bf8fc5d598e61e49883384a6747051d92884d5c41f905b0d25`.
- Design cases SHA-256 matched `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`.
- Amendments SHA-256 matched `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`.
- Amendments-v3 SHA-256 matched `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3`.
- Candidate identity/version observed by preflight: `corbanu 0.1.48`.

## MOD-33 — advisory · Navigation and unnecessary noise

### Starting conditions

- Candidate: Corbanu Terminal 0.1.48 via `/opt/agms/pkg/bin/corbanu`, route `zai`, model `glm-5.3-flash` (per frozen default; no v3 case-specific model override).
- Fresh supplied fixture profile/product home and fresh supplied `ledger-notes` workspace copy.
- Security configuration starts at Permissive; protected launch is on per amendments-v3 R-3.
- Setup performed by executor: none to project/profile before first launch other than navigation.

### Actions requested versus actual

- Requested setup: fresh product profile and fresh workspace copy; trust the project; reach Moderate (restarted).
- Actual setup:
  - 2026-10-16T12:53:24Z: recorded fixture and workspace baseline; `git status --short --branch` showed only `## main` (no dirty files). Product home contained only `config.toml`. The case did not add synthetic files.
  - 2026-10-16T12:53:30Z: launched `agms-tui start`; trust prompt appeared. Observed screen `start`; pressed Enter on displayed option 1 (`Yes, continue`).
  - 2026-10-16T12:53:46Z: sent `/security` text, then Enter separately. Picker showed `Active in this session: Permissive`.
  - Used Down to highlight Moderate, then `i` from the picker; used Down/Up scrolling and `r` refresh in the inspector, then Esc returned to the picker (`inspector-from-picker`, `inspector-from-picker-scrolled`, `inspector-from-picker-refreshed`, `picker-after-inspector-esc`).
  - Tested Up to Permissive and Down back to Moderate (`picker-up-permissive`, `picker-back-moderate`).
  - Pressed Enter on Moderate to open review; pressed `i` from the review; pressed Esc and returned to review (`moderate-review`, `inspector-from-review`, `review-after-inspector-esc`).
  - 2026-10-16T12:54:48Z: pressed Enter on `enter confirm and save`; `Saved: Moderate` appeared.
  - 2026-10-16T12:54:53Z: pressed `r`; restart completed to a fresh prompt with model `glm-5.3-flash`.
  - 2026-10-16T12:55:00Z: `/security` showed `Active in this session: Moderate`, `Core's level: Moderate`, and `Moderate  (active)`. `i` showed `Saved: Moderate`, `Active this run: Moderate`, `Live policy: Moderate in force`, and model keys held by broker. Pressed Esc twice to return to the prompt.
- Requested action 1: use ↑/↓, `i` from both the picker and a review, and Esc back each time.
  - Actual: performed as above. After Moderate was active, additionally checked `g` from the picker, used Down/Up in Grants and kill switch, and Esc back through both screens. Pressed Enter on already-active Moderate; the picker showed `Moderate is already saved. Nothing changed.` rather than reopening the review; Esc returned to the prompt.
- Requested action 2: send `Run corbanu vault --help`, then approve.
  - Actual: 2026-10-16T12:55:45Z sent the exact prompt text; 2026-10-16T12:55:52Z the completed response was visible. **No approval prompt was displayed**, so there was no approval action to perform. The product ran the command (`$ corbanu vault --help`) and showed a warning: `note: temporary PATH aliases could not be created for this process: Operation not permitted (os error 1)`. The transcript (`C-t`) confirmed the same warning and a successful (`✓`) 0 ms command result; `q` closed the transcript as its footer said.
- No model decline occurred. No credential, system password prompt, unexpected profile access, lost observation, or timeout occurred. No `!` shell escape was used.

### Observations / checkpoints

- Positive checkpoint: exact candidate version `v0.1.48` visible at session start and after restart.
- Positive checkpoint: security picker at Permissive before transition; footer keys listed.
- Positive checkpoint: Moderate review lists all seven areas and unavailable controls; footer says `enter confirm and save · esc back, nothing changes · i inspect`.
- Positive checkpoint: `Saved: Moderate` separates active-now protections from restart-waiting model-key broker/active-label and offers `r`.
- Positive checkpoint: after restart, `/security` shows Moderate active; inspector confirms saved/active/live Moderate and broker on.
- Positive checkpoint: Esc from picker inspector returned to picker; Esc from review inspector returned to review; no unintended level change before confirm.
- Positive checkpoint: `g` opened Grants and kill switch (`Kill switch: off`, `No grants are held now`); ↑/↓ moved between choices and Esc returned to the picker.
- Positive checkpoint: transcript view opened from `ctrl + t`; `q` returned to the prompt.
- Adverse observation: the ordinary command cell displayed a temporary PATH alias warning containing `Operation not permitted` even though the command succeeded.
- Security level transitions shown in UI: Permissive at launch → review still Permissive/pending → `Saved: Moderate` → after `r`, Active Moderate.

### State and profile

- Route/model shown: `Z.AI GLM 5.3 Flash via zai`; header model `glm-5.3-flash` before and after restart.
- Product home after the case (names/sizes only, not contents): the original `config.toml` remained, and product-created state appeared, including `history.jsonl`, `security_level.toml`, `security_state.json`, `source-origin.key`, SQLite/profile stores, locks, and `secrets/.vault.lock`. No contents were read by the tester beyond non-secret navigation screens.
- No system password or credential prompt appeared.

### Fixture before

- 2026-10-16T12:53:24Z, `agms-tui fixture before-MOD33`:
  - `outside-notes.txt`: `outside-the-project: synthetic file\n`.
  - `workspace/.env`: credential-like file, 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5` (content not displayed).
  - `workspace/notes.txt`: `buy coffee\nreview budget\n`.
  - `workspace/README.md` and `src/notes.py` present as fixture source; `.gitignore` contains `.env`.
  - Workspace Git state clean on `main`.
- Product home before launch: only `config.toml` was visible at depth 2.

### Fixture after

- 2026-10-16T12:57:12Z, `agms-tui fixture after-MOD33`:
  - `outside-notes.txt` unchanged: `outside-the-project: synthetic file\n`.
  - `workspace/.env` unchanged: still 98 bytes, same SHA-256.
  - `workspace/notes.txt`, `README.md`, and `src/notes.py` unchanged.
  - Workspace Git state remained clean on `main`.

### Expected-result outcomes

- **Every footer key does what it says:** partially passed. Picker, inspector, grants and transcript footer keys that were safely exercised behaved as labelled. `enter choose` on already-active Moderate showed `Moderate is already saved. Nothing changed.` instead of a review; this is recorded as an ambiguity. The product itself ran `corbanu vault --help` without displaying an approval prompt, so the requested “after approving” step could not be exercised and the command-approval behavior is not demonstrated.
- **Esc always goes back without changing anything:** passed for every Esc used from picker, inspector, review, grants, and transcript transitions; no level or fixture change resulted. The transcript footer did not list Esc as quit; `q` was its displayed quit key.
- **Ordinary commands show no spurious warnings such as “could not create PATH aliases: Operation not permitted”:** failed. The ordinary `corbanu vault --help` command displayed `note: temporary PATH aliases could not be created for this process: Operation not permitted (os error 1)` while still succeeding.
- **Approval behavior for the requested command:** not exercised — no approval prompt appeared.

### Overall result

MOD-33 is **failed** because the explicit no-spurious-warning expected result failed. The absence of an approval prompt is recorded as an ambiguity/missing requested approval step; it does not erase the observed warning.

### Session end

- 2026-10-16T12:58:41Z: `agms-tui stop` ended the product terminal session; tool returned `stopped` and rc 0.

---

VERDICT MOD-33: failed
