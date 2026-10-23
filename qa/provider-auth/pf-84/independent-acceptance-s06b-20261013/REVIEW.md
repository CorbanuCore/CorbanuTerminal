# Independent review of the PF-84-S06 slice B re-acceptance (2026-10-13)

**Verdict: AGREE WITH CHANGES**

I read only the allowed files:

- this folder (`README.md`, `captures/`, `scripts/`, `review-rounds/round1–4.md`);
- the S06 sprint record and `s06b-gate.md`;
- `docs/provider-accounts.md`;
- the PF-84 decisions in `unified-provider-auth.md`;
- the 10-11 README and REVIEW;
- the #555 `design.json` and `original-proposal.md`.

I did not open `codex-rs/**`, root `scripts/**`, PR diffs or `.codex-work`, and I modified no files. I scanned this folder for real-looking credentials (Z.AI `hex32.alnum16`, long `sk-…`, `sk-ant-`, `AKIA…`, JWTs). It found only the documented fake canaries and the masked `sk-canar***y-S6B`.

## Round 4 dispositions

| R4 item | Status |
| --- | --- |
| #1 REG-05 explanation, "the rest pass" | Addressed. `README.md:155` is now an open, unexplained observation. I checked it against the captures. The line is present after a turn in `555-RS01-b:33`, `555-GH01:33` and `555r3-GH01:53`. It is missing in `555-RS05` and in `555r2-RS04` (the latter after the `pickturn` turn). `:338–345` now lists the advisory cases separately. |
| #2 second `tB-20` pass | Addressed (`:233–236`). The first pass ran under Permissive with no warning, and its captures were deleted. That is disclosed. |
| #3 V6 `plugins/featured` | Addressed (`:200`). mock-t has 2 `featured` requests with `tuidefault` and 35 with no credentials, which is consistent with `:184–186`. |
| #4 *Use for this session* starts a new conversation | Addressed in A10 (`:98`), B11 (`:97`) and Not exercised (`:256–258`). |
| #5 D3 user-switch evidence | Addressed in the D3 row (`:107`) and Decision 2 (`:272–273`). `tB-13:48–50` shows only the selection step. |
| #6 product-authority disposition of the partial legs | Addressed for ESC-03, REG-01, GH-02 and PR-01 (`:287–291`). Other legs are still missing (finding 2). |
| #7 C3 fpx | Addressed. The clause is now "not counted", and C3 cites `b-cli.txt:149–155` and `real.txt` oa-ghost (line 37). The Re-runs list doesn't record the overwrite (finding 4). |
| #8–#12 nits | Addressed: clean-up (`:364–366`), C1 "provider rows identical", FK-01 precedence wording, C6 glitch full-key-only, REG-03 evidence (`555-MSG04`, `555r2-FK06`). |

I re-checked these claims and they hold:

- **z4 discrimination:**
  - FK-01/02/03/04/05/08/09, RS-04 and PR-01b answered on `work`.
  - FK-06 and PR-01a/c got Z.AI's 401 naming `personal`, and PR-01d got the unnamed-credential 401.
  - The z4 launch lines show `ZAI_API_KEY=<fake default key>`.
  - The o1 runs used `(no ZAI key)` (`tui-invocations.txt:26–31`). That makes the RS-01 and RS-04 "D" verdicts valid.
- **Mock design:**
  - A credential-less `/responses` gets 401 (`mock.py:204–207`). Exactly 1 `responses-401-noauth` appears, in mock-b4 (b-forced case 3), and 0 in mock-t.
  - A reused refresh token is logged as `refresh-rejected why=reused` (`mock.py:189`). mock-b1 has 1 `refresh-rejected` (`bad`, expired), so B2's "0 reused" can be detected and holds.
- **#555 counts:** `design.json` has 33 cases, 22 of them blockers. "19 pass" is correct.
- **B4/B7:** `b-forced.txt` cases 1–11 against 1b/2b/5b/7b, and the `tB-16:28–33` k2 hash.
- **C2:** `carry-flagoff.txt`, S1 and S2 IDENTICAL.
- **C7:** all four `tB-20-*` captures show `Aggressive active` with the accounting warning, and both logs show `route mismatch`.
- **#658 counts:** `tB-19-…-traffic.txt:2–15`.
- **B3 refused add:** in mock-t, the next device sign-in after `devz` is `tuiwork~2`, so no second `devz` sign-in occurred.
- **PR-02 docs basis:** `provider-accounts.md:36–38`.

## Findings

### Blocking

None.

### Should-fix

1. **Analytics is described as a documented limit, but `tB-19` contradicts the gate's own limit** (`README.md:197`, `:201`).
   - The gate says "Analytics events keep the manager the session started with" (`s06b-gate.md:149`). The `tB-19` session **started** on named `work`, yet its two analytics posts carried `tuidefault` (`tB-19-…-traffic.txt:12`). That is a further gap of the #658 kind, not a known limit.
   - V6 started on the default, so V6 is consistent with the gate. Split the two cases, and add analytics to the #658 list for sessions that start on a named account.
   - `wham/usage` (2, `tuidefault`) is called documented. The documented limit covers only rate limits read "before the first reply" (`provider-accounts.md:175–176`), and the timing was not checked. Say "not checked whether before the first reply".

2. **The limited-testing list (Decision 4, `:284–291`) still omits legs that are counted as passes or held open elsewhere:**
   - **ESC-04** (blocker), "PASS (no BASE comparison)" (`:148`). The design's expected result is that onboarding appears "as it did before the change". With no before-build capture, that comparison is untested.
   - **ESC-01** (blocker): the design requires "the terminal is restored", and `:145` says this was not observed.
   - **D3's "refused with approvals off" leg** (plan D3 row, `unified-provider-auth.md:740`). It is listed under Not exercised (`:253–255`), but the D3 verdict cell (`:107`) says plain "PASS (as the plan defines D3)". Qualify the verdict as covering the approval leg only. Then add all three to Decision 4 so that product authority, not the executor, accepts them.

3. **REG-05: even when the line is present, it doesn't identify the active credential.**
   - On every Z.AI named session it reads the generic `Z.AI API key configured` (`555-RS01-b:33`, `555r3-GH01:53`, `tB-14-D3-a:32` on `personal`).
   - OpenAI names the account: `OpenAI API key (account fake)` (`tB-06:40`).
   - The design's expected result is that the line "shows the active credential". Note this in REG-05, or cross-reference it to #609.

### Nits

4. **Re-runs list** (`:216–238`): `:72–74` promises that every overwritten attempt is listed, but the C3 fpx refusal capture that was overwritten (`:102`) is not in it.
5. **REG-06** (`:156`): the list FK-05, FK-06, GH-01, GH-02/MSG-03, GH-03, ESC-01 covers 7 blocker cases, since MSG-03 is a separate blocker, not 6.
6. **REG-01** (`:151`): "fails the same way on both". The raw trace differs in its text (`Location: tui/src/app.rs:1037` on BASE, `:1042` on CAND). This is trivial, but the design asks for matching text. Say "differs only in the trace's line number".
7. **B11** (`:97`): V1, V4 and V5 cite no capture, although `tB-08`, `tB-06` and `tB-07` exist and support them. **D3** (`:107`): "After *Allow once*, the child ran on `account work`" links `tB-14-D3-a`, which shows only the prompt. The evidence is in `tB-14-D3-b:46–50`.
8. **`cleanup.txt` has been post-processed.**
   - The macOS `ls` errors show shortened paths (`ls: .codex-work/targets/s06b*`).
   - The glitch line shows an unexpanded `'~/corbanu-rtx/s06b*'`. Real GNU `ls` output would show `/home/…` after tilde expansion.
   - Either paths were rewritten or the remote glob was quoted. If it was quoted, the check could not detect leftovers. That matters because the glitch vault and its fallback key held the real Z.AI key.
   - Say the paths were abbreviated, or re-run the remote check with an absolute path.
9. **Method contradiction** (`:34` against `:27`): "I read no … reviews", yet the executor read the 10-11 REVIEW and responded to review rounds 1–4. Say "no code reviews or worker reviews".
10. **Markdown and stray text:**
    - `:186–187` lost the list indentation under its bullet.
    - `:171`: the sentence "With this run, the 33 #555 cases have now been run…" sits under #588, where it doesn't belong.
    - `:13` links `REVIEW.md`, which exists only once this review is saved.

## Closing

All round 4 findings are addressed and I could verify them against the captures. The captures support the core conclusions:

- #570, #571, #572 and #573 are fixed on `53c02c9301`;
- per-account storage, refresh write-back, env-key isolation, forced-login scoping and fail-closed refusal hold on what was exercised;
- most #555 routing is now checked at the provider, with the limits stated honestly;
- C7 under Aggressive (#654) and the default-token side traffic (#658) are reported without overstatement.

The recommendation to accept slice B behind the flag, under an explicit limited-testing agreement, is reasonable once three changes are made: the analytics statement is corrected, Decision 4 also names ESC-04, ESC-01's terminal restoration and D3's approvals-off leg, and the D3 verdict is qualified.

Any human-test handoff must carry these limits forward: instruction-only isolation with real-vault access, debug builds without companion binaries, B/C/D criteria derived by the executor rather than a frozen design, the unscanned round-2 homes, and the open #654, #658 and #595 items.
