# PF-56 external review brief

You are the single authorized external reviewer for Corbanu Terminal's unified provider authentication initiative. Use exactly Claude Fable 5 at high effort. Do not spawn subagents. This is read-only review: do not edit files, commit, install dependencies, run network-facing application flows, or access credentials.

Repository: /home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0
Implementation range: f7356a94e032234022a462d65b576a7de2854859..21cf3199f2
Current qualification HEAD: cf201dff78

Review the implemented PF-48 through PF-55 provider catalog, status/eligibility, API-key, OpenAI account, Claude account, onboarding, provider management, startup/model picker, resume, and native-child convergence. Focus on concrete correctness, security, credential-custody, exact provider identity, cancellation/stale-result behavior, blocking/concurrency, restart persistence, command-auth handling, and test gaps. Enforce these frozen policies:

- configured providers are active by default; current is separate;
- deferred Corbanu runs only after provider selection is done and never overrides current;
- deactivation retains credentials and current-provider deactivation requires exact usable replacement;
- successful real environment/managed/account/command authorization permits normal selection;
- failed command auth stays visible without invented enrollment UI;
- unusable current blocks request/spawn after cancel and never silently switches;
- resumed replacement is session-specific unless global selection is explicitly changed;
- no raw credential crosses TUI/AppEvent/debug boundaries;
- no regex is allowed on LLM call paths.

Use git diff/show, rg, sed, and read-only inspection. You may run narrowly targeted existing tests only if needed, but prefer source/test review. Treat prior sprint ledgers as claims to verify, not proof.

Output a concise Markdown report with:
1. exact model/effort you are running;
2. reviewed range and key paths;
3. findings ordered by severity, each with path/line, failure mode, impact, and generalized repair;
4. test/evidence gaps;
5. explicit CLEAN verdict if and only if there are no actionable in-scope findings.

Do not praise generally. Do not invent evidence.
