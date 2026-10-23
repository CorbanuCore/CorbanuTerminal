You are an independent security reviewer. Review the change on this checkout: `git diff origin/main...HEAD` (branch sec-tui5-nested-bins, PR #226 in CorbanuCore/CorbanuTerminal). Read the code; do not modify files.

Context: Corbanu's `/security` level Aggressive makes agent commands run in a sandbox where the Corbanu home is read-only and its vault store unreadable. `corbanu` refuses (or holds to Aggressive) agents started by agent commands ("nested launches", PR #219). The standalone `codex-exec`, `codex-tui`, `codex-app-server` and `codex-mcp-server` binaries skipped that check. This change moves the stored level and detection into a new crate `codex-security-level` (codex-rs/security-level) unchanged, and makes the standalone binaries refuse nested launches in both `refuse` and `pass` modes.

Check:
1. The move is behaviour-preserving for `corbanu` (compare with the removed code in codex-rs/tui/src/security/{level,nested}.rs); visibility changes are safe.
2. Each standalone binary runs the check before anything that could start an agent or read credentials; nothing (arg0 dispatch, the codex-linux-sandbox alias) is broken.
3. Bypasses: ways an agent command under Aggressive could still start an agent through these binaries, or other binaries that should be covered.
4. Tests (exec/tests/suite/nested_launch.rs, tui/tests/suite/nested_launch.rs) are meaningful.
5. Build hygiene: Cargo/Bazel (BUILD.bazel, workspace members), cargo-shear (unused deps).

Output: a verdict (approve / approve with fixes / request changes) and numbered findings with file:line, severity and a concrete fix. Be concise.
