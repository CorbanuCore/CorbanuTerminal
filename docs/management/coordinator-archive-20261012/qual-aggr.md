You are the integrator for the P1 security milestone **"Aggressive ships"** (qualification lane). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding), then on origin/main: root AGENTS.md
(sections "Code-blind functional test design", "Model for adversarial functional cases" incl. the 2026-10-10 GLM 5.3 Flash
amendment), qa/code-blind-functional/ (README.md, isolated-execution.md, designer-prompt.md, executor-prompt.md,
RECORD_TEMPLATE.md, check.py), docs/plans/active/p1-security-hardening.md (milestone rows, PF-13-S07, PF-83 TUI rows,
implementation sequence), the completed P0 plan docs/plans/completed/main-2026-10-08-p0-security-levels.md (milestone
definitions), and the prior VM harness and records: /Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915
(README.md, NATIVE-MACOS.md, increment-28 driver, increment-33/34 records) plus pf83-glm-run3.md/.result.md in the
workers folder.
Goal: run the milestone end to end and bring Travis a sign-off request with honest evidence.
1. Candidate: current origin/main (#413 merged as 9ead7a5755: broker_model_auth default-on under Aggressive) — record the exact commit and
   build the packaged macOS candidate (and note Windows/Linux applicability). First refresh the plan's stale rows: the
   PF-83 "typed text can approve" bug was fixed by #174/PR #187; "Enable full access" message, pf83-unran-18 and the
   zai-anthropic max-output rows say "verify on the next candidate" — verify them on this candidate as cases.
2. Packet: user-intent brief for Aggressive (what a user who picks Aggressive should experience: protections on, broker,
   secret handling, egress policy, approvals, picker/inspector, what's refused and how it's explained), constraints,
   screenshots of the real TUI from the candidate (picker, inspector, approval prompts, refusals, errors/recovery). No
   code, no commit messages, no results. Outside any source checkout.
3. Fresh designer (no inherited context, packet only, designer-prompt.md; installed `corbanu exec`, Opus 5.5 High,
   read-only, instruction-only isolation recorded honestly). Freeze design.json with SHA-256s; commit/checkpoint it
   before execution.
4. Isolated execution in a VM per isolated-execution.md: reuse the existing macOS guest "macOS 3" (agent@192.168.64.3;
   keep "macOS 2" stopped). Travis's standing instruction: VMs are expensive to create — programmatically erase data on the
   guest as needed for a fresh run instead of making new VMs. Re-measure the isolation gate first (source hosts denied,
   inference reachable, native-prompt guard, negative probes from isolated-execution.md). Product under test runs on GLM 5.3
   Flash (`-m glm-5.3-flash -c model_provider="zai"`); if the guest's allowlist only covers GLM 5.2 routes, update the guest
   pf allowlist for the same api.z.ai host and record it; GLM 5.2 only with a recorded reason. ZAI key reaches the guest
   only through the owner-approved path used in increments 33/34. Fresh executor context per case group (executor-prompt.md).
   Record every refusal verbatim.
5. Evidence check by an independent fresh agent (not the executor). Run qa/code-blind-functional/check.py.
6. Records: a docs-only PR to main with the milestone record (qa/security-levels/milestones/aggressive-<date>/ or the
   location the procedure names), plan row updates, and results.json/design.json/evidence index; key scan before commit.
   Merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`;
   never force-push; never rm -f). File issues for defects found; do not fix code yourself.
7. Do NOT mark the milestone passed: it needs Travis's named sign-off. End with a plain-language sign-off request listing
   passes, failures, blocked/out-of-scope cases and known limits.
If a step needs an owner action (VM login, guest password, network), stop that step and report exactly what's needed
(password only via `corbanu vault auth-helper` on a consuming command if one exists in the vault). Keep context lean.
Report as in sec-common.md.
