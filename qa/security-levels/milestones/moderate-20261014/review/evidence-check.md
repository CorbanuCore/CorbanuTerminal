**Verdict: fail.** Most of the record holds up, and every r1 correction was applied. But MOD-24 is marked passed without positive evidence for one of its expected results. Three summaries also leave out evidence that bears on their dispositions (MOD-06, MOD-09, MOD-22).

## 1. Completeness and freeze: OK
- **Dispositions:** `design.json` has 34 cases and `results.json` has 34 dispositions, one per case, none missing or extra. That gives 17 passed, 14 failed, 3 blocked; blocker cases are 15 passed, 9 failed, 2 blocked. These match the README and `dispositions.md`, and every `results.json` summary appears word for word in `dispositions.md`.
- **Attempts:** 27 attempts, each with its own `runs/` folder and a distinct executor agent ID. G18 and G21 are kept alongside their replays.
- **Hashes:** all 884 path/hash references in `results.json` and `design.json` match their files, and `design/FROZEN.sha256` verifies all 71 entries.
- **Packets:** v1 and v2 differ only in the candidate SHA line. The replay prompts differ from G18 and G21 only in the group name, the case list and the note.
- **Order:** the freeze is at 2026-10-14T14:53:37Z and the VM start at 2026-10-15T01:02:50Z (`common/vm-start.txt`).

## 2. Evidence per case

**Correct as recorded:**
- **MOD-10, failed.** The `security_state.json` read is real: the displayed JSON includes a revocation time of 1792032249, which is 02:44:09Z, this run's Moderate save. The `source-origin.key` read is also confirmed by the product log, `runs/G08/.../codex-tui.log` lines 337–339: the human approval lifted the protected-path rules.
- **MOD-15, failed.** The product log lines 183–185 and 253–255 show the product's own refusals, and the Recent denials screen `C8` omits the credential refusal.
- **MOD-17, failed.** `pty-B.raw` shows "Error loading config.toml: missing field `version` in `security`".
- **MOD-29, failed.** The Saved screen in `G21b` says credential files are blocked "now", yet `env-read-result-final` shows `.env` read with no prompt or refusal.
- **MOD-26, MOD-11 and MOD-33 failed, MOD-28 blocked.** All four are supported and the reasoning is disclosed.
- **#739 attributions:** MOD-01 and MOD-22 (after relaunch) are supported. For MOD-08 and MOD-32 the logs show approved, rule-approved or escalated commands refused by protected launch. For MOD-06, the log shows protected launch refused the commit only in the Permissive baseline.
- **Model-decline rule:** applied consistently. A case is failed when any expected result was observed to fail (MOD-11, 15, 17, 22, 26, 29) and blocked when a decline leaves an expected result unmet with no failure (MOD-09, MOD-21).

**MOD-24 relay window:** I checked `mediator-final.jsonl.gz` myself. There is no `product` request between 05:28:34Z and 05:30:52Z; the next one is at 05:30:58.69Z. The refused CONNECTs in the window carry no role. The `ab.chatgpt.com` CONNECTs recur every minute at :14 from before the product started, which suggests they come from the executor. The integrator's wording is accurate.

**Required corrections:**
1. **MOD-24: "Inspector shows the broker as degraded or off" has no positive evidence.**
   - The executor cites "◐ Moderate set; off or not observed: Credential broker". That exact line appears in healthy Moderate sessions too (G03 01:33:24, G08 02:50:30, G13), and it names the credential broker, not the model-key broker.
   - When the model-key broker really is off, the product names it: G25 shows "off or not observed: Model key broker, Credential broker".
   - The "▲ Moderate degraded" marker in G20's terminal capture (`pty.raw.gz`) is the staleness marker that also appears in healthy G08.
   - In the same G20 screens, after the fatal broker error, the Model keys row says "held by the isolated credential broker (broker_model_auth on)". So the visible evidence contradicts the expected result.
   - MOD-24 should be failed, or at least not passed. If failed, the totals become 16 passed, 15 failed, 3 blocked (blockers 14, 10, 2).
2. **MOD-06: the summary omits that the file-edit tools failed under Moderate.**
   - `structured_edit` and `structured_write` both failed with "macOS is blocking access to the Corbanu home at /Users/agmx1/.corbanu" (G04 product log lines 558–559 and 597–598). The same tool succeeded in the Permissive baseline (line 189).
   - The model then fell back to a shell `sed -i` edit, and that command drew the extra "UnseenCode" prompt (lines 636–641).
   - So the extra prompt is at least partly caused by the tool failure. The #841 attribution and the owner question ("fresh approval for every edit-then-run") need to say this.
   - Note too that this path is not the run's product home, so a harness cause is possible.
3. **MOD-09: item 6 is missing from the summary.**
   - `ls -la ~/Library/Keychains` ran with no prompt or refusal (G07 log line 434). The directory did not exist in the test account.
   - The expected result "each route is refused or prompted" was therefore not demonstrated for this route. Say so.
4. **MOD-22: an off-script deviation that explains "no prompt was shown" is not disclosed.**
   - Before action 1, the G18b executor ran the full MOD-07 sequence under Moderate, including saving a "don't ask again" prefix rule (log lines 135–281, screens `mod07-*`).
   - That saved rule plausibly explains why there was no prompt after the relaunch. Disclose it, as MOD-03's skip is disclosed.

## 3. Replays: honest
- Both replays used fresh executors, and the earlier attempts are kept.
- The G21b note is navigation only.
- The G18b note is correctly described in the README as a post-results clarification, and the reason for choosing these two cases is stated.
- Optional: the MOD-22 row in `dispositions.md` could also say this (r1 asked for it there).

## 4. Isolation: honestly represented
- Every run has a `probes.json` with no failures.
- All 36 isolation receipts name their run, agent, case, design hash and candidate hash, and they match `results.json`.
- The guard recorded 21,518 checks and none flagged.
- The export redaction matches its manifest.
- The harness deviations r1 found are disclosed.
- Minor: `su.exp` itself is not in the record. The README's "holds no credential" rests on how `harness/admin.sh` calls it (the password goes in on stdin).

The named limits are complete apart from what corrections 1–4 imply.

## 5. Other things a reviewer should know
- The inspector shows the same "off or not observed: Credential broker" line in every Moderate session. MOD-05 passes on the row next to it, and MOD-24 relied on the line itself in the opposite sense. Any re-judgement should treat the two cases consistently.

VERDICT: fail