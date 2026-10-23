# Owner decisions this amendment must follow

Verbatim from the owner's decision log (Travis), unless marked "integrator". Issue numbers are the product's issue
tracker; you do not need them beyond telling the decisions apart.

## A. Moderate run 1, all 13 decisions "as recommended" (2026-10-15, 22:06Z)

(1) #838: deny Corbanu's security-state and key files to agent commands under Moderate even after approval.
(2) #837: approved commands may write .git, while hooks and git config stay protected. #875 found further routes, so this is D5.
(3) #840: fix it so saved rules keep working (D6 kept).
(4) #841: keep the .env prompt, fix the reason shown for Corbanu's files, and ask before edit-then-run only after untrusted content.
(5) #839: fix both; it blocks shipping.
(6)/(7)/(10) Merge #850 (#845), #849 (#847) and #854 (#843).
(8) #842: other windows on the same home switch to Moderate too.
(9) #846: correct the wording.
(11) MOD-28: revoke does not suspend saved rules; say so and re-run with a session approval.
(12) Re-run MOD-01/22 on the next build.
(13) MOD-09/21: accept as blocked and re-run once with a different test model.
Next Moderate run: re-run only the affected cases on a fresh build after the fixes.

The recommendations these answer (from the run-1 report to the owner, verbatim):
1. #838 (security): I recommend denying those Corbanu files [security_state.json and source-origin.key] to agent commands under Moderate even after approval, as config, history and the vault already are.
2. #837: I recommend letting an approved command write to `.git` while still protecting hooks and git config from silent edits.
3. #840 ["Don't ask again" rules saved under Permissive stop working under Moderate]: I recommend fixing it so saved rules keep working, as your decision D6 says.
4. #841 (extra prompts): I recommend keeping the `.env` prompt, fixing the wrong reason shown for Corbanu's files, and asking before edit-then-run only after untrusted content.
5. #839 (inspector) [Recent denials misses refusals; the model-keys line contradicts the top line]: I recommend fixing both the missing refusals and the contradictory model-keys line.
6. #845 (startup) [a project `.codex/config.toml` with a `[security]` table stops Corbanu from starting]: I recommend merging fix #850.
7. #847 (agent file tools fail) [under Moderate the agent's own file tools failed with "macOS is blocking access to the Corbanu home"]: I recommend merging fix #849.
8. #842 (second window): I recommend switching other windows on the same home to Moderate too, as the kill switch does.
9. #846 (advisory): after Aggressive is confirmed, `.env` stays readable until the restart, but the Saved screen says it's protected now. I recommend correcting the wording.
10. #843 (advisory, spurious warning) ["could not create PATH aliases: Operation not permitted"]: I recommend merging fix #854.
11. MOD-28 (blocked): should "Revoke all active authority" also suspend saved "don't ask again" rules? I recommend no: say so on the revoke screen, then re-run the case with a session approval.
12. MOD-01/22 (#739, already fixed by #813): I recommend re-running both on the next build.
13. MOD-09/21 (blocked because the model declined, per the #447 rule): I recommend accepting them as blocked and re-running once with a different model.

## B. #739, Permissive under protected launch (2026-10-14, 22:58Z), verbatim

Owner decision (Travis, 2026-10-14, all as recommended): option B (SECURITY SIGN-OFF): with the security features on,
Permissive stops offering approvals that protected launch will refuse (writes outside the project / runs outside the
sandbox); the prompt and the /security review text say so.

(Context: the test machine's profile has the security features on, including protected launch,
`secretless_agent_launch`. MOD-01 and MOD-22 were written before this decision and expect an approved run outside the
sandbox to succeed under Permissive. The same decision was applied to the Aggressive milestone's cases on 2026-10-15
as: "no offer, plain refusal or sandbox denial, file unchanged, note for a remembered rule; test the #626 step
separately with a rule saved without protected launch".)

## C. #822, refusal wording (issue text; fixed by #851, refined by #900)

"If Core refuses a `shell_command` before asking anyone, the TUI's command cell still says '✗ Did not run … Approval
declined', although nobody was asked to approve it. [...] The cell should still say 'Refused' or show the refusal text
instead of 'Approval declined'." The coordinator's brief for this re-run includes "#851's refusal wording" among the
fixes to re-check. (Integrator: whether #900's refinement is in the candidate depends on the build; judge only that a
refusal nobody was asked about is not labelled "Approval declined".)

## D. Coordinator's brief for this re-run (2026-10-16), verbatim parts

- "re-run MOD-01/22 (fixed by #739/#813); re-run MOD-09/21 once with a different test model than the one that declined
  (still GLM-family or another non-Claude model available via a disposable profile — record which); MOD-28 with a
  session approval; and re-run every Moderate case whose defect has since been fixed: #838 (#869), #846 (#870),
  #840+MOD-28 wording (#877), #839 (#881), #842 (#883), #841 (#892; MOD-06/26/30), #845 (#850), #847 (#849), #843 (#854),
  plus #851's refusal wording (#851/#900)."
- "Do NOT re-run the .git cases tied to #837 (MOD-06's git-commit part / MOD-11) — #837 awaits Travis's decision (D5);
  record them as 'pending owner decision', not failed."
- "a fresh code-blind designer amends the frozen Moderate cases only where the owner decisions changed expected
  behaviour (from case text + decisions only)."

## E. Earlier decisions still in force (unchanged; listed for completeness)

- D4 reading (2026-10-14): "D4 reading kept (confirmation raises Core's whole level at once; broker and 'active' label
  wait for the restart)." The first run's rule G-2 said this was still open; it is now decided as that reading.
- #447 model-decline rule (2026-10-14): "a case passes when the product's model declines a step but its text still
  meets the expected wording; blocked otherwise." (G-1.)
- #766 (nested `corbanu exec`) was a named limit of the first run (G-3); it is not among this re-run's decisions.

## F. Integrator choices (not owner decisions)

- Different test model for MOD-09 and MOD-21: the product's model GLM 5.2 on Z.AI (`agms-tui start --model glm-5.2`),
  the only other model the test machine provides. The first run used GLM 5.3 Flash. The tester (executor) model stays
  GLM 5.3 Flash.
- The candidate is the signed integration build of 2026-10-16 (commit 4ef3be8467 or later); its screens are in
  `screens/`.
