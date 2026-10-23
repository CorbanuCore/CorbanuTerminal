**Verdict: request changes.** Matching every argv suffix is the right direction, and the change cannot widen an allow. The problem is coverage: the scan never sees heredoc bodies, and it splits strings only one level deep. That leaves several simple, ordinary-looking shell commands that get past the rule.

I reviewed by reading the code only. Nothing was compiled or run (the sandbox is read-only), so the bypasses below come from tracing the code paths, not from executing them.

**Does it widen an allow?** No. `strict_forbidden_matches` keeps only `Decision::Forbidden` matches, and both callers change the decision only to Forbidden (exec_policy.rs:376-385, unix_escalation.rs:618-630). A `host_executable` path restriction is skipped on the strict path, so it can only refuse more. However, the regression test that proved this was deleted (see 7).

**What a bypass costs today.** Aggressive runs with `approval_policy = untrusted`, a denied-read vault store and a scrubbed environment. So a missed match usually turns "refused" into "the user is asked to approve". It runs silently only when an allow rule or approved prefix covers the outer program, as the test's `env` allow rule does.

## Findings

1. **High: exec_policy.rs:376-381.** Heredoc scripts are never scanned. The strict pass runs only over `commands_for_exec_policy(command)`. For heredoc scripts, `parse_shell_lc_single_command_prefix` returns only the first command's words, so `bash -lc "sh <<'EOF'\ncorbanu vault list\nEOF"` is checked as `["sh"]` and the body is ignored.
   *Fix:* in strict mode, also scan the original `command`, e.g. `commands.iter().chain(std::iter::once(&command.to_vec()))`, or scan `command` separately.

2. **High: exec_policy.rs:928-935 and 957-972.** Splitting stops after one level and does not split on shell punctuation.
   - **One level only.** Only top-level argv entries are split. Any script the plain parser rejects (because of a redirect, `$(…)`, `&`, …) is split once with shlex, and a quoted inner command stays as one word. These all get through:
     - `sh -c 'corbanu vault list' 2>&1`
     - `env -S 'corbanu vault list' >/dev/null`
     - `x=$(pwd); bash -c "corbanu vault list"`
   - **The plain parser removes the shell word.** `bash -lc "eval 'corbanu vault list'"` reaches the strict pass as `["eval","corbanu vault list"]`, with no shell word ahead of the string, so it is never split. The same applies to `python3 -c`, `perl -e`, `node -e`, `ssh host '…'`, `su -c`, `script -c`, `watch`, `flock -c`, `osascript -e`, and `git -c 'alias.v=!corbanu vault list' v`.
   - **shlex does not split on `;&|()<>`.** In a non-plain script these get through:
     - `echo $(date);corbanu vault list` gives the single word `$(date);corbanu`
     - `cd /tmp&&corbanu vault list` gives `/tmp&&corbanu`, which `program_key` turns into `tmp&&corbanu`
     - `echo hi>/dev/null;corbanu vault`
   - **shlex's quoting is not bash's.** `` echo $'\'' ; corbanu vault list ; # ' `` runs `corbanu` in bash. shlex reads the middle as one quoted word and still returns `Some`, so the whitespace fallback never runs.
   - *Fix:* split recursively. Re-split every produced word that contains whitespace, carrying the "shell seen" state; this terminates because each split makes words strictly shorter.
   - After shlex, also split on `; & | ( ) < > { } \` = ,`. Better, add a second loose pass that removes `' " \` and splits on whitespace plus those characters. That also catches `$'…'` tricks and `subprocess.run(['corbanu','vault'])`.
   - Add to the runner list: `eval`, `source`, `.`, `python*`, `perl`, `ruby`, `node`, `ssh`, `su`, `script`, `watch`, `flock`, `xargs`, `git`, `osascript`, `ash`, `busybox`, `csh`, `tcsh`, `yash`, `nu`, `pwsh`, `powershell`, `cmd`.

3. **Medium: exec_policy.rs:905-909.** Leading options are skipped only up to 16 words. `corbanu -c a=1 -c a=2 … (9 pairs) vault list` puts 18 words before `vault` and gets through.
   *Fix:* first check `policy.rules().contains_key(&program)`, which rules out almost every word. Then, only for programs that have rules, try every later position with no limit.

4. **Medium: exec_policy.rs:898-935.** Pathological inputs are slow.
   - **Quadratic in argv length.** Every argument containing whitespace re-scans `command[index + 1..]`. In the zsh-fork path this is realistic: `sh -c '…' sh *.md` with many file names containing spaces. That scan happens while holding the policy read lock, on an async worker thread.
   - **Expensive per word.** Each word can cost up to 16 candidates × 17 cloned strings, with no early filter, so a large script costs tens of millions of allocations.
   - *Fix:* append only `.take(STRICT_MATCH_WINDOW)` of the remaining argv (suffixes starting there are already checked by `check(command)`). Check the program before allocating, and build candidates from slices. If you add a size cap, exceeding it must refuse or prompt, never pass.

5. **Medium: exec_policy.rs:880 and 943-953 (Windows and npm).**
   - The runner list has no `pwsh`, `powershell` or `cmd`, so any PowerShell script the parser can't flatten (e.g. `$x=1; corbanu vault list`) is never split.
   - Only `.exe` is stripped. An npm install produces `corbanu.cmd` and `corbanu.ps1` shims, and the npm package's `bin` is `bin/codex.js`, so `node …/codex.js vault` gives the key `codex.js`.
   - *Fix:* strip `.exe`, `.com`, `.cmd`, `.bat`, `.ps1` and `.js`; add the Windows shells; remove `^` (cmd's escape) when splitting.

6. **Medium (design, beyond name matching): exec_policy_tests.rs:2537.** The test asserts that `corbanu exec 'use the vault'` is allowed. Aggressive is applied as overrides by the TUI launch path, so a nested `corbanu exec` probably loads the user's plain config and does not inherit the vault rule. I did not check whether a nested process can reach the native credential store from inside the outer sandbox.
   *Fix:* either forbid the agent-launching subcommands of vault programs (`exec`, `resume`, a bare prompt, `app-server`, `mcp-server`), or pass Aggressive/strict to child processes through a marker the CLI honours.

7. **Low: tests (exec_policy_tests.rs:2499-2545, 2551-2606; unix_escalation_tests.rs:428 and others).**
   - The unit test calls `strict_forbidden_matches` on raw argv, which skips `commands_for_exec_policy`, the path where 1 and 2 live.
   - Add `requirement(&strict, …)` cases using agent-shaped `bash -lc` scripts: a heredoc, `sh -c '…' 2>&1`, `eval '…'`, `$(date);corbanu vault list`, and 18 option words.
   - There is no strict-mode test for the zsh-fork path; every provider in the tests sets `strict_rules: false`.
   - The deleted "never widens an allow" test should come back. For example, with a `git status` allow rule, `env git status` and `sh -c 'git status' 2>&1` must not become `Skip` under strict.

8. **Low: exec_policy.rs:948.** Because candidates are always lowercased, wrapped forms miss user forbidden rules whose program name has capitals (`env MyTool …` against a rule for `["MyTool"]`).
   *Fix:* try both the original basename and the lowercased one.

9. **Low: false positives (no code change needed, but document them).** The same `git commit -m "… corbanu vault …"` is allowed when the script parses as plain and refused once it gains a redirect. Fixes 1 and 2 will also refuse the common ``git commit -m "$(cat <<'EOF' …)"`` pattern. This is acceptable given the "refuse only" design. State it in the Aggressive description, and make sure the refusal reason tells the agent it can rephrase.

10. **Nit: exec_policy.rs:383, config_toml.rs:368-373, config.schema.json.**
    - `matched_rules` gets duplicate entries: one from the exact match plus one per matching skip candidate. Remove the duplicates.
    - The docs describe the matching as if it were complete. Say it is best-effort name matching, not a containment boundary.

## Out of reach for name matching

These need the sandbox, prompts, or the process interception to catch:
- **Names built at run time:** `c=corbanu; $c vault`, `$(printf …)`, `base64 -d | sh`, and `env -S '${X} vault'` (GNU `-S` expands variables).
- **Functions and aliases:** `f(){ corbanu "$@"; }; f vault`.
- **Other copies of the binary:** copies, hard links and symlinks. The zsh-fork path could canonicalise the program to catch symlinks.
- **Files written and then run:** scripts, Makefiles, npm scripts, git hooks, compiled helpers.
- **Anything an approved program starts by itself**, including interpreter API calls whose arguments are built at run time.
- **Windows 8.3 short names.**