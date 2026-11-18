# Command rules

Command rules let you ask before selected commands run, refuse them, or remember
an approval. They work alongside the session's permissions and sandbox.

Product basis: [Shipping MVP — LIVE](corbanu-product-spec.md#shipping-mvp-live),
**Workspaces**: “approvals, existing general sandboxing”. This page describes
Corbanu's command-rule behavior for 0.1.49.

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

- `prompt` asks for approval.
- `forbidden` refuses the command.
- `allow` supplies a command-rule approval; other safety checks still apply.

The strictest matching decision wins: `forbidden`, then `prompt`, then `allow`.
`match` and `not_match` are examples checked when the file loads. They check the
rule's prefix, not the whole runtime safety decision.

## Wrappers, shells, and quoted words

Rules check the outer command and the commands inside recognized launchers,
such as `sudo`, `env`, `chroot`, `nice`, and `timeout`. An outer allow rule cannot
cancel an inner prompt or refusal. Unknown launcher options can require approval.

For example, a prompt rule for `git push` also applies to:

```text
sudo env MODE=test git push
bash -lc "bash -c 'git push'"
bash -lc "'git' push"
bash -lc "g'i't push"
```

Literal quotes are removed when reading words, including quoted program paths.
Rules read up to eight nested scripts below each command they inspect; deeper
nesting requires approval. Launcher and shell checks may still find a forbidden
command beyond that limit. A saved approval for an outer shell does not stand
in for approving its inner commands.

## When the command cannot be determined

Rules also inspect commands in scripts with redirections, substitutions, loops,
and functions. This partial reading can add a prompt or refusal; it cannot use
an allow rule to approve the complex script.

If a command cannot be determined and a prompt or forbidden rule could match,
Corbanu asks. For example, with the `git push` prompt rule above:

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

## PowerShell on Windows

Corbanu reads scripts using a separately selected PowerShell parser. It never
starts the executable named by the proposed command just to parse its script.
It prefers the matching system installation under
`%SystemRoot%\System32\WindowsPowerShell\v1.0` or
`%ProgramFiles%\PowerShell\7`, then searches absolute `PATH` entries outside the
current folder and its descendants. That exclusion also covers alternate case,
8.3 short paths, junctions, and links back into the folder. Network or device
paths supplied as system-installation roots are rejected.

All arguments after `-Command` are joined with spaces, as PowerShell joins them:

```text
powershell.exe -Command "git status" "; git push"
```

Both commands are checked. Supported flags include `-NoProfile`, `-NoLogo`,
`-NonInteractive`, `-STA`, and `-MTA`; recognized options with values, such as
`-ExecutionPolicy`, are consumed before the script. Accepted abbreviations and
spellings include `-c`, `-nop`, `-ec`, `/Command`, and `--command`.
`-EncodedCommand` is decoded from Base64 UTF-16LE and checked as a script.

`-File` checks the file path and arguments, **not the file's contents**. It also
asks whenever any prompt or forbidden rule exists. The same conservative
approval applies if no trusted parser can read a script, or the invocation is
unsupported or ambiguous: for example `-NoExit`, script input from stdin,
invalid encoded text, or `-CommandWithArgs` with arguments. Positional scripts
are not inspected as file contents either.

## Case and executable paths

On macOS and Windows, prompt and forbidden rules match program names without
regard to letter case: a rule for `git push` covers `GIT push` too. Arguments
remain exact: `push` and `PUSH` differ. Linux program names remain case-sensitive.
Launcher recognition also accepts platform-equivalent spellings.

Allow rules require the exact program spelling on every platform. On Windows,
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
of that program. See the [policy reference](../codex-rs/execpolicy/README.md) for
exact-match precedence and path resolution.

## xargs

`xargs` is checked both as a launcher and for the command it runs. With a
forbidden rule for `git push`:

| Invocation           | Command-rule result                         |
| -------------------- | ------------------------------------------- |
| `xargs -n1 git push` | Refused: the fixed words already match.     |
| `xargs git`          | Asks: input could append `push`.            |
| `xargs -I{} git {}`  | Asks: input could replace `{}` with `push`. |

Allow rules alone do not trigger this extra input-dependent prompt. However,
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
wrapper expansion, nested-script inspection, platform case folding, or approval
and sandbox decisions. See the [policy reference](../codex-rs/execpolicy/README.md)
for its JSON output and optional absolute-path resolution.
