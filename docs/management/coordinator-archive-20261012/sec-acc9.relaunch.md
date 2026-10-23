RECOVERY NOTE: a previous run of this brief (on Claude) was stopped by the provider content filter after starting on the Windows machine. Check C:\CorbanuQA\acc9\ for its partial work (reuse what's sound, it is code-blind-safe), then continue. Same rules apply.

You are the INDEPENDENT functional acceptance executor for sprint PF-27-S09 (Windows model-client auth through the
credential broker, flag `broker_model_auth`). Be code-blind: do NOT read implementation source (codex-rs/**), PR diffs,
lane briefs, worker logs/results or review files under .codex-work. You may read only: the PF-27-S09 sprint record and
its acceptance criteria (docs/sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md on
origin/main), the archived PF-27-S05/S06/S08 records for context, qa/demos/index/PF-27-S09.md and any qa/ evidence the
sprint record links, user-facing docs/, and /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding rules: secrets, disposable homes, never the real keychain, disk hygiene).
Task: on the real Windows 11 machine, build the release-candidate binary from the current origin/main commit yourself
(toolchain under C:\CorbanuQA; keep C:\CorbanuQA\target; use your own dir C:\CorbanuQA\acc9\) and verify every
acceptance criterion from the outside, as a skeptical tester:
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password (only if needed) via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; never print it.
Model key for live turns: GLM 5.2 on zai, ZAI key fetched on the Mac with
`ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"` on the consuming command and piped over ssh stdin to a
file in your own dir (delete it afterwards); never echo it. Cover at least: flag off = unchanged behaviour; flag on with an
env key and with a vault key, in a normal (medium-integrity) console-equivalent session and over SSH; requests refused
(not sent direct) when the broker is absent or killed mid-session; the key not recoverable from Core's process memory
(use an external dump/scan tool of your own, e.g. procdump or a small Rust/PowerShell reader, searching for the key);
pipe server identity checks (a fake pipe squatter must not receive the key); restart/resume. Record expected vs actual
for each AC, with commands and outputs (redacted), in qa/security/pf-27-s09/independent-acceptance-<date>/README.md via
a docs-only PR to main (merge only when ALL checks are green, never with checks pending; `gh pr merge N --merge
--match-head-commit <sha>`; never force-push; never rm -f). Verdict: ACCEPT / ACCEPT WITH LIMITS / REJECT, with
counterexamples filed as GitHub issues. Clean up all your Windows dirs, scheduled tasks, credentials and key files.
Report as in sec-common.md, and state exactly what Travis needs to decide.
