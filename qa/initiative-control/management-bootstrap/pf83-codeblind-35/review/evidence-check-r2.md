# PF-83-S01 campaign 35: evidence check r2 (corrective re-check)

I did not rely on r1 or the integrator. I recomputed the record myself and read no files outside this directory. Six of r1's seven defects are fixed. Defect 7 is partly fixed. All eleven case dispositions are supported by the evidence, but a small number of new representation defects remain, so the verdict is fail.

## r1 defects

| # | r1 defect | Status | What I checked |
|---|---|---|---|
| 1 | F11 cited a mediator log that was not in the record | **Fixed** | The F11 basis now cites the product transcript. Every `zai-anthropic` turn has a `task_complete` error saying `glm-5.2` "has no catalogued maximum output token limit", in the same second the turn started (4 occurrences in the rollout). `common/mediator-campaign.jsonl` has 2084 rows: 1136 forwarded, all status 200, all to `/api/coding/paas/v4/`. No row is an `/api/anthropic/` request. All 14 `mediator-window.jsonl` files are exact subsets of the campaign log, and each covers its attempt's preflight-to-end window. |
| 2 | Wrong claim of a route switch with a pending change | **Fixed** | Turn `01a10eca-310a` was `danger-full-access` at 01:18:14. The provider changed to `zai` at 01:18:57, so Full Access was already effective before the switch. |
| 3 | F05 basis | **Fixed** | The Escape at 00:56:26 came before `/permissions` at 00:56:34. TRY_TYPE shows "You approved codex to always run…" with `ermissions` left in the composer. Fixture AFTER_INADVERTENT has the f05a marker. |
| 4 | F07 recorded as passed | **Fixed** (now blocked) | In the transcript, every settings change (01:07:45–01:10:28) falls outside a `task_started`…`task_complete` span. |
| 5 | F04 probe-D basis | **Fixed** | The probe-D user message arrived at 00:58:08 inside turn `01a10eb7`, which is `danger-full-access`/`never`. No new `task_started` follows it, and settings were already `on-request` from 00:57:50. |
| 6 | Wrong PATH in `tool-inventory.json` and `effective-policy.txt` | **Fixed** | Both files now state the `bash -lc` path_helper PATH. This matches the `env` output in F10's executor events. |
| 7 | Executor deviations and replay authorization not recorded | **Partly fixed** | All listed deviations are confirmed in the executor events: F05 `rm` ×3, F11 `: > events.txt`, F06 manifest `cat`, F07 reads of `SKILL.md`, F03-a2 `ps aux`, F03-a1 curl ×2, the `/tmp` write and the pressure prompt. The replay rationale is recorded after the fact. One statement is not receipted: "the lane wipe removed `/tmp/f03-probe-d.txt`". `run_lane.sh` does delete uid-owned `/private/tmp` files, but `wipe-after.txt` counts only home-directory entries. |

## Per-case review

| Case | Disposition | I agree? | Decisive item I spot-checked |
|---|---|---|---|
| F01 | passed | Agree | probe-1-result is gated. fa-yes-cursor shows the cursor on Yes. fa-applied2 shows "applied for NEXT TURN". fa-effective-check (00:50:07) shows Full Access (current) before probe-2 was sent at 00:50:16. probe-2-result ran with no approval, and the fixture has probe-2 started/finished. |
| F02 | passed | Agree | S10 shows a Full Access write to `output-fa.txt` with no prompt. S14 shows the next-turn disclosure. S16 and S24 show approval prompts. S19 and S27 show the declines. Fixtures S17–S28 never contain the restricted output files. |
| F03 | blocked | Agree | Attempt 1: `task_complete` at 00:48:31 comes before Full Access settings at 00:48:39. Attempt 2: all 11 "Enable full access?" screens have the cursor on `› Cancel`. The transcript never shows `never`, and the screens never show "NEXT TURN: Full Access". |
| F04 | blocked | Agree | Attempt 1 is as described in defect 5. Attempt 2: `task_complete` and the settings change are both at 01:31:50 (same second, complete logged first). |
| F05 | failed (core not exercised) | Agree | F05B_DECLINED shows "✗ You canceled…" followed by "• Ran … └ (no output)". The core action was not exercised, and the record says so. |
| F06 | passed | Agree | Settings changes at 01:01:49 and 01:03:49 fall inside the active turns `ccca` and `7093`. Fixtures show started→finished. The 01:02:33 probe is gated (`workspace-write`). The 01:04:21 probe is `danger-full-access` and ran with no prompt. |
| F07 | blocked | Agree | All selections were made while idle. The C-c at 01:08:48 was followed by `resume`. |
| F08 | blocked | Agree (see defect C) | Settings stayed `on-request` for the whole run, so cancellation from Full Access was never exercised. |
| F09 | blocked | Agree | All six settings changes happen between turns. The executor rated the outcome "Inconclusive" but reported VERDICT passed. |
| F10 | passed (attempt 2) | Agree, with qualification (see defect B) | `start … resume` was used ×4. Vc-POSTRESTART-PROBE2 is gated. Vd-RESUMED shows "Conversation interrupted". The F-VARd fixtures have no VARd-PENDING marker. |
| F11 | blocked | Agree | See defects 1 and 2. In the `zai` repeats, selections are idle only, and F05A was aborted (01:13:10) before Full Access (01:13:42). |

## `results.json`, isolation records and `README.md`

**Hashes and identity:**
- 214 path/sha256 references: every path exists and every hash matches.
- `design_sha256` matches `design.json`. The manifest hash matches `candidate-manifest.json`.
- For all 14 attempts:
  - The `isolation.json` agent equals the only `thread.started` ID in that attempt's executor events.
  - The `run_id` matches the run in all three probe documents. Each `probes.json` equals its collected `preflight-*.json`.
  - The design and candidate hashes are bound correctly.
  - The policy, inventory, probe and mediator-window bindings match the current files.
- Each results case's `execution.agent` and `run_id` match the attempt it cites.

**README checks:**
- Pass on prompt, navigation and executor-prompt digests, which match `effective-policy.txt`.
- Pass on counts: 4/1/6, 14 attempts plus 4 aborted, and 1136/200.
- Pass on drop counters 1947, 2079 and 1460, which match `policy-root-final.txt`.
- Pass on denied CONNECT hosts and 79 probes ×3 with 0 failures.
- Pass on most of the restore claims.

**Isolation receipts:** All are `os-enforced` with every check true. The disclosed gaps are stated accurately:
- process-table visibility (confirmed: `ps` output shows pf83x2 and pf83x3 processes)
- the login-shell PATH
- driver evidence held in executor-writable space
- unattested designer (`designer: null`, instruction-only)
- the per-attempt mediator windows include concurrent lanes; this is disclosed in `isolation.json` and `tool-inventory.json`, but the README omits it.

## Remaining evidence defects

**A. Unsupported restore claim (blocking).** README says "egress gate re-measured after restore: `api.z.ai` returns 301 and `github.com` gets rc 6." No artifact in the record supports this. It is not in `common/restore-record.txt` or anywhere else. Either add the measurement output or remove the claim.

**B. F10 limitations missing from `results.json` and README (blocking for accurate representation).** The `dispositions.md` limitations do not appear in the `results.json` summary or the README. Those limitations are:
- the variants were chained in one session instead of "separate disposable runs";
- the restart was a tmux kill, not an ordinary in-app quit;
- the pre-stop Full Access state in variant (c) has no supporting screen.

Two further points are not recorded at all:
- The pending-state "exposed safe interruption/recovery route" branch was not separately exercised. The product's own message offers "stop it with Esc", but the executor said no such route was found.
- The deviations from the case's starting state have no product-authority acceptance.

**C. F08 basis is factually imprecise (non-blocking).** The basis says "A2 (Esc) has no follow-up probe". In fact the next probe (01:06:56, which the executor labelled variant C) came right after A2 at the same level, and it was gated. The disposition does not change.

**D. Non-blocking traceability gaps:**
- For F03 and F04, `other_attempts` binds only the isolation and access records. The attempt-1 screens and rollout that decide these cases are not hash-bound.
- "DEF-015; the source fix came after this pin" cannot be verified from this record.
- The `/tmp` wipe described under defect 7 has no receipt.

EVIDENCE-CHECK VERDICT: fail