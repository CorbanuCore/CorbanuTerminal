# Command rules

Command rules let you ask before selected commands run, refuse them, or remember
an approval. They work alongside the session's permissions and sandbox.

Product basis: [Shipping MVP — LIVE](corbanu-product-spec.md#shipping-mvp-live),
**Workspaces**: “approvals, existing general sandboxing”. This page describes
the upstream-sync branch at `76ce4f9cb6`, not the 0.1.49 main release.
Main merge #11 (#2582) is still pending at this baseline; the differences below
must be rechecked when it lands.

## Write a rule

Put a `.rules` file in the `rules/` directory of your active Corbanu home, then
start a new session. Remembered command approvals use `rules/default.rules`.
Trusted configuration layers can supply rules too; administrator requirements
may restrict which rules load.

```starlark
prefix_rule(
    pattern = ["git", "push"],
    decision = "prompt",
    justification = "Review the destination before publishing commits.",
    match = ["git push origin main"],
    not_match = ["git status"],
)
```

A pattern matches the command's first words, in order. Extra words are allowed:
this example matches both `git push` and `git push origin main`. Use a list of
alternatives for one word, such as `["git", ["push", "fetch"]]`.

- `prompt` requests approval through the configured approval path.
- `forbidden` refuses the command.
- `allow` supplies a command-rule approval; other safety checks still apply.

The strictest matching decision wins. In `.rules` files the order is `forbidden`,
then `prompt`, then `allow`. Managed `requirements.toml` rules can also use
`prompt-user`, which requires a fresh human decision and ranks between
`forbidden` and `prompt`; it is not a valid `.rules` decision.
`match` and `not_match` are examples checked when the file loads. They check the
rule's prefix, not the whole runtime safety decision.

## Wrappers, shells, and quoted words

Rules check the outer command and the commands inside recognized launchers,
such as `sudo`, `env`, `chroot`, `nice`, and `timeout`. An outer allow rule cannot
cancel an inner prompt or refusal. Unknown launcher options can require approval.

For example, a prompt rule for `git push` also applies to:

```text
sudo env MODE=test git push
bash -lc "'git' push"
bash -lc "g'i't push"
```

Literal quotes are removed when reading words, including quoted program paths.
This sync baseline reads scripts behind launchers and partially readable scripts,
but does not yet have main's general eight-level nested-script checking (#2531).
Do not assume `bash -lc "bash -c 'git push'"` receives the same inner-rule check
as on main. A saved approval for an outer shell is not a substitute for checks
on the commands it runs.

## When a POSIX shell command cannot be determined

For POSIX shells such as bash, rules also inspect commands in scripts with
redirections, substitutions, loops, and functions. This partial reading can add a prompt or refusal; it cannot use
an allow rule to approve the complex script.

If a command in that POSIX script cannot be determined and a prompt or forbidden
rule could match, Corbanu asks (with the PowerShell exception below). For example, with the `git push` prompt rule above:

```text
bash -c '$x push'
bash -c 'git "$action"'
bash -c 'env X="$PWD/y" cmd'
```

The last example can ask even though the visible program is `cmd`: a run-time
word in a launcher's arguments prevents the reader from reliably identifying
the inner command. If the known words already match a forbidden rule, Corbanu
refuses outright. If there is no prompt or forbidden rule that could match,
this extra check adds no prompt; ordinary safety checks still apply.

With approvals disabled (`approval_policy = "never"`), a command that needs
approval is refused. Disabling prompts does not turn an unreadable command into
an approved one.

## PowerShell on this sync branch

Sync parses supported literal PowerShell commands in-process, without starting
the executable named by the proposed command. Main's trusted external parser
selection is not used here. The reader recognizes `-Command`/`-c` and a limited
set of preceding flags, but currently reads only the first script argument.
Do not rely on it to check all arguments after `-Command`, `-EncodedCommand`,
or file contents as described for main in #2568/#2580. These are branch
differences, not guarantees carried by this documentation port.

## Case and executable paths

Prefix rules compare case exactly by default on this sync baseline. Main's
additional macOS/Windows prompt-and-forbidden program-name folding (D692) is
still pending in main merge #11. Launcher-name case folding (#2503/D688) is also
missing here: `SUDO git push` does not receive the inner `git push` rule check.
The built-in dangerous-command check already recognizes platform-equivalent
spellings; that is separate from rule matching. Windows basename lookup also retains its existing name
normalization for restrictive rules.

Sync supports explicit `canonical_matching=True` on a rule, which compares all
prefix words without regard to case using Unicode canonical matching. This is
opt-in and can affect allow rules and arguments too. See the
[policy reference](https://github.com/CorbanuCore/CorbanuTerminal/blob/sync/upstream-rust-v0.167.0/codex-rs/execpolicy/README.md#canonical-matching).

Ordinary allow rules require the exact program spelling on every platform. On Windows,
an allow rule for `git` does not also approve `GIT` or `git.exe`. Failure to match
an allow rule leaves the command to the remaining rules and safety checks; it
does not itself guarantee a prompt.

Absolute executable paths can match rules for their basename. With no
`host_executable` entry, an allow rule needs the exact filename: for example,
`C:\tools\git` can match `git`, while `C:\tools\git.exe` cannot. To approve a
specific installed `.exe` by its basename rule, list it explicitly:

```starlark
host_executable(name = "git", paths = ["C:\\tools\\git.exe"])
prefix_rule(pattern = ["git", "status"], decision = "allow")
```

An allow match then requires the listed path's spelling, including the drive
letter's case. **A host-executable list also limits prompt and forbidden
basename rules to listed paths.** It is not a way to prohibit every other copy
of that program. See the [policy reference](https://github.com/CorbanuCore/CorbanuTerminal/blob/sync/upstream-rust-v0.167.0/codex-rs/execpolicy/README.md) for
exact-match precedence and path resolution.

## xargs

`xargs` is checked both as a launcher and for the command it runs. With a
forbidden rule for `git push`:

| Invocation           | Command-rule result                         |
| -------------------- | ------------------------------------------- |
| `xargs -n1 git push` | Refused: the fixed words already match.     |
| `xargs git`          | Asks: input could append `push`.            |
| `xargs -I{} git {}`  | Asks: input could replace `{}` with `push`. |

If input could complete a forbidden rule, sync requires a fresh human decision
(`prompt-user`); fixed words already matching that rule are refused. Allow rules
alone do not trigger this extra input-dependent prompt. However,
`xargs sh -c`, `xargs env`, and `xargs command -v` still require approval even
without a restrictive rule. Unknown `xargs` options also require approval.

## Current limits

Command rules are not a complete shell interpreter. Do not rely on them alone
to inspect code reconstructed by `eval`, fed into a shell through a pipe or
here-document, or loaded from a sourced file
([#2548](https://github.com/CorbanuCore/CorbanuTerminal/issues/2548), in progress).
Ordinary pipelines can still have their visible commands checked.

A further known gap is a POSIX script passing a run-time word to PowerShell,
such as `bash -c 'pwsh -Command "$x"'`: this currently asks only when a
restrictive rule matches or could extend the visible `pwsh -Command` prefix,
not merely because another restrictive rule exists
([#2585](https://github.com/CorbanuCore/CorbanuTerminal/issues/2585)).

Use `/permissions` to inspect session permissions. This version has no `/rules`
slash command. For a prefix-rule diagnostic, use:

```bash
corbanu execpolicy check --rules path/to/policy.rules --pretty git push origin main
```

This diagnostic checks the supplied words. It does not reproduce runtime
wrapper expansion, nested-script inspection, or approval and sandbox decisions.
Explicit `canonical_matching=True` rules do apply to this diagnostic. See the [policy reference](https://github.com/CorbanuCore/CorbanuTerminal/blob/sync/upstream-rust-v0.167.0/codex-rs/execpolicy/README.md)
for its JSON output and optional absolute-path resolution.
