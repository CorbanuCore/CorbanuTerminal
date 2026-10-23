# Independent review of the PF-84-S06 slice B re-acceptance (2026-10-13)

**Verdict: AGREE WITH CHANGES**

I read only the allowed files. I did not open `codex-rs/**`, root `scripts/**`, PR diffs or anything under `.codex-work`. I did not modify any file.

Most verdict rows match their captures. These are confirmed:

- **#570:** b-forced cases 1b, 2b, 5b and 7b against CAND.
- **#571:** the mock now refuses requests that carry no credentials, and mock-t has none.
- **#572:** `tB-02` and `tB-03`.
- **#573:** `t1-setup.txt` and `tB-10-b`.
- **The z4 re-runs:** they do discriminate for FK-01–05, FK-08, FK-09 and PR-01b.
- **The leak scan:** it ran after the last capture.

Two problems make the report unsafe to rely on as written. The recommendation text is wrong, and the GH-01 "(D)" verdict rests on an unscripted re-run whose configuration was never recorded. Several citations also overstate what the captures show.

## Findings

### Blocking

1. **The recommendation miscounts, mislabels and leaves out a required limit** (`README.md:151, 234–256`).
   - **Wrong count.** `design.json` has **22** blocker cases, not 24: FK‑01–07, RS‑01–04, GH‑01–03, MSG‑01, MSG‑03, ESC‑01, ESC‑03, ESC‑04, PR‑01, REG‑01 and REG‑02. The README's own Pri. column also adds up to 22. Fix "24" at lines 151 and 243.
   - **The exceptions list is wrong.**
     - "The routing claims are proven at the provider (D)…" (line 247) is listed as an exception, but it is not one. It is also overstated (see findings 2 and 3).
     - REG‑02 is a blocker that needs a disposition, but the list leaves it out (finding 8).
   - **The isolation limit is missing.** Line 236 says "Accept S06 slice B behind the flag" with no conditions. The Method section admits the isolation was by instruction only (lines 17–24). Policy (AGENTS.md, September 12 amendment) says missing enforcement "blocks applicable functional handoff".
     - The S06 sprint still lists "Code-blind functional design frozen…" under Remaining (sprint `:68`).
     - The B1–B11/C/D criteria here were written by the executor from the gate. They are not a frozen independent design. Only the #555 cases come from one.
   - **Fix:** state that acceptance is conditional on Travis explicitly agreeing to limited testing. State that this run does not close the sprint's code-blind item.

2. **GH‑01 "PASS (D)" is not supported.**
   - The kept capture `555r2-GH01.txt` comes from a **manual** re-run. It resumed `01a1347e-b31e…`, which is FK‑05's fork (`tui-invocations.txt:112`, timestamped 02:13).
   - It is not the scripted step. `fk_suite2.sh` resumes `$SRC`, which produced the discarded capture.
   - After step h, the script restores `z4-config.bak`, which sets `zai = "personal"`. Nothing records that `zai = "ghost"` was set again before the manual run.
   - Under `personal`, resuming a conversation recorded on `work` gives the same screen (compare FK‑02). So the capture cannot tell GH‑01 apart from a plain resume.
   - **Fix:** record the config used for that run, or downgrade GH‑01 to PASS (A), backed by the z2 first pass.
   - Also, "Reproduce → `fk_suite2.sh`" (line 274) reproduces the discarded result, not the kept one.

### Should-fix

3. **The "D" label overstates FK‑06 and PR‑01a/c/d.** In z4, `personal` **and** `default` both hold fake keys (`fk_suite2.sh:3`, `drv.sh` `ft`).
   - A 401 therefore proves only "not `work`". Whether the request went to `personal` or `default` still rests on Corbanu's own message (`555r2-PR01a…`, `PR01c…`, `PR01d…`, `FK06…:28`).
   - The PR‑01 re-runs also have no `/status`: steps e, f and g in `fk_suite2.sh` never call it, although the design asks for it.
   - **Fix:** label these "D (rules out `work` only)".

4. **C7 (`README.md:104`) has unsupported or overstated parts.**
   - **Unsourced quote.** "accounting route mismatch" appears in no capture or script (`rg mismatch` finds nothing outside the README). Add the log excerpt.
   - **Degraded security state.** `tB-18`, `tB-18-c` and the D3 *Cancel* run (`tB-14-D3-c:21–22`) all ran in the state "Stored security level is unreadable … `security_level.toml` is missing … Aggressive enforced" (`tB-18:43`).
     - Only `tB-14-D3-b` is a clean "Aggressive active" run.
     - So "that holds on the default account too" is shown only in the degraded state.
     - Disclose how that state arose (it looks like a fixture side effect), and re-run or qualify.
   - **Wrong citation.** `tB-11` is cited for `OpenAI · account work`, but its `/cost` view (lines 81–88) shows no such row. That view is a scrolled pop-up: its header says 15 other requests, and only 7 are visible. Cite `tB-04:64` instead.
   - **Unsupported scope claim.** "Not S06-specific" has no BASE comparison. Say "not specific to named accounts".
   - **Unlisted failed attempt.** `tB-18-b` failed on the fixture: `work` was not configured in z3 after FK‑07 removed it. It is not listed under Re-runs.

5. **The B1 TUI claim is not shown** (`README.md:85`). `tB-10-h` is only an `account list`; it says nothing about `auth.json`. The file listing in `t1-setup.txt` was taken before the TUI adds. Add a listing taken after the adds, or drop the claim.

6. **Connector-traffic numbers and the "no issue filed" call** (`README.md:176–184`).
   - **Wrong count.** mock-t holds **35** `plugins/featured` requests with no credentials, plus one sent with `tuidefault`. The README says 24.
   - **Undispositioned default-token traffic.** mock-t also shows these requests carrying the **default** `tuidefault` token:
     - `ps/plugins/installed` (69);
     - `ps/plugins/list` (10);
     - `wham/settings/user` (2). One of these is at t+294.4, the moment V6 switched to `work2`.
   - None of these are in the gate's known limits (analytics, `/apps`, early `/status` limits). The docs say plugin installs follow the active account (`docs/provider-accounts.md:169–172`). Either file an issue, or give a disposition with timing that shows these happened before the switch.

7. **B4 and "Other observations" claim BASE controls that don't exist** (`README.md:88, 172–175`).
   - `b-forced.txt` has no case 3b. For a default `exec` session, "on both CAND and BASE" is backed only by the TUI blocks in `tB-16`.
   - "BASE and pre-S06 did the same" for the request sent without credentials (`mock-b4.jsonl:34`) has no BASE capture. Cite the 10‑11 run for the pre-S06 claim, or remove it.
   - Round 1, finding 14 raised this and it is still open.

8. **The REG‑02 label, and a blind spot in the C6 scan.**
   - REG‑02 is a blocker case, and "any part of a key" is not met: `b-forced.txt:136` shows `sk-canar***y-S6B`. Label it "NOT MET (inherited); disposition requested", not PASS.
   - `leakscan.py` searches 12-character fragments at offsets 8, middle and end. That mask form (an 8-character prefix plus a 5-character suffix) would not be detected for a **real** key.
   - **Fix:** state that `login status` never ran in a home holding a real key, or add prefix and suffix needles to the scan.

9. **D3 coverage.** The plan's D3 row also requires "refused with approvals off" (`unified-provider-auth.md:740`). That leg was not exercised and is missing from Not exercised (lines 202–215). The *Cancel* leg ran in the degraded state described in finding 4.

10. **Clean-up is promised, not recorded** (`README.md:258–262`).
    - "Are deleted after the PR" leaves real OpenAI, Z.AI, Kimi and Claude Plan credentials on disk until then, in r1, z1–z4, o1, n1 and t1.
    - Each vault's fallback key sits beside it (`b-cli.txt:67–70`), so those vaults are effectively unprotected.
    - The evidence does not need the homes. Delete them now and record the date and command result.
    - The glitch deletion is also asserted without any record.

11. **Raw attempts.**
    - Lines 71–72 ("named there rather than kept") should say plainly that this departs from the AGENTS.md rule to preserve raw attempts.
    - The statement is also inconsistent: `555r2-GH01-x-discarded…` is in fact kept.
    - The Re-runs list omits GH01‑x and `tB-18-b`.
    - The first-pass `555-ESC03-a` was overwritten.

12. **PR‑02 recommendation (a) needs its source.** `docs/provider-accounts.md:36–38` says an account named with `--account` "(of any provider)" is checked before the session starts. So the refusal is documented behaviour, and the mismatch is between the frozen design and the docs. Cite it in Decision 1.

### Nits

13. **A10:** "six `… via amazon-bedrock` models" is wrong. `tB-02:20–25` shows five Bedrock models plus `GLM 5.2 via zai-anthropic`.
14. **C2:** it needs the same caveat as C1. Home `hm` was made by BASE `82b1b2cf32`, which already contains S06 (`carry-flagoff.txt:1`). Pre-S06 carry-over is the 10‑11 C2.
15. **B1 home listing:** it also contains `secrets/.vault.lock` and a `tmp` lock (`b-cli.txt:67–71`). Harmless.
16. **"Before removing the flag" (lines 255–256):** add the sprint's Remaining item (early `/status` rate limits and `/apps` come from the default login) and live Bedrock.
17. **`tB-15-V6-setup.txt:1`:** "ChatGPT login is disabled … exit 1" is not explained.
18. **Formatting:** the table cells have a doubled space in "First pass (A):  " (lines 119–144). `555-MSG01-b…:53` has a stray-keystroke composer ("Runs/review"), carried over from round 1.

### Round-1 disposition check

| Round-1 finding | Status |
| --- | --- |
| 1 | Addressed. |
| 2 | Addressed by z4, except findings 2 and 3 above. |
| 3 | Disclosed, but not carried into the recommendation (finding 1). |
| 4 | Addressed (ESC‑03 PARTIAL). |
| 5 | Partly addressed (finding 11). |
| 6 | Addressed (RS‑04 re-run with a turn). |
| 7 | Addressed. |
| 8 | Addressed (#656). |
| 9 | Addressed (SHA not recorded, disclosed). |
| 10 | Addressed except the REG‑02 label (finding 8). |
| 11 | Addressed, apart from findings 4 and 9. |
| 12 | Addressed as #654, apart from finding 4. |
| 13 | Addressed. |
| 14 | Partly addressed (finding 7). |
| 15 | Addressed except the D3 approvals-off leg (finding 9). |
| 16 | Addressed. The scan ran at 02:22, after the last capture at 02:14, with `~2` needles and fragments. |
| 17 | Not done (finding 10). |
| 18 | Mostly addressed. |

The captures support the core claims: the four targeted defects are fixed on `53c02c9301`, slice B's per-account storage, refresh, isolation and fail-closed behaviour all hold, and most #555 routing is now checked at the provider. Before this README is saved as the accepted record:

- correct the recommendation (22 blockers, the conditional acceptance, the code-blind gap);
- downgrade GH‑01 or record its config;
- qualify the "D" labels and the C7 claims;
- fix the miscited and miscounted evidence;
- delete the real-key homes now and record it.

Any human-test handoff should carry the instruction-only isolation limit forward.
