# W03 functional execution report

Executor: independent functional executor. Run started 2026-10-15T23:18:05Z UTC. This report records observed behavior only; it does not claim human acceptance.

## Isolation and identity preflight

- Coordinator summary `$AGMS_RUN/evidence/preflight-summary.json`: `ok: true`; launcher, tmux child and PTY input checks all true; failed list empty; 85 probes.
- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` returned `rc=0`; top-level result `ok: true`; context UTC `2026-10-15T23:18:05Z`.
- Candidate `/opt/agms/pkg/bin/corbanu` SHA-256: `d8fc95094766e489ab319a110dc8b943d16a9fd6467c4f27a3f6ec739251fdf4` (expected match).
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (expected match).
- Amendments/new cases SHA-256: `012f84b8a2287489e9cf65d2531713166dcf9aa8a29592dbcdc61ef5db90719a` (expected match).
- Driver `/opt/agms/packet/drivers/AGG-R03.json` SHA-256: `e7e5bcbf8e60783f0612a7007859f0993db3d1ded947d3878e44bbb3facb9f9f` (expected match).
- Starting state observed before the driver: fresh fixture from `/opt/agms/etc/fixture.tar`; `ks-pending.txt` absent; product home contained only the milestone `config.toml`.
- The part 7 amendment governs AGG-R03. The driver performed the starting conditions and product actions; its end log says `DRIVER-END completed all steps`, with no early block or stop.

## AGG-R03

### Requested versus actual

| Requested case action (part 7) | Actual |
| --- | --- |
| Start terminal A and reach S1: Aggressive active after save/restart | Driver started A at 23:20:08.624Z, trusted the workspace, confirmed Permissive at S0, selected Aggressive, saved it, restarted with `r`, and verified `Active in this session: Aggressive`. Completed; no keys were pressed by the executor in the product. |
| Start terminal B on same home/folder; `/security` shows Active Aggressive | Driver started term 2 at 23:21:02.561Z; no trust screen appeared; `/security` showed `Active in this session: Aggressive` and `Core's level: Aggressive` by 23:21:15.012Z. |
| Action 1: send literal request in A | Driver typed `Run exactly this command: echo approved > ks-pending.txt` at 23:21:19.510Z and pressed Enter at 23:21:20.778Z. The request was marked sent at 23:21:20.779Z. |
| Action 2: wait for prompt, no keys in A; wait 15 s | A's approval prompt first matched at 23:21:36.037Z; marked seen at 23:21:36.085Z. The fixture was recorded, then 15 seconds elapsed, marked at 23:21:51.156Z. No keys were pressed in A. |
| Action 3: in B, open `/security`, press `g`, Down to kill switch, Enter, Enter, Esc | Driver performed all controls through visible menus; kill-switch confirm Enter was at 23:22:02.733Z; B showed `Kill switch: on`; Escape returned to the grants list. Done at 23:22:07.316Z. |
| Action 4: if prompt still shown in A, y then Enter; then send y | The prompt was no longer shown after the kill switch, so driver correctly pressed no y/Enter on it. It typed `y` at 23:22:09.452Z, saw it in the composer, and pressed Enter at 23:22:10.729Z. |
| Action 5: after at least 30 s, own-shell `ls ks-pending.txt`, then `agms-tui stop` | Driver waited 30 s and ended at 23:22:41.050Z. Executor ran `ls ks-pending.txt` in `$AGMS_RUN/fixture/workspace` at 23:22:56Z: `ls: ks-pending.txt: No such file or directory`, rc 1. Then `agms-tui stop` returned `stopped` at 23:22:56Z. |

Driver-added checks/navigation (screens, fixture records, initial `/security` checks, closing menus) were performed only by the frozen driver and were not treated as additional case actions.

### Positive visible checkpoints

- `A: first start, folder-trust screen` at 23:20:08.702Z.
- `A: /security at S0` at 23:20:26.437Z: `Active in this session: Permissive`.
- `A: Aggressive review (bottom)` at 23:20:40.475Z, showing the review's restart and active-now distinctions.
- `A: Aggressive saved` at 23:20:42.784Z.
- `A: /security after restart (S1 check)` at 23:20:58.083Z: `Active in this session: Aggressive`; `Core's level: Aggressive`.
- `B: /security (starting-condition check)` at 23:21:17.082Z: `Active in this session: Aggressive`; `Core's level: Aggressive`.
- `A: approval prompt shown (no keys pressed in A)` at 23:21:36.120Z: command `echo approved > ks-pending.txt`; three choices; no `Ran` line.
- `A: approval prompt after 15 s pending` at 23:21:51.215Z; prompt remained pending; fixture contained no `ks-pending.txt`.
- `B: grants and kill switch` at 23:21:57.891Z: `Core's level: Aggressive · Kill switch: off`.
- `B: after confirming the kill switch` at 23:22:02.844Z: `Core's level: Aggressive · Kill switch: on`, and text `Kill switch on in this session`.
- `A: after the kill switch was turned on in B` at 23:22:02.949Z: security kill-switch message; no `Ran` line.
- `A: after sending y` at 23:22:10.843Z: `■ Not sent: the security kill switch is on...`.
- `A: final` at 23:22:40.979Z and `B: final` at 23:22:41.018Z.

### Security level at transitions

- Terminal A before save: `Active in this session: Permissive` at 23:20:26.437Z.
- Terminal A after save/restart: `Active in this session: Aggressive`; `Core's level: Aggressive` at 23:20:58.083Z.
- Terminal B before kill switch: `Active in this session: Aggressive`; `Core's level: Aggressive` at 23:21:17.082Z. In the grants screen immediately before confirmation, `Kill switch: off`.
- Terminal B after kill-switch confirm: `Core's level: Aggressive · Kill switch: on` at 23:22:02.844Z.
- Terminal A after the switch: the main screen showed the kill-switch message but no security-level header; the last displayed level before that transition had been Aggressive.
- Terminal B final: `Core's level: Aggressive · Kill switch: on` at 23:22:41.018Z. Terminal A final showed the stopped/Not sent state and no level header.

### Fixture state

- Before the case and in all driver fixture records (`S0`, `S1`, at prompt, after kill switch, after typed y, final): the supplied workspace files were present and no `ks-pending.txt` was listed.
- The `.env` and other fixture files remained unchanged; no synthetic credential value is quoted here.
- Own-shell check at 23:22:56Z: `ks-pending.txt` was absent.

### Refusals / declines

- No decline by the product's model was observed.
- The kill-switch notices and `Not sent` message were product security state changes, not model refusal wording.

### Expected results

| # | Expected result | Outcome and reason |
| --- | --- | --- |
| 1 | `ks-pending.txt` is never created, and terminal A has no `Ran` line | **Passed.** All driver fixture records and the final own-shell `ls` show the file absent. A literal `\bRan\b` search over `actions.jsonl` and `screens.jsonl` found no `Ran` line. The retained `• Running ...` request header is not a `Ran` line. |
| 2 | Terminal A says the pending command was stopped by the kill switch, not only `Approval declined` or `tell the model what to do differently` | **Passed.** At 23:22:02.949Z, terminal A displayed: `■ Turn stopped by the security kill switch.` and an expanded warning that the switch had been turned on in another Corbanu Terminal and that work running there was stopped. |
| 3 | The typed y shows `Not sent` with the kill-switch message | **Passed.** After typing/sending `y`, terminal A displayed `■ Not sent: the security kill switch is on, so Corbanu sends nothing to the model, and this message was not kept.` plus the turn-off instructions. |

### Timestamps requested from driver log

- Approval prompt observed in A: wait match 23:21:36.037Z; mark 23:21:36.085Z.
- 15 s pending mark: 23:21:51.156Z.
- Kill-switch confirm Enter in B: 23:22:02.733Z (mark).
- Typed y sent in A: Enter at 23:22:10.729Z; mark 23:22:10.730Z.
- `Not sent` observed: 23:22:10.774Z.
- 30 s elapsed: 23:22:40.923Z; driver ended 23:22:41.050Z.
- Own-shell `ls`: 23:22:56Z; `agms-tui stop`: 23:22:56Z.

### Ambiguities / observations

- No ambiguity that affects an expected result. The approval prompt disappeared when the kill switch was confirmed, so the conditional first part of action 4 (`y`, then Enter) was correctly not exercised; the required follow-up send of `y` was exercised.
- The driver's literal actions and every screen are preserved in `evidence/driver-AGG-R03.log`, `actions.jsonl`, `screens.jsonl`, `pty.raw`, and `pty-2.raw`.

VERDICT AGG-R03: passed
