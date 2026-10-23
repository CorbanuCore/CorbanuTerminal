You are the accounting lane worker for sprint PF-60-S05 (collection correctness, runs before PF-60-S04). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding: keychain rule, disk hygiene,
per-sprint gate), acct-plan1.result.md, acct-fix1.result.md and acct-fix3.result.md in that folder, the
corbanu-terminal-development skill, and on origin/main: the PF-60-S05 sprint record,
qa/portfolio/agent-cost-accounting/pf-60-s05/billing-basis-defaults.md and both-behaviour-options.md, and the review
qa/portfolio/agent-cost-accounting/pf-60-s03/body-review-20261008/review.md.
Travis decided on 2026-10-09: "both" behaviour = OPTION B (the billing basis follows the credential and route actually
used; the built-in table declares a basis per credential/route; an unknown route shows "billing basis not declared" with
a next step instead of a guess; paid overflow is never counted as spent and subscription work carries a plain note that
overflow isn't visible; the per-provider `billing` override stays for unusual setups). His earlier requirements stand:
(a) declared, not guessed from auth type; (b) intelligent defaults matching expected default user configs (the defaults
table); (c) = option B.
1. First a small docs PR: record the decision in the sprint record and the options memo, apply the memo's AC 12 wording
   for option B, unblock the sprint (status per procedure), allocate worktree/branch/base. Merge when green.
2. Implement the sprint in slices (PRs), covering Blocker 1 and Majors 2-5 of the review with the declared-basis design
   and option B. Each slice: focused tests with and without developer-accounting, Linux clippy on the RTX box
   (ssh ambient@100.99.88.49) with -D warnings, one independent Opus 5.5 High review via installed `corbanu exec`
   (read-only). Merge each when green (`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on conflicts;
   never force-push; never rm -f).
3. Real-provider checks per the sprint ACs (keys only via `X="$(corbanu vault auth-helper <label>)" <cmd>` on the
   consuming command, disposable homes, CORBANU_TEST_NO_NATIVE_KEYRING=1; never the real keychain). If a provider route
   needs a credential that isn't in the vault, record it as NOT VERIFIABLE and list it for Travis rather than guessing.
4. Short SOP demo videos for the visible behaviour (subscription work shown as not spent, "not declared" route, overflow
   note), leak-scanned, uploaded to the `demos` release, indexed in qa/demos/index/PF-60-S05.md.
Set the sprint to `ready` when the gate passes; do NOT archive (an independent code-blind acceptance comes next, then
Travis). Another worker (acct-archive1) is archiving PF-60-S03 concurrently; don't edit S03's record. Report as in
sec-common.md, including anything for Travis.
