# Group G15 independent functional run report

Run started: 2026-10-13T20:34:39Z (preflight probe UTC). Executor report created while work was in progress.

## Preflight and identity

- Coordinator preflight: read `$AGMS_RUN/evidence/preflight-summary.json` (`"ok": true`, 85 probes, no failed probes) and both adjacent `preflight-launcher.json` and `preflight-tmux-child.json` files were present.
- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=0`. JSON `"ok": true`; all listed probes reported `ok`.
- Candidate: `/opt/agms/pkg/bin/corbanu`, SHA-256 `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`; `--version` reported `corbanu 0.1.48`.
- Frozen cases SHA-256 matched `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Amendments/new-cases SHA-256 matched `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9`. No amendment heading for AGG-25 or AGG-34 was present.
- Fixture present under `$AGMS_RUN/fixture/workspace`; outside notes file at `$AGMS_RUN/fixture/outside-notes.txt`. Product home contained the supplied `config.toml` only, with security features enabled and no security level configured.

Assigned order: AGG-25, then AGG-34.

## AGG-25

Requested actions:
1. Launch a fresh profile with `-c 'cli_auth_credentials_store="file"'` as the E1 starting condition.
2. `/security`, then Down Down, then Enter.
3. Scroll down and press Enter.
4. Fix the displayed cause as instructed, relaunch, and retry.

Actual actions and observations:
- 2026-10-13T20:36:15Z launched with the requested option. The product process was later visible as `... -c cli_auth_credentials_store="file"`, confirming that the product received the literal.
- 2026-10-13T20:36:23Z, `25-TRUSTED-MAIN`: accepted the displayed trust-folder choice `1. Yes, continue` with Enter. Main screen showed `glm-5.3-flash`, route `zai`, and Permissive was the prior configured state (no explicit security row on the main screen).
- 2026-10-13T20:36:31Z sent `/security` as text without Enter; the screen showed the command entry and matching menu row. Enter then opened the picker, `25-PICKER-INITIAL`: `Active in this session: Permissive`, with `> Permissive  (active)`.
- 2026-10-13T20:36:35Z pressed Down Down; `25-AGGRESSIVE-HIGHLIGHT` showed `> Aggressive`.
- 2026-10-13T20:36:43Z pressed Enter to open the review. `25-REVIEW-TOP` unexpectedly showed `Preflight passed: no known raw-secret route stays open to agents or the model.` It did not show `Preflight blocked`, a specific fix, or the E1 `✗ launch flags cli_auth_credentials_store...` line. Pressing Down eight times scrolled the review (`25-REVIEW-SCROLLED`); it continued to show `Preflight passed` above.
- 2026-10-13T20:37:24Z pressed Escape rather than Enter. The picker showed `Cancelled. Nothing changed.` and remained `Active in this session: Permissive`. This preserved the profile instead of confirming through an unexpectedly permissive review.

Outcome:
- Expected `'Preflight blocked' is shown with a specific fix`: **failed**. Under the exact E1 launch option, the candidate showed preflight passed and no specific fix.
- Expected `Enter does not save and the picker still shows Permissive`: **not exercised**. The actual review stated preflight passed; pressing Enter would therefore be a normal save rather than test the blocked-path behavior, so it was not pressed.
- Expected `After the fix the preflight passes and saving works`: **not exercised**. The actual review displayed no cause to fix and no relaunch instruction.
- The case could not proceed exactly as written because the required blocked state was absent. No model decline occurred. Fixture contents were not changed.

VERDICT AGG-25: failed

---

## AGG-34

Requested actions:
1. Start from a fresh profile, trust the workspace, open `/security`, select Aggressive so the review is open.
2. Press `n`, Enter, `r`.
3. Open `/security`, then press `i`.

Actual actions and observations:
- 2026-10-13T20:38:36Z stopped AGG-25 and moved its generated profile aside, then created a fresh `product-home` containing only the supplied feature-flag `config.toml`. Fixture remained unchanged.
- 2026-10-13T20:38:41Z launched fresh. 2026-10-13T20:38:45Z, `34-START-TRUST`: accepted the displayed `1. Yes, continue` with Enter; `34-MAIN` showed route `zai` and `glm-5.3-flash`.
- 2026-10-13T20:38:50Z typed `/security` and pressed Enter. `34-PICKER` showed `Active in this session: Permissive`. Down Down highlighted Aggressive (`34-HIGHLIGHT`). Enter opened the review (`34-REVIEW-OPEN`): `Preflight passed`, and `Nested agents: refuse` with the list `corbanu exec`, `review`, `resume`, `fork`, a new session, `app-server`, and `mcp-server`.
- 2026-10-13T20:38:55Z pressed `n`; `34-REVIEW-NESTED-PASS` changed to `Nested agents: pass.` It stated that agent-started `corbanu exec` and `review` run with Aggressive enforced, while `interactive sessions`, `app-server`, and `mcp-server` stay refused.
- 2026-10-13T20:38:55Z pressed Enter; `34-SAVED` showed `Saved: Aggressive (nested agents: pass)` and the offered restart control `r restart now`. The product UI described Core as Aggressive immediately and the Aggressive session rows as starting after restart.
- 2026-10-13T20:39:01Z pressed `r`. By 2026-10-13T20:39:06Z, `34-AFTER-RESTART-FIRST` showed the restarted main screen; no trust prompt appeared.
- 2026-10-13T20:39:10Z typed `/security` and pressed Enter. `34-PICKER-AFTER-RESTART` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`. Pressing `i` opened `34-INSPECTOR-TOP`: `Saved: Aggressive`, `Active this run: Aggressive`, and `Live policy: Aggressive in force; next start Aggressive`.
- 2026-10-13T20:39:24–26Z scrolled to `34-INSPECTOR-AGENTS`. It showed `Nested agents: pass` and the same refused group visible in the review: `interactive sessions`, `app-server`, and `mcp-server`.
- 2026-10-13T20:40:10–12Z, `34-INSPECTOR-BOTTOM`: after more scrolling, the inspector header became stale (`status observed 1m ago; press r`). The nested row remained visible and unchanged. This is an observation, not a case failure because `r` is the displayed refresh control and the expected comparison had already been captured.
- 2026-10-13T20:40:19Z ran `agms-tui fixture 34-AFTER`; all workspace and outside fixture contents matched the supplied fixture. No model turn was requested, so no model decline occurred.

Outcome:
- Expected `The saved line shows 'nested agents: pass' and the inspector agrees`: **passed** (`34-SAVED` and `34-INSPECTOR-AGENTS`).
- Expected `The inspector's list of refused nested commands matches the review's list`: **passed**; both identified `interactive sessions`, `app-server`, and `mcp-server` as refused, with the pass rule allowing agent-started `corbanu exec` and `review`.

VERDICT AGG-34: passed

## Final verdict summary

VERDICT AGG-25: failed
VERDICT AGG-34: passed

These are the independent functional execution outcomes only; no human acceptance is claimed.
