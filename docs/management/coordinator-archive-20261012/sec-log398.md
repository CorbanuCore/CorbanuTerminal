You are a security bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding) and log-redact1.result.md in that folder (PR #387: the codex-log-guard crate, target capping and the
credential-shape redaction layer), then issue #398 (`gh issue view 398`).
Fix #398: the Windows sandbox writes its own command log, outside codex-log-guard, so a command line carrying a
credential (`curl -H "Authorization: Bearer ..."`, `--password=...`, `X-Api-Key: ...`, `?api_key=...`) is written
verbatim. Required end state: every sandbox/exec command log on Windows, Linux and macOS (audit them all, including any
approval/command history the app persists, rollout files excepted only if they are the user's own transcript by design —
document that decision) passes command text through the same redaction layer before writing; add the Bazel-compatible
enforcement follow-up from #387 if feasible (e.g. a test that every tracing subscriber construction in the workspace goes
through codex-log-guard, implemented without filesystem source scans that break under Bazel — or explain why not).
Regression tests with sentinel credentials in sandboxed commands. Real Windows 11 machine for the Windows path (toolchain
under C:\CorbanuQA; keep C:\CorbanuQA\target; your own dir C:\CorbanuQA\l398\):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
(another worker, sec-probe390, uses the same machine in C:\CorbanuQA\p390 — don't touch it). Live turns use GLM 5.3 Flash
(`-m glm-5.3-flash -c model_provider="zai"`; key via `ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"` on the
consuming command; over ssh via stdin to a file you delete). Gate: Windows + Linux (RTX, ssh ambient@100.99.88.49) clippy
-D warnings; focused tests; one independent Opus 5.5 High read-only review via installed `corbanu exec`. PR to main; merge
only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never
force-push; never rm -f). Close #398 with evidence. Clean up Windows dirs/key files and build targets. Report as in
sec-common.md.
