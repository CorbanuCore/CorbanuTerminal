**Verdict: approve with fixes.** The change can only make a command Forbidden, so it never widens an allow. But several of the wrapper forms it says it handles can still slip through. Each fix below is small and self-contained. I only read the code; I couldn't build or run anything because the sandbox is read-only.

**Allow widening:** none found.
- **`bypass_sandbox`:** it is only computed in the Allow branch, and that branch is unreachable once the decision is Forbidden.
- **Amendments:** the requested amendment is computed but dropped for Forbidden.
- **Reason text:** `derive_forbidden_reason` picks the justification from the added prefix matches, as intended.
- **Sub-agents:** they inherit the parent's `Arc<ExecPolicyManager>`, so `strict_rules` carries over.
- **Guardian:** it still uses `Default`, so it is unaffected, which is correct.
- **Other paths:** unified exec (`process_manager.rs:1205`) and the shell handler both go through the manager, so they're covered. User `!` commands and apply_patch rightly sit outside exec policy.

## Findings

1. **Medium — `core/src/exec_policy.rs:888`: nine nested wrappers bypass the check.** `for _ in 0..8` stops after 8 unwraps and returns the remainder as is. So `env env env env env env env env env corbanu vault list` is checked as `env corbanu vault list`, which matches no rule.
   - **Fix:** use `loop`. Each pass strictly shortens `rest`, so it always ends. Alternatively, if a limit is kept, treat hitting it as Forbidden under strict.

2. **Medium — `exec_policy.rs:922`, `:943`: GNU long-option abbreviations bypass `env` parsing.** `getopt_long` accepts any unambiguous prefix, but the code only recognises the exact names:
   - `env --split 'corbanu vault' list`, `env --split='corbanu vault' list` and `env --s …` are treated as plain flags, so the program comes out as the split string or as `list`.
   - `env --chd /tmp corbanu vault` and `env --u HOME corbanu vault` make `/tmp` or `HOME` the program.
   - **Fix:** match the prefix against the full GNU list (`argv0, block-signal, chdir, debug, default-signal, help, ignore-environment, ignore-signal, list-signal-handling, null, split-string, unset, version`). Use the value-taking set `{argv0, chdir, split-string, unset}`, and treat an `=` suffix as an attached value.

3. **Medium — `exec_policy.rs:964`: `shlex` does not follow the `env -S` grammar.** Both GNU and BSD `env -S` treat `\_` outside quotes as an argument separator and expand `${NAME}`. `shlex` does neither.
   - `env -S'corbanu\_vault\_list'` splits to the single word `corbanu_vault_list`.
   - `env X=corbanu env -S'${X} vault list'` gives `${X} vault list`.
   - Both actually run `corbanu vault list`.
   - **Fix:** under strict, rewrite `\_` outside quotes to whitespace before splitting. Treat any `${` or `\c` in an `-S` operand as Forbidden; only refusal is possible here, and agents rarely use `-S`. Alternatively, substitute `${NAME}` from `NAME=VALUE` words already seen in the unwrap chain.

4. **Medium — `exec_policy.rs:903-908`: clustered `exec` options are not parsed.** Bash's `exec` takes `-cla NAME`. The loop only treats the exact word `-a` as taking a value, so `exec -ca name corbanu vault` and `exec -la x corbanu vault` leave `name …` as the program.
   - **Fix:** for `exec`, if a cluster contains `a`, take the value from the rest of the cluster or the next word, as `unwrap_env_args` already does.

5. **Medium — `exec_policy.rs:921`: BSD `env` options and case are not handled.**
   - FreeBSD `-L user[/class]` and `-U user[/class]` take a value, so `env -L root corbanu vault` yields `root …`. I haven't checked whether macOS `env` has these flags.
   - On macOS's default case-insensitive APFS, `ENV corbanu vault`, `CORBANU vault list` and `Corbanu vault list` all run the real binary. Neither the wrapper check (`:892-895`) nor rule matching (exact key, or basename via `executable_lookup_key`, which only lowercases on Windows) normalises case. This applies in lenient mode too.
   - **Fix:** add `'L', 'U'` to `SHORT_WITH_VALUE`. Under strict, also check the forbidden rules against an ASCII-lowercased program basename, and lowercase it before the `TRANSPARENT_WRAPPERS` lookup.

6. **Medium (if zsh-fork is enabled) — `core/src/tools/runtimes/shell/unix_escalation.rs:668`: intercepted execs skip the unwrap.** The intercepted-exec check calls `Policy::check_multiple_with_options` directly. For a complex script, such as `if …; then env corbanu vault; fi`, the up-front check can't parse the commands, so it only prompts. zsh-fork then intercepts `/usr/bin/env corbanu vault …` and checks it without unwrapping. The child `exec` that `env` makes is not intercepted. Aggressive does not turn zsh-fork off.
   - **Fix:** move the forbidden-through-wrappers step into a shared helper, e.g. `fn strict_forbidden_matches(policy, commands, opts) -> Vec<RuleMatch>`. Expose `strict_rules()` on the manager and call the helper from `evaluate_intercepted_exec_policy` as well.

7. **Medium (existing gap, larger than this change) — `exec_policy.rs:978-1016`: complex scripts never reach rule matching.** Anything `parse_shell_lc_plain_commands` rejects falls back to `[bash, -lc, script]`, so no forbidden rule applies, even to the plain program name. That covers:
   - redirects: `corbanu vault list 2>/dev/null`
   - assignment prefixes: `FOO=1 corbanu vault`
   - subshells and groups: `(corbanu vault list)`, `{ …; }`
   - command substitution: `$(corbanu vault list)`
   - `if`, `for`, and so on

   Nested `sh -c 'corbanu vault'` is also never re-parsed.
   - **Fix:** under strict, also run `codex_shell_command::bash::parse_shell_lc_literal_commands` (make it `pub`). Its comment says it is meant for spotting dangerous literal commands. It skips assignments and recurses into substitutions. Feed every literal command through `unwrap_command` and apply only the forbidden matches.
   - Also treat `[sh|bash|zsh|dash|ksh, -c|-lc, script]` as a wrapper by re-parsing `script`.

8. **Low — `exec_policy.rs:879`: the wrapper list will never be complete.** Commands not on the list still run the program: `/usr/bin/time`, `nice`, `timeout 5`, `stdbuf`, `setsid`, `xargs`, `sudo`/`doas`, `caffeinate`, `arch -arm64`, `builtin exec`, `chroot`, and others.
   - **Fix:** under strict, test every argv suffix `cmd[i..]` against forbidden rules only, plus the `-S` and `sh -c` splitting. The cost is over-matching, e.g. `echo corbanu vault` gets refused. That seems acceptable for Aggressive, and the refusal reason explains it.
   - Otherwise, document the gap.

9. **Low (existing gap, not widened by this change) — `exec_policy.rs:53-130`, `:445-460`: a wrapper-only allow rule can run a command outside the sandbox without a prompt.**
   - **Mechanism:** an allow rule on `["nohup"]`, `["command"]`, `["exec"]` or `["env", "-i"]`, approved from a suggestion or written by the user, makes `nohup sh -c 'corbanu vault list'` Skip with `bypass_sandbox: true`.
   - **Why it matters:** this runs outside the sandbox with no prompt, which defeats the main control.
   - **Gap in the ban list:** it uses exact matching and only contains `["env"]`.
   - **Fix:** ban wrapper prefixes of any length when suggesting rules. Under strict, never set `bypass_sandbox` when the first word is a wrapper or shell. Optionally, have the Aggressive check flag such user allow rules.

10. **Low — `exec_policy.rs:966`: unbounded recursion on `-S`.** `unwrap_env_args` recurses once per `-S`. `env -S -S -S … corbanu` costs about 3 bytes per level, so an argv near `ARG_MAX` can overflow the stack and abort the session.
    - **Fix:** turn the recursion into a loop over a `Vec`.

11. **Low — `config/src/config_toml.rs:368-371`, `core/config.schema.json`: docs not updated.** The `strict_rules` doc only covers fatal parse errors, but the key now also changes how commands are matched.
    - **Fix:** update the doc comment and regenerate the schema. Gating on `strict_rules` is the right scope, since it is only refusal, Aggressive-only, and inherited by sub-agents. A setting name like `strict_exec_policy` would describe both behaviours better.

12. **Low — `core/src/exec_policy_tests.rs:2566-2572`: the "never widen an allow" test checks nothing.** `env git status` could never be Skip in either mode.
    - **Fix:** add `prefix_rule(["env"], decision="allow")` and assert that `env corbanu vault list` is Forbidden under strict and Skip with `bypass_sandbox` in lenient mode.
    - Add regression cases for findings 1–5: nine-deep nesting, `--split`/`--chd`, `\_`, `exec -ca`, `-L`.

13. **Nit — `exec_policy.rs:881-882`: relative-path handling depends on the session's working directory.** As the comment says, relative paths are left to host-executable resolution. That path calls `AbsolutePathBuf::try_from`, which resolves against the Corbanu process's working directory, not the command's `workdir`.
    - This still works today, because the lookup is lexical and only the basename is compared: `././corbanu`, `bin/../corbanu` and `corbanu.exe` on Windows all resolve.
    - It breaks if a `host_executable` list exists, which Aggressive verification already flags, or if `current_dir()` fails because the directory was removed.
    - **Fix:** in strict mode, reduce the unwrapped program to `executable_lookup_key(file_name)` explicitly. Your summary describes this behaviour, but the code doesn't do it.

## Gaps to state even after the fixes

Name matching can't stop these; the sandbox and the unreadable vault store remain the real control:
- symlinks, hard links or copies of the binary (`ln -s "$(command -v corbanu)" /tmp/x; /tmp/x vault`)
- shell aliases and functions
- running the program through `python`/`node`/`perl` `exec`
- variables (`$P vault`)
- `eval`