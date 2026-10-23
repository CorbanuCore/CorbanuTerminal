I fixed R1, R1b, R2, R3 and 13b in PR #318, which passed CI and is merged into main (`290bb28532`). I didn't accept or archive PF-60-S03, and the confirming acceptance re-run is still owed.

**Decision for you (13b):** a default build now answers every `/usage requests…` form with one line: "Per-request cost history is not part of this build." It shows no cost page and no next step.
- **Why:** the sprint record's 2026-09-16 answer says shipping `/usage requests` to users is not authorised, and `/cost` is developer-only.
- **What else changed in the default build:** the `/usage` menu no longer offers "Recorded requests". The `/usage` description, its usage error and the signed-out hint no longer mention the per-request view.
- **The alternative** was to keep the page and reword its next step, but that still shows a page that can never fill. This is recorded in the sprint file; change it if you prefer the other way.

**What changed:**
- **R1 / R1b:** covered by the 13b line above. In a developer build, a future date still says "Run /cost for today", which now leads to a working page.
- **R2:** a week or month bucket that reaches today reads "Cost so far — this conversation" and "In progress — totals so far, recorded through …". It counts toward the range total and has no next step. Weeks after today read "Not started yet". Buckets cut short by retention, by the requested range or by unverified days are still marked partial.
- **R3:** the range header and the day pages now take the floor from the same place, so both say "daily totals kept since 2025-10-09".
- **Outside the sprint's listed file scope** (noted in the sprint file): one state-crate retention helper, `slash_command.rs`, and some snapshot files.

**Gate evidence:**
- **Tests:** the focused TUI and state tests pass on both feature sets: 613 on the default build, 614 with `developer-accounting`. The existing `cost_scope` tmux test failed once, then passed on its automatic retry. It did the same before this change.
- **Linux clippy** on the RTX box, `-D warnings`, with and without the feature: clean on the final code (`248aaaa46`).
- **Review:** Opus 5.5 High through installed `corbanu exec`, read-only. Round 1 asked for changes over one Major: buckets after today still hit the same dead end. Round 2 approved with nits, and I fixed those nits too.
- **TUI check:** freshly built debug binaries for both builds, each in a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, after a real GLM 5.2 turn on zai. The key was only on the consuming `exec` command, and a scan found it in no file. The cost shown, 0.02165212 USD, matches my own calculation.
  - Captures of every fixed screen and both review reports are in `qa/portfolio/agent-cost-accounting/pf-60-s03/fix-289-residuals-20261008/`.
- **#289:** I commented with the evidence. It was already closed.

**Still open** (tracked in new issue #325):
- A bucket that ends after the last recorded request but isn't in progress still reads "Partial — excluded". Examples: yesterday, or the current hour, before any turn since.
- A request the provider rejected with a 401 is still counted as "billed".

**Cleanup:** my build target, worktree, binaries and scratch homes are removed on the Mac, and my worktree, target and scripts on the RTX box. The local branch `fix/pf60-s03-289-residuals` was not deleted.