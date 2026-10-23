You are the P1 security worker for issue #391. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding), sec-s09.result.md and
sec-probe390.result.md (if present) in that folder, `gh issue view 391` (including Travis's decision comment), #389 and #390
(both closed with evidence), and the archived PF-27-S05/S08/S09 records.
Travis decided 2026-10-10: once #389 and #390 pass (they have), turn `broker_model_auth` ON BY DEFAULT for the Aggressive
security level first; Moderate/Permissive stay as they are (a separate later decision). Implement exactly that:
- Aggressive enables brokered model-client auth by default on every OS where the broker exists (Windows via PF-27-S09; check
  macOS/Linux status from the archived records — if those OSes already default it under Aggressive, keep; if the broker's
  model-auth path isn't supported on an OS, leave it off there and say so). An explicit user setting still wins.
- Fail closed under Aggressive: if the broker can't start, model requests are refused with a clear message naming the
  setting and how to change level, never sent direct.
- Update user docs, the security-level inspector/`/security` view text, and the P1 plan/feature-flag records.
Tests: unit/integration tests for the default matrix (level x OS x explicit override) and the fail-closed path. Gate:
focused tests; Windows clippy (real machine, toolchain under C:\CorbanuQA, keep C:\CorbanuQA\target, your own dir
C:\CorbanuQA\a391\; ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net;
another worker may use C:\CorbanuQA\l398 — don't touch it) and Linux clippy on RTX (ssh ambient@100.99.88.49), -D warnings;
one independent Opus 5.5 High read-only review via installed `corbanu exec`; a tmux functional run on the real Windows
machine and on macOS/Linux under Aggressive with GLM 5.3 Flash (`-m glm-5.3-flash -c model_provider="zai"`, key via
`ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"` on the consuming command; disposable homes,
CORBANU_TEST_NO_NATIVE_KEYRING=1 except where Credential Manager is the point) showing brokered requests by default and the
fail-closed message; a short SOP demo video (qa/demos/README.md, leak-scanned). PR to main; merge only when ALL checks are
green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close
#391 with evidence. Clean up Windows dirs/key files and build targets. Report as in sec-common.md.
