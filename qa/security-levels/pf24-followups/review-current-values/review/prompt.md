You are an independent senior security and Rust/TUI reviewer (Opus 5.5, high effort). Review the last commit on this branch (`git show HEAD`, base origin/main 93707fbb1b). Do not edit files; read the code and report.

Context: PF-24-S03 (merged) added a flagged `/security` level picker. Its Aggressive review screen listed only Aggressive's value for each mapping row (`security::aggressive::ROWS`: Sandbox, Approvals, Network, Vault, Child agents). Code-blind case B-06 asks that the review show what the user would lose, using their real settings. Example: Config C, which has on-request approvals, an extra writable root, network on, live web search and an environment that passes everything through.

This commit adds `aggressive::current_values(&Config)`, computed when `/security` opens. It returns one string per row, and the review shows "• <Row> now: <current>" above "Aggressive: <value>".

Check:
- Accuracy. Can any "now" value overstate protection that is not actually in force? Look at sandbox summary vs named profiles, network vs web search, vault store readability with `can_read_path_with_cwd`, the secret-env probe vs real `create_env_from_vars` behaviour (`set` entries, `inherit`, `filters`), shell snapshot/login shell, and roles. This is a security display and must never claim more than is enforced.
- Whether the config used (the chat widget's `self.config`) matches the session's effective settings after `/permissions` changes, resume or fork.
- Rendering: wrapping, very narrow widths, height on a 24-row terminal. The body is clipped above the footer.
- Tests, snapshots and style against codex-rs/AGENTS.md and codex-rs/tui/AGENTS.md.

Output Markdown: a verdict (approve / approve with fixes / request changes), then numbered findings each with severity, file:line, the problem, and a concrete fix. Be concise.
