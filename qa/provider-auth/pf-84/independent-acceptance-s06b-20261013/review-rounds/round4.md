# Independent review of the PF-84-S06 slice B re-acceptance (2026-10-13)

**Verdict: AGREE WITH CHANGES**

I read only the allowed files: this folder, the S06 sprint, `s06b-gate.md`, `docs/provider-accounts.md`, the plan's PF-84 decisions, the 10-11 README and REVIEW, and the #555 `design.json` and `original-proposal.md`. I did not open `codex-rs/**`, root `scripts/**`, PR diffs or `.codex-work`. I modified no files. I ran a pattern scan of this folder (Z.AI `hex32.alnum16`, long `sk-…`, `sk-ant-`, `AKIA…`, long JWTs). It found no real-looking credential, only the documented fake canaries and the masked `sk-canar***y-S6B`.

I checked these claims against the captures and they hold:

- **#570:** `b-forced.txt`: CAND keeps `auth.json` in cases 1/2/5/6/7/8/9/10/10b/11. BASE deletes it in 1b/2b/5b/7b.
- **#571:** `tB-04:33`, `tB-05:45`. No `/responses` request in mock-t lacks credentials. The mock does return 401 to credential-less requests (`mock-b4` `responses-401-noauth`), so a regression would show.
- **#572:** `tB-02:20–25`, and `tB-03:28,36,56` with the decoys set (`tui-invocations.txt:16`).
- **#573:** `t1-setup.txt:8–11`, `tB-10-b`.
- **The k2 bearer in `tB-16:29,33`:** I recomputed SHA-256(`sk-canary-keyacct-S6B`) = `af0ebda0…`. It matches.
- **The z4 re-runs:** they discriminate as claimed. FK-01–05, FK-08, FK-09, RS-04 and PR-01b answered on `work`. FK-06 and PR-01a/c/d got Z.AI's 401.
- **GH-01:** `555r3-GH01-config.txt:13` shows `zai = "ghost"`.
- **C7 under Aggressive:** `tB-20-*` with "Aggressive active", plus the log lines.
- **C6:** `leak-scan-final.txt` covers this folder and `$S/logs`, with 0 hits for every real key and fragment.
- **Counts:** the 22-blocker count and the "19 pass" arithmetic are correct.

## Findings

### Blocking

None.

### Should-fix

1. **REG-05's explanation is contradicted by a capture, and "The rest pass" overstates the advisory cases.**
   - **REG-05** (`README.md:155`) says the `Account:` line is missing only "straight after a resume, before any turn (RS-05)".
     - `555r2-RS04-o1-picker-enter-turn-status.txt` resumes through the picker, runs a turn (`pickturn`), and its `/status` still has no `Account:` line.
     - `555-RS01-b` (resume by id, then a turn) does have the line, at line 33.
     - So the omission isn't explained by "no turn yet". The design accepts the missing line only if "there is an explanation for why it is missing", so REG-05 is unexplained. Record it as an open observation, not explained.
   - **"The rest pass"** (`README.md:328`). REG-04 is "EXPLAINED" and REG-05 is "OBSERVATION"; neither is a pass. List them separately.

2. **A re-run is missing from Re-runs.** `tui-invocations.txt:115–130` shows the whole `tB-20` Aggressive sequence twice for both CAND and BASE: `a1cand`…`a4base` at 115–122, then again at 123–130. That is a second `r3_suite.sh` pass. The kept `tB-20-*` captures and the accounting logs (`ts >= $M`) come only from the second pass.
   - The README says "every re-run and every discarded or overwritten attempt is listed" (`README.md:72–74`).
   - Add this re-run, give its reason, and say whether the first pass gave the same result.

3. **The V6-timeline bullet still contradicts the mock** (`README.md:198–200`). This is left over from round-3 finding 5.
   - It says "`plugins/featured` is sent with no credentials in named-account homes".
   - In mock-t, `plugins/featured` carried `tuidefault` at 01:30:24, during V6, in t2. t2 is a named-account home that also has a default login.
   - Lines 183–187 now make the correct distinction between homes with and without a default login. Line 200 contradicts them, so drop or qualify it.

4. ***Use for this session* starts a new conversation, and the README doesn't say so.**
   - In `tB-02:15–16`, `tB-03:18–20`, `tB-05:18–20`, `tB-13` and `tB-15:62`, the old session prints "To continue this session, run corbanu resume <id>" and then a **new session id** appears (`tB-03:43` `01a13448…` against `01a13447…`).
   - V2 (`tB-04`) switched with `/model` before its first turn.
   - So A10, B11 V2/V3/V6 and the D3 user switch never switched providers inside a conversation that already had turns. That is the path where history carries across, as #578 in the gate shows.
   - The rows aren't wrong, but "mid-session" overstates them. Add "a provider/account switch inside a conversation with prior turns" to Not exercised, and note the new-session behaviour in A10 and B11.

5. **The D3 user-switch evidence is thinner than the row implies** (`README.md:107`, `tB-13`). `tB-13` shows only the selection step under Aggressive:
   - it picks `openai · work` from a Z.AI session, which is a cross-provider stamp;
   - it then prints "New session: requests to openai use account `work`";
   - there is no `/model` switch and no turn on `work` afterwards.

   So "not prompted" was observed only when the account was chosen. It wasn't observed when the account was first used, or for a same-provider (Z.AI) switch. Say this in the row, and in Decision 2, which asks Travis to rule on exactly this behaviour.

6. **The PARTIAL and limited blocker legs have no product-authority disposition.**
   - ESC-03's draft leg (`README.md:147`) and REG-01's flag-off ESC-01 picker leg (`:151`) are blocker cases left PARTIAL.
   - GH-02's "ideally mentions ghost" (`:141`) and PR-01's missing `/status` in the z4 re-runs (`:149`) are also gaps.
   - AGENTS.md requires product authority to accept out-of-scope dispositions. Decisions for Travis (`:254–279`) names only PR-02, D3, REG-02 and the general limited-testing item.
   - Name ESC-03 and REG-01 there, at least as part of Decision 4's limited-testing agreement. Otherwise the executor's ambiguity note becomes an unrecorded waiver.

7. **The fpx sign-in refusal in C3 has no capture behind it** (`README.md:102`).
   - "`fpx` before sign-in (TUI, exit 1 before the UI)" appears in no capture. `tB-08` shows only the later successful run. The first `tui[v1]` launch (`tui-invocations.txt:33`) is its only trace.
   - C3 also cites no capture at all. Add the refusal capture or drop the fpx clause, and cite `b-cli.txt:149–155,191` and `real.txt` (oa-ghost) for the other legs.

### Nits

8. **Clean-up record** (`README.md:341–348`):
   - `cleanup.txt` is dated 09:55:28Z. It records only the state after deletion; neither the `rm` nor its exit status is captured.
   - Besides `logs/` and the review files, it also lists `build.sh`. The prose says "only `logs/` … and the review files".

9. **C1** (`README.md:100`): "flag-off `/providers` is identical". The two `tB-12` files differ in their `/status` cards (session ids, token totals). The `/providers` rows match. Say "provider rows identical".

10. **FK-01 row** (`README.md:121`): "earlier sessions that explicitly chose those accounts". The `default` request came from PR-01d, where `default` won by precedence, not by an explicit choice.

11. **Glitch scan** (`linux-reg06.txt`, "real key in any file"): this checked the full key only, with no fragment needles. C6's "Glitch: 0 files" should say so.

12. **REG-03 could cite stronger evidence.** The footer reads "via zai default" even on `personal` sessions (`555-MSG01-b`, `555-MSG04`, `555r2-FK06`). That shows directly that "default" in the footer is not the account. The design asked exactly this question.

### Disposition check of rounds 1–3

| Item | Status |
| --- | --- |
| R1 #1–#18 | Addressed, directly or through the R2/R3 dispositions |
| R2 #1–#18 | Addressed, as R3 confirmed. R2 #11 (raw attempts) has a new gap: finding 2 |
| R3 #1 (C6 timing) | Addressed: C6 restated, final scan in `leak-scan-final.txt`, unscanned homes named |
| R3 #2 (B4 "never read") | Addressed, cross-referenced to #658, listings-vs-installs reading stated |
| R3 #3 (C7 overstated) | Addressed |
| R3 #4 (traceability) | Addressed: `tui-invocations.txt` now 130 lines, mock-t refreshed (installed 103 = 69 + 34), Reproduce updated |
| R3 #5 (connector bullets) | Partly: lines 183–187 fixed, line 200 still contradicts (finding 3) |
| R3 #6 (D3 state, citation) | Addressed |
| R3 #7 (limits, debug builds) | Addressed |
| R3 #8 (universal claims) | Addressed |
| R3 #9–#13 (nits) | Addressed |
| R3 #14 (clean-up capture) | Partly (finding 8) |

## Closing

The captures support the report's core conclusions:

- the four defects that held slice B (#570, #571, #572, #573) are fixed on `53c02c9301`;
- per-account vault storage, refresh write-back, env-key isolation, forced-login scoping and fail-closed refusal hold on what was exercised;
- most #555 routing is now checked at the provider, with the limits stated.

The C7 Aggressive failure (#654) and the default-token side traffic (#658) are reported accurately and are not overstated. The recommendation to accept slice B behind the flag, under an explicit limited-testing agreement, is reasonable. Before this becomes the accepted record:

- correct REG-05 and the "rest pass" line;
- disclose the second `tB-20` pass;
- fix the V6 `plugins/featured` sentence;
- state that *Use for this session* starts a new conversation, so a switch inside a conversation with history was not exercised;
- put ESC-03 and REG-01's untested legs in front of Travis.

Any human-test handoff should carry these limits forward: instruction-only isolation, debug builds without companions, the not-exercised list, and the open #654, #658 and #595 items.
