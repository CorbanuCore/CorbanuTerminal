You are a Windows security worker (P1 security hardening). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding) and sec-s09.result.md in that folder,
then issues #390 and #391 (`gh issue view N`) and the archived PF-27-S06/S08/S09 records.
Context: Travis decided (2026-10-10, #391) that `broker_model_auth` stays default-off until #389 and #390 pass, then turns
on by default for Aggressive first. You own #390.
Fix/prove #390: in the EXCLUSIVE pipe-squat case (a hostile process creates the first instance of the broker's pipe
names with FILE_FLAG_FIRST_PIPE_INSTANCE, or max instances 1, before Core/broker start), Core must refuse the squatter via
the checked-pipe server process-id check, never send any frame or key to it, and fail closed with a clear user-facing
error — never fall back to direct requests. Also cover the squatter racing in after broker restart. If the current
behaviour falls short (e.g. hang, unclear error, or the broker silently uses a weaker pipe), fix it. Add regression
probes to the `windows-security-probes` CI job (windows-2022) and run them on the real Windows 11 machine elevated and in
a normal (medium-integrity) session, over SSH (toolchain under C:\CorbanuQA; keep C:\CorbanuQA\target; your own dir
C:\CorbanuQA\p390\):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password only via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; never print it. Live model turns
use GLM 5.3 Flash (`-m glm-5.3-flash -c model_provider="zai"`, ZAI key via `ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"`
on the consuming command, piped over ssh stdin to a file in your dir, deleted afterwards).
Gate: Windows clippy -D warnings for touched crates; Linux clippy on RTX (ssh ambient@100.99.88.49); focused tests; one
independent Opus 5.5 High read-only review via installed `corbanu exec`; a short SOP demo video of the refused squat
(qa/demos/README.md; leak-scanned). PR to main; merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close #390 with evidence. Clean up your
Windows dirs/tasks/key files and build targets. Report as in sec-common.md.
