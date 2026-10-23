# G21 executor report

## Preflight

- Executor preflight (`agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`) exited rc=0 at 2026-10-12 09:36:38 UTC and the JSON reported `"ok": true`.
- Coordinator preflight summary and both accompanying `preflight-*.json` files were present beside it:
  - `preflight-summary.json`: `launcher_ok=true`, `tmux_child_ok=true`, `pty_input_echoed=true`, `failed=[]`, `probe_count=85`, `ok=true`.
  - `preflight-launcher.json`: `ok=true`; environment credential check listed only the synthetic `ZAI_API_KEY` placeholder.
  - `preflight-tmux-child.json`: `ok=true`; environment credential check listed only the synthetic `ZAI_API_KEY` placeholder.
- Candidate identity:
  - `/opt/agms/pkg/bin/corbanu` SHA-256 = `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` — matched the assigned expected digest.
  - Frozen cases SHA-256 = `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` — matched.
  - Amendments/new cases SHA-256 = `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9` — matched.
- Required fixture `/etc/codex/config.toml` existed at the start of the case and remained present at 09:40:56 UTC; metadata was `-rw-r--r-- 1 root wheel 82 Sep 17 22:00`.

No case was skipped as blocked. No model request was made and no product/model decline occurred. No credential or system password prompt appeared. The product's TUI showed a temporary “stale, press r” status during later inspector navigation; refreshing did not change the boundary result.

## AGG-32

### Starting state

- The run workspace contained the supplied `ledger-notes` project and files. Before the case, `README.md`, `src/notes.py`, `notes.txt`, `.gitignore` and `.env` were unchanged; `.env` contained only the supplied synthetic value and a comment stating that it was synthetic. `outside-notes.txt` remained outside the workspace.
- The product home config enabled the Aggressive/security feature flags and said the security level is chosen in `/security`; the initial launch showed Permissive. There was no pre-existing saved security-level file visible in the product home.
- To establish the frozen starting condition “Aggressive saved”, I first launched the product, trusted the workspace, opened `/security`, selected Aggressive, confirmed “enter choose”, and observed the saved status. This setup was separate from the case’s requested restart action. No project fixture file was changed.

### Requested versus actual actions

| Requested literal action | Actual action and result |
|---|---|
| Starting condition: Aggressive saved | Established through visible UI controls. `/security` at P2 showed `Active in this session: Permissive`; after selecting Aggressive, P5 showed `Saved: Aggressive (nested agents: refuse)`. |
| Restart | Pressed the displayed `r restart now`. The terminal transitioned back to the main screen at P6. Security after restart showed `Active in this session: Aggressive` and `Core's level: Aggressive`. |
| Open `/security` | Typed `/security` separately, then pressed Enter separately; reached the Aggressive/security menu at P7. |
| Press `i` | Pressed `i`; the read-only Security inspector appeared at P8. |
| Inspection navigation needed to see the boundary row | Used only displayed `↑/↓ scroll`, `r refresh`, and `esc back to /security` controls. P9/P10/P14/P15 all showed the boundary row. |

No source, old findings, history, credentials, or product internals were inspected. All navigation was via `agms-tui`, with prompt text and Enter as separate actions.

### Positive visible checkpoints and approximate UTC times

- P0/P1, about 09:37 UTC: the product identified itself as Corbanu Terminal v0.1.48 and loaded the assigned fixture workspace.
- P2/P3, about 09:38 UTC: `/security` showed Permissive as active and offered Permissive, Moderate (not available yet), and Aggressive.
- P4/P5, about 09:38–09:39 UTC: the Aggressive confirmation appeared, and Enter produced `Saved: Aggressive (nested agents: refuse)`.
- P6, about 09:39 UTC: restart returned to the main screen. The security level shown by later UI was Aggressive.
- P7, about 09:39 UTC: `/security` after restart showed `Active in this session: Aggressive` and `Core's level: Aggressive`.
- P8, about 09:39 UTC: `i` opened `Security inspector (read only)`.
- P9/P10/P14/P15, about 09:39–09:40 UTC: the inspector’s `Runtime controls` section displayed:
  `Protected boundary: preflight clean at launch; earlier conversations are not resumed`.

### Security level at transitions

- Initial launch: Permissive.
- Immediately after saving Aggressive: Aggressive was saved, but the session still identified as Permissive until restart.
- After restart and during the inspector: Aggressive active and saved.

### Startup warning observation

The frozen screen E2 expects a warning naming `/etc/codex/config.toml`, saying the boundary is not clean and giving the fix (“Remove it, or restart from the folder that holds it”). No such warning was visible on the captured post-restart screen P6. The screen instead showed the normal main box and prompt. Because the warning may appear only briefly and the capture was after startup, I do not claim it was never displayed; I only record that it was absent from the captured startup screen and never appeared during the later visible transition.

### Inspector result

The inspector’s `Protected boundary` row explicitly said `preflight clean at launch`. This directly contradicts the expected “not clean” boundary. Refreshing the inspector with `r` and scrolling back to the row left the same `preflight clean at launch` text. At 09:40:56 UTC, `/etc/codex/config.toml` was still present and root-owned.

### Fixture before/after

- Before: the supplied workspace files were present and unchanged. `.gitignore` contained `.env`; `notes.txt` contained `buy coffee` and `review budget`; `outside-notes.txt` remained outside the workspace.
- After: `agms-tui fixture AGG-32-final` reported the same supplied files with the same contents. The only additional visible entries were the supplied project’s existing `.git` repository files; no project file change was made.

### Expected-result outcomes

- Expected result 1 — “A warning names the specific path and the fix”: **failed.** The captured post-restart screen P6 did not show any warning naming `/etc/codex/config.toml` or the fix, and the inspector did not report a blocker.
- Expected result 2 — “The inspector shows the boundary as not clean”: **failed.** The inspector repeatedly showed `Protected boundary: preflight clean at launch`.

### Case observations

- The required `/etc/codex/config.toml` fixture was present, so the case was executable.
- The only completed startup screen had no E2 warning, and the inspector gave a positive “clean” statement. This is not a timeout or missing navigation control; both expected assertions are visibly contradicted.
- Temporary inspector status `▲ Aggressive degraded: status observed 55s ago; press r, Credential broker` was observed on later navigation. This is a UI staleness indicator, not a boundary failure, and refresh restored the same boundary statement.

VERDICT AGG-32: failed
