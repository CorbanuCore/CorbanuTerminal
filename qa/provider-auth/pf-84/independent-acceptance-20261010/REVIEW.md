<!-- Reviewer: installed corbanu exec, -m claude-opus-5-5-plan, model_provider=claude-plan, reasoning high, -s read-only; prompt in the PR description. Output saved verbatim below; executor dispositions follow. -->

# Review

**Verdict: ACCEPT WITH CHANGES.** The functional evidence is strong and mostly traceable. Before this run can close the sprints' code-blind gate, some verdicts need relabelling, two unreported defects need to be added, and some traceability gaps need closing.

## Findings

1. **High · Method · "code-blind" is overstated.**
   - The README says the run was "code-blind", but the executor read material that `qa/code-blind-functional/README.md` excludes. The sprint code boundaries include file and line references. The gate files contain test results, review dispositions and implementation rationale, such as the `spawn_orchestration.rs` preflight.
   - The executor also built the candidate from source in the repo worktree. Its isolation came from its own restraint, with no recorded negative access probes. There was no separate designer, no frozen `design.json`/`results.json`, and no handoff-checker run. It tested a debug cargo build, not the packaged candidate.
   - **Change:** call this "independent source-blind functional acceptance". Either complete the gate's missing parts or record a limited-testing agreement from product authority. Do not tick the sprints' "code-blind handoff checker" box on the strength of this run.

2. **Medium · Method · real-key retrieval used the live operator vault.**
   - `realenv.sh` and `claude-path-skew.sh` call `/Users/Neo/.local/bin/corbanu.bak-20261010 vault auth-helper` with `-u CORBANU_TEST_NO_NATIVE_KEYRING`. That path reads the native keychain vault key.
   - So "no keychain" and "security shim 0 calls" are true only for the candidate and baseline processes.
   - **Change:** state this in Safety, and record the owner authorisation for real-credential access.

3. **Medium · S02-13, S02-11, S02-6 · the TUI captures cannot be traced to a selector.**
   - The launch arguments and config for t1–t4, t0, vault-list and aggressive are not recorded. `tui-run.sh` takes extra arguments, but its invocations are not saved.
   - Every status line reads "zai default", even in t2 (`fake`) and t3 (`ghost`). Either the selected account was not used, or the indicator misreports it.
   - The config-table form `[provider_accounts]` is never shown working. Only `-c` appears in the captures, and t0, which used the config form, failed.
   - **Change:** save the invocations plus `config.toml` and `account list` for each TUI home. Record the "default" indicator as a finding, probably for S03 or S04.

4. **Medium · S02-16 · over-claimed, and two problems are missing.**
   - The capture shows that a named `claude_config_dir` label is released by `vault auth-helper` (65 bytes, exit 0). It is a subscription-account label, and the README leaves it out.
   - The refused Claude token exits **0** with empty stdout. A `X="$(…)" cmd` caller would then carry on with an empty credential.
   - **Change:** mark the row PARTIAL and file both, or get a product disposition.

5. **Medium · S01-4 / S01-1 · `corbanu-debug` is under-graded.**
   - In V2, `CORBANU_HOME=homeB` is silently ignored: debug home, 0 warnings. That is the same "silently ignored" behaviour S01 exists to remove.
   - W5 and W7 were never run against the candidate binary.
   - **Change:** split the verdict into PASS for `corbanu` and FAIL (low-medium) for `corbanu-debug`, add the missing runs, and consider raising #418 to medium.

6. **Medium · S02-3b / #414 · severity and scope.**
   - The FAIL is justified. But `corbanu-debug` on a newer build alongside an older `corbanu` on PATH is the default installer layout, so this version skew is common, not exotic.
   - Only the env-token path was tested. A pre-PF-84 helper with no env token, falling back to the default vault token, was not.
   - **Change:** add that case and say in #414 that it blocks removing the flag.

7. **Medium · Recommendation · "S02: accept" does not follow from the verdicts.**
   - S02 has three FAIL rows (3b, 9b, 14), and S02-15 is not verifiable.
   - Under AGENTS.md, a functional failure blocks an unqualified handoff, and out-of-scope dispositions need product-authority acceptance.
   - **Change:** reword to "limited acceptance behind the flag, conditional on Travis accepting #414, #415, #416 and the open S02-15", and mark S02-15 BLOCKED (needs product input on sibling routes), not final.

8. **Low · S02-7 · over-claimed.**
   - The capture shows Aggressive "enforced, but the protected boundary is unverified", and the tester approved the command.
   - A directly started CLI under saved Aggressive adds and removes accounts with exit 0.
   - **Change:** describe the verdict as covering the agent-command path only, in an unverified preflight state. Get a product ruling on whether the sprint's "refused under Aggressive" also covers self-started CLI runs.

9. **Low · #419 items not reported.** P3 (`ghost` account) invoked the Claude token command **28 times** before failing. Add this to #419.

10. **Low · S01-5 · no comparison value.** The capture never records homeB's expected fake-key suffix (`6bb0`). Add one line with that suffix.

11. **Low · S02-10 · wording.** After M9 the vault size is back to 617 bytes, but its hash changed (`ea89…` vs `70dd…`). Say "same size, re-encrypted", not "restores".

12. **Low · #417 · the evidence is thin.** The t0 capture shows only the onboarding screen. Add the home's config and account list.

13. **Low · Reproducibility and hygiene.**
    - These are not in the evidence directory: `shim/security`, the `pathbin` and `pathbin-pre` shims, the `activate.sh` stand-in, the mock provider config (`mockp`, `cmdp`), how `canaries.json` was generated, and how baseline home G was onboarded.
    - The scripts hard-code `/Users/Neo/…`.
    - The synthetic `sk-ant-…cfgdir-canary-0001` (elided here) may trip push-protection scanners; rename it.
    - Scratch deletion is claimed but not evidenced.

## Secrets

No real secret was found in the captures or scripts.
- The only partial key shown is the masked fake OpenAI suffix (`…6bb0`).
- The 12-hex fingerprints (sha12) shown are of synthetic tokens.
- The repo-directory leak scan is clean for all 4 real keys.
- Limitation: the scan matches whole values only, so a partial fragment of a key would not be caught.

## Rows confirmed

- **S01:** S01-1 (for `corbanu`), S01-2, S01-3, S01-5 (except finding 10), S01-6 and S01-7. The doc text matches lines 68–77.
- **S02:** S02-1, S02-2 (the keychain part is correctly left unverified), S02-3a, S02-4, S02-5, S02-8, S02-9a, S02-9b (FAIL is justified), S02-10, S02-12 and S02-14 (FAIL is justified).
- **Issues:** #415 (low) and #416 match the evidence. #419 items 1, 2 and 4 are visible in the captures.
---

## Executor dispositions (added after the review; the README was revised accordingly)

1. Renamed to "source-blind". The README says this is not the milestone code-blind VM gate and does not tick the handoff-checker box. Reading the sprint records and gate files was the owner's instruction.
2. Safety now says the helper reads the operator vault through the installed signed binary, as `sec-common.md` allows.
3. Added [tui-setup](captures/tui-setup-homes-and-invocations.txt) with per-home config, account list and invocations. The "zai default" word is the reasoning effort ([evidence](captures/tui-status-line-effort-not-account.txt)), not a finding. The config-table selector is now proven ([table](captures/s02-config-table-selector.txt)). t0 used `-c`, not the table.
4. The exit 0 came from a bug in my capture (it read `PIPESTATUS` in a subshell). On rerun the named Claude token exits **1**, and the capture is replaced. The config-dir label returns a directory path, which is not a secret; noted in S02-16.
5. Split into S01-4a PASS and S01-4b FAIL (low). V4–V6 run against the candidate, and #418 commented. I kept low severity: caller-set `CODEX_HOME`/`CORBANU_DEBUG_HOME` are kept, and only `CORBANU_HOME` is ignored.
6. Commented on #414: blocks flag removal; the vault-default fallback path was not tested (keychain safety).
7. Recommendation reworded to limited acceptance behind the flag, conditional on Travis. S02-15 stays NOT VERIFIABLE, because PASS/FAIL/NOT VERIFIABLE is the required verdict set.
8. S02-7 is scoped to the agent path, in the "unverified boundary" state, and asks for a product call.
9. #419 already lists the 28 spawns (item 3).
10. Added homeB's synthetic suffix `6bb0` to the S01-5 capture.
11. M9 wording: "re-encrypted (new hash)".
12. Covered by item 3.
13. Added fixtures (activate.sh stand-in, shims, mock provider configs, canary generator). Split the synthetic `sk-ant-…` literal. Scratch was deleted after the run.
