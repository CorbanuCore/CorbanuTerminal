You are an accounting planning worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding), the corbanu-terminal-development skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/corbanu-terminal-development/SKILL.md) for how
plans and sprints are written, and the agent-cost-accounting plan/sprint files (PF-60-*, under docs/sprints/ and
qa/portfolio/agent-cost-accounting/). Key input: the independent review
qa/portfolio/agent-cost-accounting/pf-60-s03/body-review-20261008/review.md (Blocker 1 and Majors 2-5) and
acct-s03a.result.md in the workers folder.

Travis approved on 2026-10-08 a new collection-correctness sprint that runs BEFORE PF-60-S04, covering the review's
Blocker and four Majors:
 1. kimi-code subscription work recorded as money spent;
 2. plan work with no catalogue price shown as "Pay per use" (plan-ness stored only on the price record);
 3. unversioned stored-bytes change from 2026-09-21 (verify with real old ledgers / fixtures; version it either way);
 4. compaction, web search and image generation discard a paid response when accounting fails (make them best effort:
    record as unrecorded, return the response);
 5. ephemeral sessions and guardian review forks silently uncollected (attribute forks to the parent; warn + list the
    rest in the coverage audit).
His requirements for the billing basis (item 1/2):
 (a) the basis (subscription vs pay per use) is DECLARED per provider (in provider config / the built-in provider
     table), not guessed from the auth type;
 (b) INTELLIGENT DEFAULTS that match what each built-in provider's default user configuration is expected to be (e.g.
     claude-plan / ChatGPT login / kimi-code membership URL = subscription; plain API-key providers = pay per use).
     Research each built-in provider's real billing model from its official docs (cite them) and list the default for
     every built-in provider in a table;
 (c) a clear specification of how "both" behaves (a user with a membership AND an API balance on the same provider,
     or a provider that can bill either way). DO NOT DECIDE (c). Write it up as 2-4 concrete options with a worked
     example each (what /cost shows, what is counted as spent, what the user configures), pros/cons, and your
     recommendation. Travis wants to discuss it before implementation starts.
Deliverables (docs only, no code):
 - A new sprint spec in the accounting plan's sprint folder, numbered/named per the repo's conventions so it sits before
   PF-60-S04 (adjust the plan's order/index and PF-60-S04's dependency accordingly), with goals, acceptance criteria
   (testable, including real-provider checks and an independent code-blind acceptance like S03's), out-of-scope, the
   (a)/(b) design, and a section "Open decision: 'both' behaviour" pointing to the options memo. Status: draft, blocked
   on the 'both' decision.
 - The options memo for (c) next to it.
 - Run the repo's plan/sprint checkers; they must pass.
Don't touch PF-60-S03's acceptance state (a separate re-check is running). Docs-only PR to main; merge when green
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f).
Final message: the sprint path, a plain-language summary of the 'both' options and your recommendation (this goes to
Travis verbatim, so keep it short and jargon-free), and the PR number.
