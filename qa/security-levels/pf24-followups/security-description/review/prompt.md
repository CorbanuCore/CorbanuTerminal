You are an independent senior Rust/TUI reviewer (Opus 5.5, high effort). Review the last commit on this branch (`git show HEAD`, base origin/main 93707fbb1b). Do not edit files; read the code and report.

Context: PF-24-S03 (merged) added a flagged `/security` level picker. When the `security_levels` flag is on, or a non-Permissive level is stored, `security::launch::finish` installs a process-wide `LevelContext` with `picker_enabled = true`, and `SecurityView` opens `SecurityLevelPicker` instead of the read-only profile explorer. The slash-command description still said "explore security profiles and protection readiness (read only)". This commit makes the description follow `security::level::context()`.

Check:
- Is the context always installed before the command popup reads descriptions, including in onboarding, resume and remote/daemon paths?
- Are there other places that show the `/security` description or assume it is static, such as help, docs or snapshots?
- Is the wording accurate? The level takes effect at the next start, and Moderate shows as unavailable.
- Test quality, and style against codex-rs/AGENTS.md and codex-rs/tui/AGENTS.md.

Output Markdown: a verdict (approve / approve with fixes / request changes), then numbered findings each with severity, file:line, the problem, and a concrete fix. Be concise.
