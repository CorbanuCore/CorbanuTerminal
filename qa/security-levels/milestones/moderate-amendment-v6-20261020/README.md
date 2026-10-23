# Moderate case text: amendment v6 (MOD-22 inspector step, owner decision D96)

A case-text amendment for future runs of MOD-22 of the Moderate milestone. No run is recorded here.

- **Decision (owner-approved, 2026-10-20):** D96, "MOD-22 case text: add `/security`, `i` (inspector) after the
  relaunch step." Recommended by the Tart parity run ([record](../moderate-tart-parity-20261020/README.md), MOD-22:
  blocked because the case text has no inspector step after the relaunch, so "the person's own approval and sandbox
  settings match the baseline" could not be judged; two testers read the gap differently).
- **Amendment:** [design/amendments-v6-final.md](design/amendments-v6-final.md), given to testers as
  `amendments-v6.md` on top of the frozen cases and amendments v2-v5 (precedence v6 over v5 over v4 over v3 over v2).
  It replaces MOD-22's action 4 (`/security`) with `/security`, a picker screen, `i`, the inspector's approval,
  sandbox and network rows, and says how they are compared with amendments-v2's 'baseline' screen. No expected result
  and no rule changes.
- **Sign-off path (as amendments v4 and v5):** a fresh read-only code-blind designer (Opus 5.5 High) wrote the draft
  from the frozen cases, amendments v2-v5, the decision, the scope and the test-machine description
  ([design/designer-prompt.md](design/designer-prompt.md), [design/designer-output.md](design/designer-output.md),
  draft [design/amendments-v6-draft.md](design/amendments-v6-draft.md); the new packet files are in
  [design/designer-packet/](design/designer-packet/), the copied inputs are listed by SHA-256 in
  [design/designer-packet.sha256](design/designer-packet.sha256); the frozen cases and amendments v2-v5 are the files of
  [moderate-localmodel-20261020/design/designer-packet/](../moderate-localmodel-20261020/design/designer-packet/) plus
  its `amendments-v5-final.md`). A separate fresh read-only reviewer answered `VERDICT: CHANGES` and its final
  version is frozen word for word ([design/review-output.md](design/review-output.md)); not re-reviewed. No integrator
  edits. The case text is owner-approved under D96; Travis's sign-off of the Moderate case texts is not otherwise
  changed.
- **Changes the review made:** the picker screen label without a space (`after-relaunch-picker`); scrolling allowed in
  the inspector (arrow keys banned only in the picker); no second Esc if the first one already closed the popup; the
  missing-row note no longer cites G-1.
- **Freeze:** [design/FROZEN.sha256](design/FROZEN.sha256).
- **Separate:** amendment v7 (D97: MOD-12, MOD-23, MOD-29) is its own record; the two do not overlap.
