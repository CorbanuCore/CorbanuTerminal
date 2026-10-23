You are an independent senior security and Rust reviewer (Opus 5.5, high effort). Third pass on the Aggressive vault-rule matching change: review the last commit (`git show HEAD`) against your second-pass findings, summarised below. Do not edit files; read the code and report.

Second pass (request changes) found: heredoc bodies never scanned; splitting only one level and not on shell punctuation (`$(date);corbanu`, `cd /tmp&&corbanu`, `eval '…'`, `python -c`, `git -c alias`); option skipping capped at 16 words; quadratic/allocation-heavy work; Windows/npm launcher suffixes (`.cmd`, `.ps1`, `codex.js`); lowercase-only keys missing mixed-case user rules; duplicate matched rules; docs implying completeness; tests that bypassed `commands_for_exec_policy` and no strict zsh-fork test.

What changed:
- `strict_forbidden_matches` now runs on the original command (not the parsed commands), so heredoc bodies are included.
- Every argv word is split into one flat token stream on whitespace, quotes, `\`, `^`, `;&|()<>{}[],=$!` and `env -S`'s `\_`; this covers scripts and strings at any nesting depth without a runner list.
- A token is a candidate program only if `policy.rules()` has an entry for one of its keys (file name as written and lowercased, launcher suffix stripped, attached `-…S` cluster prefix dropped).
- When the next token starts with `-`, up to 256 following words may be skipped (`corbanu -c k=v … vault`).
- Matches are deduplicated; a shared `apply_strict_forbidden_matches` is used by the normal path and the zsh-fork path.
- Docs say best effort / not a containment boundary.
- Tests: agent-shaped `bash -lc` scripts through `create_exec_approval_requirement_for_command` (heredoc, `2>&1`, `eval`, `$(date);`), a "strict only adds refusals" comparison, and a zsh-fork strict test.
- Not changed: nested `corbanu exec` (your finding 6) is an Aggressive design question for the product owner; over-matching (finding 9) is documented.

Look for: remaining practical bypasses for agent-issued commands, any way this can widen an allow, performance on pathological inputs, painful false positives, and test validity.
