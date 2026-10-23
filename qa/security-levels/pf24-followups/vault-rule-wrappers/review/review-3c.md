**Verdict: approve with fixes.** Fix #1 (a regression) and #2 before merge. The rest can be follow-ups. I didn't edit any files or run any tests.

The new design is sound. Strict matching can only set `Decision::Forbidden` and add forbidden matches, so it can't widen an allow. The amendment is derived after strict runs and ignored when the result is Forbidden, and the zsh-fork path only turns into `Deny`. Scanning the original command fixes the heredoc gap, and dedup works because `RuleMatch` has no per-candidate fields.

### Findings

1. **High: quote and escape concatenation now bypasses strict matching (regression)**, `core/src/exec_policy.rs:957-972`. `loose_tokens` splits on `'`, `"`, `\`, `^` and `` ` ``, but shells join adjacent pieces. So `sh -c "corbanu va''ult list"`, `eval 'corbanu v\ault'`, a heredoc line `cor""banu vault`, and Windows `cor^banu vault` all tokenize as `va`/`ult`. Pass 2 caught the nested-script forms because it used shlex. The lenient matcher only rejoins concatenation in the top-level parse, not in nested scripts or heredocs.
   **Fix:** run a second stream where `'`, `"`, `\`, `^` and `` ` `` are deleted instead of split, then split on the rest, and match the union of both streams. Keeping both preserves the Windows-path and backtick-substitution cases. Also feed the parsed `commands` from `commands_for_exec_policy` into strict matching. Add regression cases: `sh -c "corbanu va''ult"` and a heredoc `c\orbanu vault`.

2. **Medium: the 256-word option skip counts tokens, not arguments**, `exec_policy.rs:914-916`. One quoted option value with many spaces pushes `vault` out of the window: `corbanu -c 'x="a a a … (300×)"' vault list`. Pathological long values give the same result with no adversary.
   **Fix:** drop the cap and change the search instead. For each program occurrence, only try skip offsets where the token equals some forbidden rule's second pattern element (read via `RuleRef::as_any` → `PrefixRule`). That makes the skip unbounded and also cheaper (see #3).

3. **Low/medium: pathological cost blocks an async worker**, `exec_policy.rs:908-931`. Every token naming a program with *any* rule (including allow rules like `git status`) tries up to 256 skips, cloning up to 17 strings and matching each time. Input like `"corbanu -x " × 100k` (about 1 MB, possible in a heredoc) means roughly 25M candidates and 400M string clones, i.e. seconds to tens of seconds of synchronous work inside `create_exec_approval_requirement_for_command`.
   **Fix:**
   - Precompute the set of program keys that have at least one `Forbidden` rule and gate on that, not on `get_vec(..).is_some()`.
   - Use the second-token pruning from #2.
   - Reuse one candidate buffer.
   - Add a test asserting a 1 MB adversarial input finishes within a bound.

4. **Medium: false positives now apply to every user forbidden rule, not just the vault rule**, `exec_policy.rs:888-896`, test `exec_policy_tests.rs:2549`. Under `strict_rules`, any forbidden rule matches prose:
   - `["rm"]` refuses `git commit -m "rm dead code"`.
   - `["git","push"]` refuses `echo "don't git push yet"`.
   - Agents working on Corbanu itself can't run `rg 'corbanu -c' | grep vault` or write vault docs through a heredoc.
   
   Pass 2's non-forbidden cases (`git commit`, `grep`) were moved to the forbidden list.
   **Fix:** pick one:
   - Limit loose matching to the managed Aggressive rules, marked by file or flag (preferred).
   - Require a rule of at least 2 tokens.
   
   Either way, make the refusal text say a strict text match fired and suggest `apply_patch` for data, so the agent stops trying quote variants.

5. **Low: rule keys in mixed case still miss other casings**, `exec_policy.rs:975-1000`. A token's keys are only "as written" and "lowercased", so a user rule keyed `Corbanu` misses `CORBANU`. That's the same binary on default case-insensitive APFS.
   **Fix:** build a lowercase → original-keys index once per call and look up `name.to_ascii_lowercase()` in it.

6. **Low: the docstring understates what's out of reach**, `exec_policy.rs:893-895`. Besides names built at run time, these also escape:
   - arguments supplied at run time (`V=vault; corbanu $V`, `corbanu "$@"`, `printf vault | xargs corbanu`, `find -exec corbanu {}`);
   - parameter expansion (`corbanu ${x:-vault}`);
   - ANSI-C escapes (`$'\x76ault'`).
   
   **Fix:** say "names or arguments built at run time" and list these examples. Also note that the zsh-fork execve check is what catches a real `corbanu` exec reached through a shell.

7. **Low: the "only adds refusals" test is weak**, `exec_policy_tests.rs:2641-2648`. It compares only commands where strict matching finds nothing, so it passes no matter what happens on a match.
   **Fix:** also compare cases where lenient already returns Prompt or Forbidden from a rule, and assert strict is never less restrictive. Ideally, loop over every case in `agent_shaped` and the benign list, checking strict ≥ lenient.

8. **Low: missing negative tests for the known residual bypasses.** Add explicit "known not caught" assertions (or `#[ignore]`d cases) for #1, #2 and #6, so a future change can't quietly turn a documented gap into an untracked one.

9. **Nit: `HEAD` isn't rustfmt-clean.** `loose_tokens` (`exec_policy.rs:958-965`) and the test closures are misformatted in the commit. The formatted version exists only as uncommitted changes in the working tree. Commit the `just fmt` output before running the final tests and TUI check.

10. **Nit: the option-cluster rule is broad**, `exec_policy.rs:978-984`. Any single-dash token containing `S` returns its tail, so `-XSfoo` yields the key `foo`. That's harmless for refusing, but a comment should say it's intentionally loose, or the check should be limited to tokens right after an `env` token.

**Not re-raised:** a `corbanu exec` started from inside an agent command (finding 6) still needs a product-owner decision on Aggressive. Over-matching (finding 9) is now documented, though #4 argues for narrowing its scope.