You are the accounting implementation worker for Corbanu Terminal (a Codex fork).
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/acct-scope-62-20261002
Branch: fix/acct-scope-62-20261002, based on integrate/management-workstreams-20260911 at 1aca553905.
Read AGENTS.md, codex-rs/AGENTS.md, codex-rs/tui/AGENTS.md, docs/tmuxHarness.md and the sprint
docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md first.

## Goal: finish the sprint item `acct-scope-62` (the /cost inspector)
Review found:
1. The inspector shows an unqualified zero ("No recorded attempts in this day", subtotal 0) for a day on which the store holds collected, priced attempts. It scopes to the current root conversation rather than the day, and says nothing about that.
2. A mixed-provider run omits custom-provider usage without warning.
3. A missing price shows a zero subtotal with no next step.
4. The reader audit's own monetary checks are unbound, so a page that loses its amount passes silently.

Some of this may already be fixed on the branch (recent commits removed `$0` beside unknown estimates). Verify each point against the current code first, record what is already fixed, and fix only what remains. A page must never imply zero when the evidence is "not in this scope" or "unknown". Say what the scope is and what to do next.

The collection and /cost UI are behind the debug-only `developer-accounting` feature. Build and test with `--features codex-core/developer-accounting,codex-tui/developer-accounting` where needed.

## Requirements
- Keep changes in codex-rs/tui/src/chatwidget/tokens.rs (and its tests) and codex-rs/state accounting read paths. Follow the module-size rules in codex-rs/AGENTS.md.
- Add regression tests for each of the four points, plus snapshot tests for the rendered pages. Include a narrow screen and a mixed-provider day.
- Finalization order: `just fix -p <crate>`, `just fmt`, then `just test -p codex-tui` and `just test -p codex-state` (never bare cargo test).
- Interactive proof per docs/tmuxHarness.md: add a tmux scenario under codex-rs/tui/tests/suite/ with a mocked provider. Seed priced attempts on a day across two conversations and a custom provider, open /cost, and assert the scope wording, the non-zero totals and the missing-price next step. Exit via /exit. Run with `CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all <selector> --retries 0` and record the evidence checklist from the doc.
- Commit with clear messages and push the branch to origin. Do NOT push to the integration branch; the coordinator reviews and fast-forwards it.
- Update the sprint's Done/Remaining ledgers for acct-scope-62 with the evidence, in the same branch.
- Never read or print secrets or real credential stores. Tests use synthetic fixtures only.

## Report (your final message)
Per point: was it already fixed or newly fixed, the files changed, and the tests added. Then the commit SHAs, test results, the tmux scenario name and its result, and anything left open.
