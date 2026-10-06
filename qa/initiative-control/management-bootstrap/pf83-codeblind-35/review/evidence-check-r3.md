# PF-83-S01 campaign 35: evidence check r3 (second corrective re-check)

I worked only in the record directory, read no product source, repository files or history, and changed nothing. All the corrections that block acceptance are in place. A few small traceability gaps remain, and none of them misstates the evidence.

## Defect status

| Defect | Status | What I checked |
|---|---|---|
| r2 A: restore egress claim had no artifact | **Fixed** | `common/restore-record.txt` now ends with a post-restore section dated 02:07:58Z. It shows `api.z.ai` returning 301 and curl rc 6 for github, raw and codeload. It also shows 0 pf83 accounts, `/opt/pf83` absent and 0 campaign directories. README matches it. |
| r2 B: F10 limitations not represented | **Fixed** | `dispositions.md`, the `results.json` summary and README all now record the limitations, and F10 is **blocked**: the variants ran in one session, restart was a tmux kill, the Esc recovery route was not exercised, and the deviations have no product-authority acceptance. The (c) pre-stop screen gap is in `dispositions.md` and `results.json`, but not in README. |
| r2 C: F08 basis imprecise | **Fixed** | The basis now says Cancel+Enter and Esc both cancelled from the restricted level and the next probe (01:06:56) was gated. The screens and transcript confirm it (`wait-matched:Would you like to run` at 01:06:59, then `fail-probe-deny`). Approval stayed `on-request` throughout; the approval reviewer was briefly `auto_review` at 01:05:26. |
| r2 D: traceability gaps | **Partly fixed** | Fixed: `other_attempts` for F03, F04 and F10 now hash-binds each attempt-1 result, screens, actions, fixture, rollouts, mediator window and stderr (13, 13 and 19 refs). Fixed: the `/tmp` gap now has a receipt ("files owned by uid 602-604 in /private/tmp and /Users/Shared: 0"). Still open: PF83-DEF-015 is now worded as a reference to the sprint record and marked untested, but that record is not in this directory. |
| r1 7: executor deviations and replay authorization | **Fixed** | All deviations are listed, and the replay reason and the one-replay cap are recorded. The `/tmp` claim now rests on the restore receipt. I checked "both curl calls were denied": the first produced the one denied `CONNECT example.com`; the second ran under `workspace-write` with `network_access:false` and returned empty output, with no CONNECT logged. |

## Integrity of `results.json` and agreement across files

- **Hashes:** all 259 path/sha256 refs exist and match, including the nested `other_attempts` evidence. `design_sha256` and the candidate manifest hash match their files.
- **Identity binding:** I checked all 14 attempts, including the other attempts. In each one:
  - the `isolation.json` agent equals the single `thread.started` ID in that attempt's executor events;
  - the `run_id` appears in `probes.json`;
  - enforcement is `os-enforced`.
  - For each case, the execution `agent`/`run_id` match the attempt it cites, and every evidence path stays inside that attempt's directory.
- **Counts:** passed 3 (F01, F02, F06), failed 1 (F05), blocked 7. These agree across README, `dispositions.md` and `results.json`. There are 14 run directories plus 4 aborted, matching the claim.
- **Mediator log:** 1136 forwarded requests, all status 200, all to `/api/coding/paas/v4/`. The only "anthropic" string is in the startup allow-list row, so the F11 claim holds. The mediator-window disclosure for concurrent lanes is now in README.
- **Review ledger:** the r1 and r2 thread IDs match the reviewer event logs.

## Spot checks

- **F05 (failed):** confirmed.
  - At TRY_TYPE (00:55:34) the screen shows "You approved … always run" with `ermissions` left in the composer.
  - Escape at 00:56:26 came before `/permissions` at 00:56:34.
  - F05B_DECLINED (00:56:31) shows "✗ You canceled…" followed by "• Ran … └ (no output)".
  - FA_APPLIED shows the next-turn disclosure.
- **F06 (passed):** confirmed.
  - The settings changes at 01:01:49 and 01:03:49 fall inside active turns `ccca` and `7093`.
  - The fixtures show each slow operation started and then finished.
  - The 01:02:33 turn is `workspace-write` and its probe is gated.
  - The 01:04:21 turn is `danger-full-access` and its probe ran without approval; `run2-final` has the probe marker.
  - The screens show "applied for NEXT TURN" in both directions.
- **F10 (blocked):** confirmed. Attempt 2 restarted four times with `start resume`, each after a `stop`. The "stop it with Esc" hint appears (Vc-RESTRICTED-APPLIED), and the executor's report says no separate route was found. That supports the reasons for blocking.
- **F08 (blocked; my choice):** confirmed, as described under defect C.

## Remaining evidence defects

None that block acceptance. These are follow-ups:
1. **PF83-DEF-015** (README product finding 1): the reference to a "later source correction" points to a sprint record outside this directory. It is labelled as untested, so it does not misrepresent the evidence, but it cannot be verified here.
2. **Restore "back to original" modes:** the pre-campaign baselines that `provision.sh` saved (`home-modes.txt`, `pgsock-mode.txt`) are not in the record. The final modes are recorded; the originals are not.
3. **Reviewer model:** README and the ledger name `claude-opus-5-5-plan, high`, but the reviewer event logs only bind thread IDs.
4. **F02 summary in `results.json`:** it says "probe in pending interval" without the qualifier in `dispositions.md` that the interval was idle, so the probe was the first turn after the boundary.
5. **README F10 sentence:** it omits the (c) pre-stop screen gap, which is in `dispositions.md` and `results.json`.
6. **Forward references:** `results.json` has no `evidence_check` entry yet, and README and the ledger point ahead to r3. `make_results.py` has to be re-run with the r3 artifact, which will change the `results.json` hash.

EVIDENCE-CHECK VERDICT: pass