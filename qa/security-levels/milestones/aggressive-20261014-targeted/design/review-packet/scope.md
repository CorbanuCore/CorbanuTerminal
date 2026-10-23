# Scope of the targeted re-run (coordinator brief, owner decisions of 2026-10-14)

Cases to execute on the new candidate (origin/main 4440d2d4ce, which includes #752, #724, #662 and #742):

- Failed or blocked in the third run, cause fixed or amended: AGG-10, AGG-T03; the #740 cases AGG-11, AGG-T04, AGG-R13,
  AGG-R18, AGG-T06; AGG-01, AGG-25 and AGG-R10 (amended by the owner).
- Fresh attempts of the cases blocked by a model decline: AGG-18, AGG-R07, AGG-R03, AGG-R14, AGG-20.
- Regression sample of third-run passes touched by #662 or #752: AGG-02, AGG-04, AGG-37 (they assert what applies
  before the restart, which #662 changes) and AGG-R12, AGG-R17 (Grant once on .codex/.agents writes and on an in-folder
  hooks file next to a .codex write, which #752 changes). AGG-R12 is also the setup for AGG-R18.

Not re-run: every other case keeps its third-run result. AGG-T01/T02 wait for a pending owner decision (#739); AGG-R06
needs Claude Code in the guest (a named limit).

#752 (merged 2026-10-14) fixed #736/#737/#740: wording and when Grant once is offered; the owner's earlier decisions
(#620 option B and others) already describe that behaviour, so no amendment is drafted for it. Say so if you disagree.
