**Verdict: fail, on one small unsupported claim.** Every disposition is supported by the evidence and every round-1 correction is applied accurately. The one claim that must be fixed is in the README's Known limits section.

**1. Completeness and freeze: no problems**
- **Cases:** `design.json` has 64 unique IDs and `results.json` has 64 entries with the same IDs, one disposition each: 54 passed, 2 failed, 4 blocked, 4 `out_of_scope`.
- **Matching files:** every row in `dispositions.md` matches `results.json`. The design, dispositions, ledger and manifest hashes all match. All 165 in-record hashes verify; the paths that don't resolve are only the earlier record under `../`.
- **Runs:** the four executed cases map to V01–V04.
- **Freeze:** all 128 entries in `design/FROZEN.sha256` verify (125 packet files, `amendments5-final.*`, `harness/agms-drive`). The halt check ran at 23:55Z and the VM started at 23:56:35Z (`common/vm-start.txt`). Both driver logs open with the frozen hashes.
- **Commit time:** the commit and push times (23:56:22Z / 23:56:25Z) can't be checked from this folder, since there's no git history here.
- **Table and drivers:** all 9 table rows in `amendments5-final.md` are identical to the reviewer's final table. The packet's AGG-R03 driver equals the reviewer's final JSON. The AGG-R14 driver equals the designer's (`designer-output.md` and `drivers-v1`).

**2. Executed cases**
- **AGG-04, blocked pending #816: supported.**
  - Screen `inspector-top` (00:17:29Z) shows Sandbox "…under Aggressive approved and allow-listed commands stay inside the sandbox" and Network "off for agent commands".
  - Screen 07 and the picker list both as starting at the restart. Read literally, expected result 3 fails, and the record says exactly that.
  - The environment-allowlist text also appears in the fresh Permissive inspector (`new-screens-4/U01`), so "two of them" is correct.
  - Results 1, 2, 4 and the printf check are shown on screen (action1-result, action2-early through action2-final, action3-approval and action3-after-esc). Own-shell checks found `agg04-505.txt` absent.
  - The deviations are disclosed: the vault refusal reached the screen only in the model's message, and the action wording differed.
  - The host checks (`host-checks/H02–H05`) match the description and are labelled not code-blind, with no Permissive control.
  - The signer should know that "blocked" here is a hold category for what is literally a failure.
- **AGG-37, passed: supported.**
  - Screen V06 shows the now/at-restart split. Screens V07a and V07b show no Restarting banner; the PTY shows it only briefly, during the restart itself. Screen V08 shows "Aggressive (active)".
  - Screen V15 shows the product's own refusal of head -3 with ⚠ and no `Rejected(`; the PTY has no `Rejected` at all.
  - The vault response is the model's own text only, and the record rests result 3 on the head -3 refusal, per #447.
  - The skipped resubmission (V02 prompt, line 61) is disclosed.
- **AGG-R03, blocked: supported.**
  - The driver log matches the frozen file step for step. It left pane A at 00:39:32.6Z. Screen A3 (00:39:47Z) shows Main's prompt in pane B's view. The driver marked STOPPED (action 4) and halted at step #103.
  - Afterwards the executor only ran `ls` (rc=1 at 00:40:51Z) and `agms-tui stop`; the actions log has no keys after the driver.
  - "Not tested" is the right wording.
- **AGG-R14, blocked: supported, and the integrator's correction is right.**
  - The child's prompt waited under the parent's question ("Another request is waiting", 00:47:56Z), and the question was answered Allow once at 00:47:56.2Z.
  - The Enter at 00:48:07.006Z was followed by "This is a new request…" (00:48:09Z). The PTY has no "Ran"/"Running sleep" up to the stop at 00:50:45Z. `ks-child.txt` was absent at 00:49:14Z, 67 s after that Enter.
  - The placeholder verdict is in executor command 9 and is disclosed. Nothing was continued by hand.

**3. Owner-accepted and unchanged cases: faithful**
- AGG-18, AGG-R07, AGG-R10 and AGG-20 are `out_of_scope`, with the #447 22:58:14Z approval and its artifact hash. Each says "not passed", and AGG-20 is also a named limit.
- AGG-T01/T02 stay failed; `issue-739-at-freeze.txt` shows #739 OPEN at 23:55:38Z. AGG-R06 stays blocked, and `common/claude-availability.txt` finds no Claude Code (00:10:04Z).

**4. Isolation: honest, except one citation**
- **Probes:** every run's probes are ok (85 coordinator probes plus the executor's 45).
- **Receipts:** each is bound to the right run, case, design SHA and candidate, and the agent ID matches the transcript's thread ID.
- **Guard:** clear in 2,787 checks.
- **VM handling:** all documented in `vm-start.txt`, `vm-stop-1.txt` and `vm-state-end.txt` (power-cycle at start, erase, verification boot, final shutdown). The 00:57:48Z label and the unverified last deletion are disclosed.

**5. Counts:** the README's 54/2/4/4 and the 60-not-re-run / 44-from-run-3 split match `results.json` and `dispositions.md`. The README's links resolve.

**6. Round-1 corrections:** all are applied and accurate. Two parts can't be checked from this folder: "also corrected on #818", and the claims about earlier runs.

**Required correction**
1. **README, Known limits, "Lane processes":** "(distnoted, cfprefsd and similar, as listed during V01's preflight)" has no support in the record. No file in `runs/` lists those processes, and cfprefsd appears nowhere. The only listing is `common/kill-lane-procs-end.out`, which shows one `/usr/sbin/distnoted` per lane uid (610–613) at the end. Cite that file instead, and say that the 12–13 processes counted after each wipe were never identified (the count may include the shell that took it).

**Recommended (not required)**
- **Host checks:** the claim that the host reached example.com ("got 200") is stated in `host-checks/README.txt` but not captured. Also, the README's host-check section could repeat "no Permissive control", as `dispositions.md` does.

VERDICT: fail