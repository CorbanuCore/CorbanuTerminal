**VERDICT: fail.** The evidence supports every disposition (MOD-15 passed, MOD-28 accepted under D40, and the counts), but some written claims are wrong and must be corrected before sign-off.

## 1. Every case accounted for, and the freeze
- **Case count:** `design.json` has 34 cases. `results.json` has 34, each with exactly one disposition. The cases in `design.json` are identical to both earlier records' `design.json`. Q01 (MOD-15) and Q02 (MOD-28) have runs. All evidence, packet, access, actions and isolation hashes in `results.json` match their files.
- **Freeze:** every packet, harness, candidate and design file matches `design/FROZEN.sha256`, which is headed "frozen 03:43:23Z". The VM start (03:43:47Z, `common/vm-start.txt`) and the first run (03:46:58Z, `runs/seq.log`) come after it. The lock time (03:43:42Z) and the push of `c28aa47cd3` are only claimed, in `README.md` and `common/effective-policy.txt`; no raw artifact in the record shows them.
- **Freeze defect:** `FROZEN.sha256` lists its own hash (`6cbf…`). A file cannot contain its own hash, so `shasum -c` reports **FAILED** for that line.
- **Amendment v4:** `amendments-v4-final.md` is byte-identical to the fenced block in `design/review-output.md`. The packet's `amendments-v4.md` is the same file (`b378112f…`). The integrator's edits (`integrator-edits.diff`) are mechanical. The designer packet holds v2 and v3, and they match the earlier record.

## 2. MOD-15 and MOD-28
**MOD-15 (passed) is supported:**
- **Refusal text:** both cells show `✗ cat ~/.ssh/config` and `✗ corbanu vault list` with "Refused: Not run: Security level Moderate … turn approval prompts on (/permissions, or approval_policy in config.toml)". This is in `mod15-item2-end.txt` and `mod15-item3-captured.txt`. `pty.raw.gz` has "Refused" and "Security level Moderate" twice each.
- **No prompt:** `pty.raw.gz` contains no "Yes, proceed", "Allow", "Approval declined" or ⚠. The inspector shows "Approvals: never".
- **Product log:** `refused_approvals_off` for kind=Credentials at 03:54:05Z and kind=Vault at 03:54:45Z.
- **Item 4** ran normally. **Recent denials** lists both entries.

**MOD-15: a wrong claim that must be corrected.** The record says item 1 was "read with the agent's file reader … not `cat`". The product log contradicts this: `ToolCall: exec_command {"cmd":"cat README.md"}`. The screen just shows that command summarised as "• Explored └ Read README.md". The wrong statement appears in:
- `README.md` (named limits);
- the MOD-15 summary in `results.json`;
- the MOD-15 row in `dispositions.md`;
- the MOD-15 summary in `tools/final-t2.json`.

It does not change the outcome, but it is false and the "limit" does not exist.

**MOD-28 ("accepted under D40", not passed) is the right label:**
- **P1:** `read_shared_note` ran with no prompt (screens 24 and 25).
- **P2:** `post_note` offered only "1. Allow / 2. Cancel" (screen 30); Esc cancelled it.
- **P3:** `g` showed "No grants are held now." (screen 36). So no session option was offered.
- **Revoke review** (screen 41) matches `screens-v4/06` word for word, including the A(11) sentence.
- **After the revoke:** confirmation on screen 42; `g` showed no grants (screen 50); level still Moderate (screen 52). The log shows `revocation_generation=2 level=moderate` and no later transition.
- **Action 2** was correctly not run.
- **The executor's misdirected first P1 text** went into the open inspector. The log shows no request before 04:29:04Z, so it caused nothing.

**MOD-28: should be noted (not required).** A second detour is not mentioned in the record: after the revoke, `/security` was typed into the open picker, and its `i` opened the inspector. Nothing changed state, but the record should say so.

## 3. Owner decisions and carried cases
- **Owner decisions:** D39 (MOD-09, MOD-21), D43 (MOD-11, and MOD-06's git part) and D41 (MOD-33) match `decisions.md` and `owner-dispositions.json` (hash verified). The previous summaries are kept as a suffix.
- **Carried cases:** all other cases are identical to `../moderate-targeted-20261016/results.json` (hash `ccf22…` verified) except the added `carried_from` field.

## 4. Isolation
- **Receipts:** both are bound to the correct agent, run, case, design hash (`93e2567…` = `design.json`) and candidate. The policy, inventory, probe and mediator files they cite all match by hash.
- **Probes:** 45 + 40 + 45 per run, 0 failed, and each run's context points to its own run folder. The gate passed (85 probes, `AGMS-SMOKE-OK`).
- **Guard:** 0 hits in 3,350 checks.
- **VM handling:**
  - Data was erased, then checked live.
  - The clean shutdown happened at 04:44:16Z.
  - The verification boot did not answer the shutdown request and was force-stopped. `effective-policy.txt` says so; the README's "powered off" is accurate but less explicit.
  - The lock was released at 04:47:06Z.
- **Minor:** `effective-policy.txt` calls the session errors at 04:11:02Z and 04:26:19Z "idle relay connections". In the mediator log each one opened 1 s after an executor request ended and timed out after 600 s, and the executor's next request came seconds later. They look like executor requests that hung, which explains Q02's 10-minute gap (04:16 to 04:26).

## 5. Totals, sign-off and issues
- **Totals:** 30 passed and 4 accepted (blockers 23 + 3, advisory 7 + 1). These match `results.json` and `dispositions.md`.
- **Sign-off:** "ready for sign-off (D42)" follows, because MOD-15's pass is supported.
- **New issues:** #1035 matches screens 41 and 42. #1036 matches `mod15-inspector-recent-denials.txt`; the same stale line also appeared in Q02 (screen 19).

## Required corrections
1. Remove or correct the "item 1 read with the file reader, not `cat`" claim in the four places listed above. Item 1 ran `cat README.md` as a shell command (product log) and was displayed as "Read README.md".
2. `design/FROZEN.sha256`: remove the self-referencing `design/FROZEN.sha256` line, or explain it, so the manifest verifies cleanly.
3. `README.md` says "10 new screens"; `screens-v4/` holds 11 (00–09 plus 09b). The unused raw capture `10-outside-edit-prompt.txt` is not mentioned anywhere.

**Recommended:** describe the 04:11 and 04:26 session errors as stalled executor requests, and add the post-revoke detour to MOD-28's navigation note.

VERDICT: fail