# PF-56 Kimi external review brief

You are the single user-authorized replacement external reviewer for Corbanu
Terminal's unified provider authentication initiative. Run exactly Kimi 3.0
through Vercel at high effort. Do not spawn subagents. This is read-only review:
do not edit files, commit, install dependencies, access credentials, or launch
network-facing product workflows beyond this review conversation.

Repository: /home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0
Implementation range: f7356a94e032234022a462d65b576a7de2854859..21cf3199f2
Qualification allocation: cf201dff78db09e829344a164504a43ee5f31133

Review PF-48 through PF-55 provider catalog, status/eligibility, API-key,
OpenAI account, Claude account, onboarding, provider management,
startup/model-picker, resume, and native-child convergence. Focus on concrete
correctness, security, credential custody, exact provider identity,
cancellation and stale-result behavior, blocking/concurrency, restart
persistence, command-auth handling, and test gaps. Enforce these policies:

- configured providers are active by default; current is separate;
- deferred Corbanu runs only after provider selection is done and never
  overrides current;
- deactivation retains credentials and current-provider deactivation requires
  an exact usable replacement;
- successful real environment, managed, account, or command authorization
  permits normal selection;
- failed command auth remains visible without invented enrollment UI;
- unusable current blocks request/spawn after cancel and never silently
  switches;
- resumed replacement is session-specific unless global selection is
  explicitly changed;
- no raw credential crosses TUI, AppEvent, or Debug boundaries;
- no regex is allowed on LLM call paths.

Use read-only git, rg, sed, and file inspection. Treat sprint ledgers as claims
to verify, not proof. Do not run builds or tests; the controller owns execution.

Return a concise Markdown report with:

1. exact model, provider, and effort you are running;
2. reviewed range and key paths;
3. findings ordered by severity, each with path/line, failure mode, impact, and
   generalized repair;
4. test/evidence gaps;
5. an explicit `CLEAN` verdict if and only if there are no actionable in-scope
   findings.

Do not praise generally and do not invent evidence.
