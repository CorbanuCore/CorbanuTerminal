You are an accounting bug-fix worker for PF-60-S03 (the developer-only /cost screens). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding: keychain rule, disk hygiene, per-fix
gate, report format) and acct-fix1.result.md, acct-s03a.result.md in that folder.

The second independent re-run (PR #315, qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/
rerun2-20261008/README.md; fetch the PR branch if not merged yet) found these #289 residuals still failing on main 886f19c:
- R1: default build (no developer-accounting), `/usage requests <past day>` and today's page say "Unavailable — accounting
  ledger not installed. Next step: if this build records costs, send a turn…", even after a real turn. The next step
  leads nowhere.
- R1b: default build, `/usage requests <future day>` says "Run /usage requests for today", which leads to the same page.
- R2: a week/month bucket that reaches today is titled "Bucket unavailable" / "Partial bucket — excluded from totals"
  with a next step ("send a turn … then select Refresh") that cannot complete it. A bucket that includes today is simply
  in progress: title and wording must say so (e.g. "so far"), include what is recorded up to the coverage end, and not
  offer a next step that can't change anything.
- R3: the range header says "oldest daily total kept none" while day pages in the same home say "daily totals kept since
  2025-10-09". Make both read from the same source and agree.
- Criterion 13b (default build opening the "Cost — this conversation" page for `/usage requests`): `/cost` is
  developer-only. Unless the sprint/spec docs say otherwise, the default build must not present a cost page it can never
  fill: `/usage requests…` in a default build should say plainly, in one line, that per-request cost history is not part
  of this build (no dead-end next step). Record what you chose and why in the sprint file; Travis may revisit it.
Fix R1, R1b, R2, R3 and 13b with regression tests (default build AND `--features developer-accounting`). Plain, short
user wording (house style: no jargon, no "ledger" unless the docs already use it for users). Comment on/close #289 with
the evidence when done.
Gate: focused tests both feature sets, Linux clippy on the RTX box (ssh ambient@100.99.88.49) with -D warnings, one
independent Opus 5.5 High review via installed `corbanu exec` (read-only), and a short TUI check with the freshly built
debug binaries (disposable home, CORBANU_TEST_NO_NATIVE_KEYRING=1, GLM 5.2 on zai with the key only as
`ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"` on the consuming command) capturing each fixed screen.
Work in a new worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main, merge when green
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Remove your build target when done.
Do NOT accept or archive PF-60-S03; Travis does that. Report as in sec-common.md.
