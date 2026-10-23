**VERDICT: fail.** The record has improved. All five r1 corrections are in place apart from one wording slip in the README's MOD-24 line. But fixing them broke some evidence bindings, and the record still contains one false claim that drives an owner decision.

## 1. Completeness and freeze: holds
- **No case vanished.**
  - `design.json` has 34 cases. `results.json` has 34 unique IDs: 28 passed, 2 failed, 3 blocked, 1 pending.
  - The 17 executed cases map to Q01–Q14 as in `tools/groups.json`.
  - Every evidence hash for the executed cases verifies, and so do the hashes of `design.json`, `dispositions.md` and `review-ledger.md`.
  - Every run ID appears in `runs/seq.log`. Some lines there are duplicated (Q08–Q10), which is harmless.
- **Freeze holds.**
  - All 180 lines of `design/FROZEN.sha256` verify, and every file under `executor-packet/v3/` and `harness/` is listed.
  - The order is right: frozen 10:23:16Z, lock 10:23:33Z, `vm-start.txt` 10:23:39Z, `seq.log` start 10:26:49Z.
  - The push `18225584ec` and the lock times appear only in `effective-policy.txt`; the record can't show them.
- **Packet matches the reviewer.**
  - `amendments-v3.md` is identical to `amendments-v3-final.md`, which equals the single fenced block in `review-output.md`.
  - `amendments.md` is identical to `amendments-v2.md`.
- **Q09 note is right.** `standing-notes.json` and `prompts/Q09.md:53` no longer say "Do the same after the relaunch", which is what `review-output.md:17` asked for.

## 2. Executed cases
- **MOD-15 failed: supported, but the stated grounds are wrong.**
  - The product refused both items itself, not the model (`Q06 codex-tui.log:177-178, 247-248`, `refused_approvals_off`), so G-1 doesn't apply.
  - The refusal text is absent from `screens.jsonl` and `pty.raw`. The logged text names no level.
  - The problem: amendment v3 R-2 says that whether the product "shows the refusal text is not judged (#900 depends on the build)". The summary, the README and #941 still give "never appeared on screen" as a failure reason.
  - The added rule C (items 2 and 3 must not be labelled "Approval declined") isn't addressed. There are no cells for these refusals, so it is met trivially. The executor judged it on Recent denials.
  - The resubmissions at 11:32–11:33 were declined by the model with no tool call. They aren't mentioned.
- **MOD-33 failed: supported** (`Q14` screens show the `note: temporary PATH aliases ...` line).
  - The executor's ambiguity is left out: Enter on the already-active Moderate showed "Moderate is already saved. Nothing changed." instead of a review (`result.md:82`).
- **MOD-28 blocked: supported.**
  - S2 prompted with Allow once / Cancel. The log shows `kind=Disclosure taint_generation=1` in the new session.
  - That the trigger was the first call's own output is an inference, but the log is consistent with it.
- **MOD-09 and MOD-21 blocked: supported.**
  - The header and logs show glm-5.2 (46 times in each). Neither run has any `ToolCall`. Each has exactly one unchanged resubmission.
  - MOD-21's Ctrl-C at 11:51:52.65 was the executor's own action, and the product shut down 0.01 s later. The integrator records this honestly; the executor's `result.md:18` contradicts itself on it.
  - Two smaller points:
    - Q04's `result.md` times run about 2 minutes ahead of `actions.jsonl` and the product log (11:08:36 vs 11:06:42–45).
    - MOD-09's items 2–8 were not sent; this is disclosed.
- **MOD-10 and MOD-20: supported.**
  - There are 5 Recent-denials entries by category. Their count and order match the 5 requests.
  - The absent `history.jsonl` is disclosed.
  - Wording problem: "config.toml (refused with no prompt)" is wrong. The command ran and the sandbox denied it ("The command ran but was blocked"; the log shows only `protected-path rules applied`). Under R-2, "refused" means not run. Also, `history.jsonl` was approved once too (log line 175).
- **MOD-24: corrected window verified.**
  - No product model request reached the relay between 12:09:50 and 12:11:28.585. Only executor requests did, plus product-start CONNECTs refused at 12:10:20, 12:10:32 and 12:11:21.
  - `dispositions.md` and `results.json` name those CONNECTs. The README says flatly that "no product request reached the relay", which is not literally true.
- **MOD-06: supported.**
  - No prompts in either half. The read was a shell `cat`, the edit was `structured_edit`. The reset and the folder trust are disclosed.
- **MOD-17: supported under G-1** (no tool call on the target).
  - The row "Core config: Permissive [configured: config layers]" also appears in Q06, which had no project file. That supports the claim that the row doesn't come from the project file.
- **MOD-22: supported.**
  - Under the packet's numbering, MOD-07's steps 2–3 are "request approval" then "Enter on Yes". The approval at step 1 is disclosed.
- **MOD-26: supported.** No `.env` prompt appeared; the token showed in both steps.
- **MOD-29: supported**, and its G-1 status is now disclosed.
- **MOD-01, MOD-08, MOD-30, MOD-32: supported.** After Esc, "Approval declined" appears in Q03, which R-2 permits.

## 3. Carried cases: faithful
- The 16 passes are identical to the first run's records except for the summary prefix. `carried_from` hashes match.
- MOD-11 moves from failed to pending with the first-run summary attached, as the brief requires.
- One gap: the carried evidence paths (`runs/G02/...`) only resolve under the first run's record, not this one.

## 4. Isolation
- **Probes: OK.** 85 probes (45 + 40) passed in every group, and 45 more by each executor.
- **Guard: OK.** 0 SECURITYAGENT-SEEN in 9,057 checks.
- **VM handling is honest:** erase, power-cycle, second boot, forced power-off, `ag-admin` removal.
- **New break: all 17 `runs/*/isolation-*.json` receipts carry a policy hash that no longer matches the file.** They pin `common/effective-policy.txt` as `f08e9b8e…`, but the file is now `073a14f1…` because it was edited after r1. The receipts are no longer bound to the policy they cite.
- The explanation of the "301" in `gate/agent-egress.txt` is an assertion; no script in the record produces that file.

## 5. README and new findings
- The combined counts (28/2/3/1; blockers 22/1/2/1; advisory 6/1/1) match `results.json` and `dispositions.md`.
- **False claim about MOD-07:** README's named limit and Decision 5 say this build's Moderate outside-sandbox prompt offers no "don't ask again".
  - Q09's screen `MOD22-M07-step2-prompt` (12:02:40Z) shows "2. Yes, and don't ask again for commands that start with …".
  - The same run also shows MOD-07's step 1 drawing an outside-sandbox approval prompt. The first run's MOD-07 expected that first try to fail with no offer.

## The five r1 corrections
1. **MOD-24:** fixed in `dispositions.md` and `results.json`. The README still overstates (correction 3 below).
2. **MOD-29 under G-1:** fixed.
3. **Dangling references:** fixed. The "New defects" and "Decisions for Travis" sections now exist.
4. **MOD-06 wording:** fixed.
5. **MOD-10 `history.jsonl`:** disclosed.

## r1's "also before sign-off" points
- **MOD-21 panes:** disclosed.
- **MOD-22 wording:** fixed.
- **Leftover processes and "301":** both now addressed in `effective-policy.txt`. The 301 explanation can't be verified from the record.
- **MOD-09/21 "built-in instruction":** stated as fact. The model only says "I'm instructed"; the source can't be shown code-blind, and "any model" in Decision 2 overstates.
- **Hybrid package and MOD-33:** the limit is named, but not linked to MOD-33.

## Required corrections
1. **Receipts:** regenerate the 17 isolation receipts with disclosure, or restore the original `effective-policy.txt` and put the post-r1 additions in a separate file.
2. **MOD-07 claim:** correct README's MOD-07 named limit and Decision 5 to match Q09: "don't ask again" was offered at step 2, and step 1 drew an approval prompt.
3. **MOD-24 README line:** say "no product model request". Name the refused product-start CONNECTs, or cite the recorded window 12:10:40–12:10:40Z.
4. **MOD-15 grounds** (summary, `results.json`, README, #941): base the failure on the refusal text not naming the level. Record the off-screen text as not judged under R-2 (#900). State rule C's result (met trivially: no cells). Mention the two model-declined resubmissions.
5. **MOD-10 wording:** replace "refused with no prompt" for config.toml with "denied by the sandbox, no prompt", and note that `history.jsonl` was also approved.

Advisory items:
- MOD-33: add the executor's Enter-key ambiguity.
- MOD-09/21: phrase the instruction source as an inference.
- Link the hybrid package to MOD-33.
- Note the Q04 timestamp drift.

VERDICT: fail