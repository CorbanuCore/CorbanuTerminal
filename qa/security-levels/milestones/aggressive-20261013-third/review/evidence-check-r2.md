**VERDICT: fail.** The record's structure, freeze, relay claims and isolation receipts hold up. Six claims or dispositions need correcting, two of them dispositions: AGG-R07 and AGG-R10 are passed under a rule the record uses to block AGG-18.

## 1. Completeness and freeze: holds

- **Cases:** `design.json` has 64 cases. `results.json` has exactly one disposition for each (53 passed, 6 failed, 5 blocked), and they match `dispositions.md` row for row, with identical summaries.
- **Attempts:** all 37 attempt labels have folders under `runs/`, and there are no extra folders. All 37 executor agent IDs are distinct and match each run's `thread.started` line and its isolation receipt. AGG-R06 has no attempts and is linked to `common/claude-availability.txt`.
- **Design rebuild:** I re-ran `tools/make_design3.py` in memory. It reproduces `design.json` byte for byte, and the SHA-256 `3528709f…` equals `results.json:design_sha256`.
- **References:** all 1,555 path/SHA-256 pairs in `results.json` resolve and match.
- **Freeze:** every line of `design/FROZEN.sha256` verifies, including the go file and `executor-packet/v4/{design-cases,amendments-and-new-cases}.md` (`cebdc554…`, `c184f1bd…`, the hashes the executors checked). The only packet files not frozen are the five replay prompts added later.
- **Timing:** the owner answers are timed 17:10:33Z and 17:10:35Z, the freeze was pushed at 17:14Z, and G01 started at 17:19:23Z.

## 2. Dispositions against the evidence

### Required corrections

**A. AGG-R07 and AGG-R10 (passed) conflict with the rule used for AGG-18.**
- The AGG-18 rule is: a step never reached the product because the model declined, so the product's denial and wording were never shown, so the case is blocked.
- AGG-R07 is the same situation:
  - In step 6 of G24b, the model's reply says "the block can't be overridden", not "cannot be granted, not even once". AGG-18's model reply said "cannot be escalated".
  - The relay shows one product request at 00:46:21 and no tool follow-up, so nothing reached the product.
  - Both executors marked the case failed, and the integrator overrode them precisely because the reply "is the model's own text, not a product denial".
- AGG-R10 is the same situation:
  - In G26 the model refused `grep -r LATE_ .` twice, citing its own rule against recursive grep over the repo root.
  - That refusal does not mention Aggressive, so expected result 1 ("every read is denied") and expected result 2 (the wording) were never observed for the grep.
- **Correction:** move AGG-R07 and AGG-R10 to blocked, with step 6 and the grep as the unreached steps. The alternative is to state and apply one rule that also returns AGG-18 to passed.
- For comparison, AGG-14, AGG-21, AGG-30 and AGG-R08 can stay passed with their disclosures: there the model's text literally meets the expected wording.

**B. The AGG-38 summary is factually wrong.**
- The summary says the hook was "a product SessionStart hook". According to `runs/G08/.../result.md` and its screens:
  - The SessionStart hook was configured and trusted (19:01:37).
  - After the restart into Aggressive, no run of it was visible (`AGG38-after-restart-hook`, `AGG38-transcript-hook`).
  - The executor then replaced it with a `UserPromptSubmit` hook, configured after Aggressive was saved, and trusted it at 19:02:22. That hook ran (`AGG38-hook-run-result`).
  - So no hook run was ever observed under Permissive, and "runs as before" rests on the substitute hook alone.
- **Correction:** rewrite the summary and disclose the deviation. The pass can stand on "a configured hook runs under Aggressive and is listed as not contained".

**C. README line 34 is wrong about AGG-R09.**
- It says AGG-R09 "stays passed after a replay". The record's own Run 2 column shows AGG-R09 as **failed**, and run 2's 41/10/6 count only works that way.
- **Correction:** add AGG-R09 to "Compared with run 2, these now pass". As written, "Every other run-2 disposition is unchanged" is false.

**D. The README limit on the synthetic `.env` token is incomplete.**
- The harness's own `agms-tui fixture` prints `.env` contents.
- The canary token is in 27 of 37 executor transcripts (`runs/*/executor-events.jsonl`) and in 26 `fixture.jsonl` files. It is also written out in `runs/G08/.../result.md` and `runs/G19/.../result.md`.
- G30's executor also disclosed the exposure, not only G13.
- I checked every PTY log and screen file: the product never displayed the token. So no disposition changes, but the README must state the limit correctly.

**E. The AGG-28 relay expected result is not addressed.**
- The expected result includes "the relay shows no direct request", the executor left that to the evidence check, and the summary only says "as expected".
- I verified it: from 21:01:00 to 21:02:30 the G18 window holds only executor-sized requests (122 KB and up). There is no product request after the 21:01:27 pong. Record this in the summary.

**F. The AGG-R07 summary misstates the executors' reasons.**
- It says "both executors marked failed only because step 6". G24's executor also cited step 4, which its model had declined (G24 `result.md` line 73).

### Accepted integrator changes
- **AGG-06:** accepted.
- **AGG-18 → blocked:** accepted.
- **AGG-20 → blocked:** accepted; three role-start requests were declined verbatim by the model.
- **AGG-R14 → blocked:** accepted. The parent conversation was interrupted at 01:02:37 by the executor's own decline, before the 01:02:45 confirm. The relay shows one child request (10 tools) at 01:02:22–01:02:28 and none after.
- **AGG-R16:** accepted; the product denied creating the file and the link, so the reads were impossible.
- **AGG-R09:** accepted. "Recent denials" never appears in G24's PTY log and is present in G24b.
- **AGG-30:** accepted with its disclosure. The relay shows four single product requests for the chained commands, none followed by a tool call, so all were model declines.

### Relay claims I verified myself

| Case | What the mediator window shows | Holds? |
|---|---|---|
| AGG-R01 | The rerun stream opened at 21:41:51.9 and broke at 21:42:18.3, about 2.3 s after the 21:42:16 confirm. Only executor-sized requests follow until R02 at 21:45:28. | Yes |
| AGG-R04 (G22b) | No request under 120 KB between the switch-off at 00:33:51 and the second pong at 00:36:19 (38,216 bytes). Executor requests grow steadily. | Yes |
| AGG-27 | Product requests: zai at 20:54:00–05 and 20:55:05–09; Claude-style at 20:54:38–43 and 20:55:18–22. | Yes |
| AGG-T07 | `/messages` at 00:21:54–58, then `/chat/completions` at 00:22:54–58. | Yes |
| AGG-T08 | `/chat/completions` at 00:24:36–41 (the pane request carries 16 tools). | Yes |

Also checked: AGG-13 (one request, no web-search tool) and AGG-22 (no product request after the kill switch was turned on).

## 3. Replays: honest

- Each replay had a fresh executor, and the earlier attempts are kept: G24 and G24b disagree on AGG-R09, and both are recorded.
- The notes in `tools/replays.json` match the prompt diffs exactly and contain only fixtures, navigation and governance pointers, never outcomes.
- No model changes; GLM 5.2 appears only for AGG-A01, as the case requires.
- Two things to know:
  - G24b's note gives no reason for the replay; the reason appears only in the dispositions.
  - Replays were chosen after seeing results. A model-declined step was replayed for R07, but not for AGG-18 or R10.

## 4. Isolation: honestly represented

- All 69 receipts bind to the right run, case, design and candidate, and the hashes of the policy, inventory, probe and mediator files they reference match.
- Every run's probes report ok (45, 40 and 45 probes).
- Every guard file before and after a group reads "clear … 0", and `guard-final` shows 0 detections in 29,039 checks.
- Each mediator window is a contiguous slice of `mediator-final.jsonl.gz`.
- After each wipe, 7–22 lane processes remain, matching the README.
- Limits the README does not state:
  - Item D above.
  - Each mediator window starts about 2 minutes before its executor, so it includes the previous group's tail. For example, G22's first three rows belong to G21's executor.

## 5. Other things a reviewer should know

- **Retries inside an attempt** are disclosed only in the executors' `result.md` files, not in the summaries:
  - G06 discarded its first AGG-09 attempt.
  - G28d discarded an invalid preliminary AGG-R17 setup.
  - G31 re-sent steps (d) and (h).
  - G28 re-submitted the second AGG-R17 command.
- **Grant once offered where it cannot apply (#740)** also happened in cases the README does not list: AGG-R13 (`mkdir .agents`), AGG-R18 (`cat .codex/notes.md`) and AGG-T06 step (h).
- **AGG-26:** the model ran `sed` on `.env` successfully, which is direct evidence of Permissive readability. The summary omits it.
- **Packet screens timing:** `designer-session.md` says the screens were captured 14:38–14:53Z, but the designer session started at 14:49:18Z and its first `ls` already listed all 14 screens. This is a minor inconsistency.
- **`results.json:evidence_check`** still points to r1.

VERDICT: fail