# Functional execution report — group Q01

Candidate: Corbanu Terminal 0.1.48, `/opt/agms/pkg/bin/corbanu` (read-only).
Executor role: independent functional executor; this report records observed outcomes only and does not claim human acceptance.

## Identity and isolation preflight

- Preflight command requested: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`
- Actual: completed at 2026-10-14T08:10:56Z per the tool's context; exit code `0`.
- Executor preflight JSON top-level result: `"ok": true`; `preflight-summary.json`: `"ok": true`, `launcher_ok: true`, `tmux_child_ok: true`, `failed: []`.
- Package SHA-256 observed: `dc61bd6403b99a892b3db4ed59fef22f30a4ff65f1cd2a9ee165e994c2a50483` (matches required).
- Frozen cases SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (matches required).
- Amendments/new-cases SHA-256 observed: `072a8c3fc4910e819fbcf569fd359bfff7521f7b02587f2e272936d311da7523` (matches required).
- Product version observed by preflight: `corbanu 0.1.48`.

## Case AGG-01

Requested case, latest amendment applied: fresh profile; ledger-notes trusted; Permissive; `zai`; open `/security`; navigate to Aggressive and close twice; relaunch; inspect Permissive. The 2026-10-14 amendment changes the final inspector comparison to allow only the Vault row's environment wording from new-screens-4/U01.

- Setup requested: make a fresh fixture/profile copy, then launch `agms-tui` and trust the ledger-notes folder if offered.
- Setup actual: the supplied workspace was copied to `fixture/pristine-workspace` at 08:14:44Z, then a fresh copy was restored to `fixture/workspace`; `outside-notes.txt` was restored. A fresh `product-home` contained only the supplied read-only `config.toml` copy. Launched `agms-tui start --route zai`; the product displayed its trust prompt, and Enter accepted the already-highlighted `1. Yes, continue` at 08:15:01Z. Start screen label `AGG01-start` at 08:15:14Z showed model `glm-5.3-flash` and route `zai`.
- Security level at first transition: Permissive (`AGG01-picker-initial`, 08:15:28Z).

### Requested versus actual actions

| # | Requested | Actual | Time (UTC) | Screen |
|---|---|---|---|---|
| 1 | Setup fresh profile/fixture; trust ledger-notes | Done as above; trust accepted through its native numbered prompt with Enter. | 08:14:44Z–08:15:14Z | trust prompt; AGG01-start |
| 2 | Type `/security`, Enter | Text typed and Enter pressed separately. Picker appeared with Permissive active. | 08:15:21Z–08:15:28Z | AGG01-security-typed; AGG01-picker-initial |
| 3 | Down Down to Aggressive, Enter | Down Down highlighted `> Aggressive`; Enter opened the Aggressive review. | 08:15:40Z–08:15:49Z | AGG01-picker-aggressive-highlight; AGG01-review |
| 4 | Esc | Review returned to picker; message `Cancelled. Nothing changed.` | 08:15:57Z | AGG01-after-first-esc |
| 5 | Esc | Picker closed to the main product screen. | 08:16:13Z | AGG01-after-second-esc |
| 6 | Quit and relaunch | `/quit` typed and Enter pressed separately at 08:16:28Z; the tmux server ended (the following `wait` returned “no server running”). Relaunched `--route zai` at 08:16:40Z, no trust prompt appeared. | 08:16:28Z–08:16:40Z | no post-exit screen; relaunched main screen |
| 7 | Open `/security`, press i | Text and Enter separately; then `i` opened the read-only inspector. | 08:16:47Z–08:16:54Z | AGG01-relaunch-picker; AGG01-relaunch-inspector-top |

### Positive checkpoints and expected results

- Positive: initial picker (`AGG01-picker-initial`) showed `Active in this session: Permissive`; Permissive was active; Moderate showed `not available yet`; Aggressive was available. Security level at transition: Permissive.
- Positive: first Esc returned to picker and explicitly said `Cancelled. Nothing changed.` Security level still Permissive.
- Positive: second Esc closed to main screen, still Permissive.
- Expected result 1 — picker shows `Active in this session: Permissive`; Moderate shows `not available yet`: **passed** (`AGG01-picker-initial`).
- Expected result 2 — each Esc returns or closes without saving: **passed** (`AGG01-after-first-esc`, `AGG01-after-second-esc`); no save/confirm action was taken.
- Expected result 3 — after relaunch inspector shows `Saved: Permissive (nothing saved)` and session rows match screen 03 except the amended Vault wording: **passed** (`AGG01-relaunch-inspector-top`, 08:16:54Z). It showed the required Saved line. All eight session rows were present with the same settings as screen 03; only folder paths differed (allowed) and Vault said `only allowlisted environment variables reach agent commands`, matching `new-screens-4/U01-inspector-permissive-fresh.txt`. No other changed wording was observed.

### Fixture before/after

- `AGG01-before` (08:14:44Z) recorded the fresh ledger-notes files, including `.env` with the synthetic test value, `.gitignore`, README, notes, source and git metadata.
- At 08:17:21Z, `AGG01-after` was recorded; `diff -rq workspace pristine-workspace` reported no differences and `outside-notes.txt` was unchanged.

### Model refusals/declines

None observed.

### Model refusals/declines

None observed.

## Case AGG-02

Requested case, latest amendment applied: fresh profile; Permissive; switch picker to Aggressive and Enter; scroll the review to the bottom. Review must retain frozen expectations 1–4 and contain the added protected-read, later-created credential file, `.codex`/`.agents`, allowlisted-environment, now/restart split, and vault/Corbanu-home text.

- Setup requested: make a fresh fixture/profile copy, then launch `agms-tui` and trust the ledger-notes folder if offered.
- Setup actual: a new fresh profile/fixture copy was restored at 08:18:04Z. Launched `--route zai`; the product showed its native trust prompt, and Enter accepted the highlighted `1. Yes, continue` at 08:18:23Z. Main screen showed `glm-5.3-flash` via `zai` (`AGG02-start`).
- Security level at first transition: Permissive (`AGG02-picker-initial`, 08:18:30Z). The Aggressive review was opened but not confirmed, so no saved/security-level transition occurred.

### Requested versus actual actions

| # | Requested | Actual | Time (UTC) | Screen |
|---|---|---|---|---|
| 1 | Setup fresh profile/fixture | Done; fresh profile and project restored. The blocking trust prompt was answered through its native highlighted `Yes` choice and Enter so navigation could continue. | 08:18:04Z–08:18:23Z | trust prompt; AGG02-start |
| 2 | `/security`, Down Down, Enter | Text typed and Enter separately; Down Down highlighted Aggressive; Enter opened the review. | 08:18:30Z–08:18:35Z | AGG02-picker-initial; AGG02-picker-aggressive; AGG02-review-top |
| 3 | Scroll to bottom with Down | Sent Down in literal repeated groups of 10; after 30 total Down presses the screen no longer advanced between the 20th and 30th. The bottom view was captured. | 08:18:43Z–08:19:19Z | AGG02-review-scroll-1; AGG02-review-scroll-2; AGG02-review-bottom |

### Positive checkpoints and expected results

- Positive: picker showed `Active in this session: Permissive` and `Moderate  not available yet`; Aggressive was highlighted (`AGG02-picker-aggressive`).
- Expected result 1 — each setting shows `now` and `Aggressive` lines: **passed** (`AGG02-review-top`). Observed paired now/Aggressive rows for sandbox, approvals, network, vault, model keys and child agents; nested agents had the additional refuse/pass behavior row.
- Expected result 2 — a preflight result and controls-ready count are shown: **passed** (`AGG02-review-top`): `Preflight passed: no known raw-secret route stays open to agents or the model.` and `Controls ready: 6 of 6 ...`.
- Expected result 3 — the screen says which parts apply now and which at restart: **passed**. The amended review said Core’s protections (direct `corbanu vault …` refusal and protected paths including vault store, sign-in/credential files, Corbanu config/history/state) apply as soon as confirm; the rest of the rows above apply at restart. It identified the restart-only categories: sandbox, approval for every command, network, environment, model keys and child agents.
- Expected result 4 — footer offers enter, esc, n and i: **passed** (`AGG02-review-bottom`): `enter confirm and save · esc back, nothing changes · n refuse/pass · i inspect` (same footer also visible in top/intermediate screens).
- Amendment addition — protected reads are never granted, not even once: **passed** (`AGG02-review-top`): `... is denied to agent commands and is never granted, not even once ...`.
- Amendment addition — credential files include ones created later: **passed** (`AGG02-review-top`): credential files described as `.env, .env.* and .envrc files, also ones created later ...`.
- Amendment addition — project `.codex` and `.agents` stay read-only even with Grant once: **passed** (`AGG02-review-top`): `the project's .git, .codex and .agents stay read-only, even with Grant once ...`.
- Amendment addition — only allowlisted environment variables reach agent commands: **passed** (`AGG02-review-top`): the Vault `now` row says `only allowlisted environment variables reach agent commands ...`.

### Fixture before/after

- `AGG02-before` (08:18:04Z) recorded the fresh ledger-notes files and synthetic `.env`.
- At 08:19:41Z, `AGG02-after` was recorded; workspace diff against pristine had no differences and `outside-notes.txt` was unchanged.

### Model refusals/declines

None observed.

## Final verdicts

VERDICT AGG-01: passed
VERDICT AGG-02: passed
