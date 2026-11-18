# codex-execpolicy

## Overview

- Policy engine and CLI built around `prefix_rule(pattern=[...], decision?, justification?, match?, not_match?)` plus `host_executable(name=..., paths=[...])`.
- This release covers the prefix-rule subset of the execpolicy language plus host executable metadata; a richer language will follow.
- Tokens are matched in order; any `pattern` element may be a list to denote alternatives. `decision` defaults to `allow`; valid values: `allow`, `prompt`, `forbidden`.
- `justification` is an optional human-readable rationale for why a rule exists. It can be provided for any `decision` and may be surfaced in different contexts (for example, in approval prompts or rejection messages). When `decision = "forbidden"` is used, include a recommended alternative in the `justification`, when appropriate (e.g., ``"Use `jj` instead of `git`."``).
- `match` / `not_match` supply example invocations that are validated at load time (think of them as unit tests); examples can be token arrays or strings (strings are tokenized with `shlex`).
- The CLI always prints the JSON serialization of the evaluation result.

## Command rules in Corbanu Terminal

For rule files, examples, wrapper and nested-shell checks, unreadable commands,
Windows PowerShell, case matching, xargs, and known limits, see the
[Corbanu command-rule guide](../../docs/execpolicy.md).

The runtime checks recognized wrapper layers and readable inner commands, using
the strictest result. Unreadable commands ask when a prompt or forbidden rule
could match; approvals disabled means refusal. Nested scripts have an
eight-level inspection bound. These checks have documented limits, including
shell-fed code (#2548) and run-time PowerShell words inside POSIX scripts (#2585).

The `execpolicy check` CLI below is a **prefix-rule diagnostic**. It does not run
the runtime's wrapper/script checks or platform case folding, and its result is
not an execution approval.

## Policy shapes

- Prefix rules use Starlark syntax:

```starlark
prefix_rule(
    pattern = ["cmd", ["alt1", "alt2"]], # ordered tokens; list entries denote alternatives
    decision = "prompt",                 # allow | prompt | forbidden; defaults to allow
    justification = "explain why this rule exists",
    match = [["cmd", "alt1"], "cmd alt2"],           # examples that must match this rule
    not_match = [["cmd", "oops"], "cmd alt3"],       # examples that must not match this rule
)
```

- Host executable metadata can optionally constrain which absolute paths may
  resolve through basename rules:

```starlark
host_executable(
    name = "git",
    paths = [
        "/opt/homebrew/bin/git",
        "/usr/bin/git",
    ],
)
```

- Matching semantics:
  - execpolicy always tries exact first-token matches first.
  - With host-executable resolution disabled, `/usr/bin/git status` only matches a rule whose first token is `/usr/bin/git`.
  - With host-executable resolution enabled, if no exact rule matches, execpolicy may fall back from `/usr/bin/git` to basename rules for `git`.
  - If `host_executable(name="git", ...)` exists, basename fallback is only allowed for listed absolute paths.
  - If no `host_executable()` entry exists for a basename, basename fallback is allowed.
  - Allow rules require exact program spelling on every platform. On Windows,
    basename lookup can recognize `git.exe` as `git` for restrictive rules, but
    an allow rule needs either an exactly spelled filename or an explicitly
    listed host path with the same spelling, including drive-letter case.
  - A host-executable list constrains prompt and forbidden basename matches too;
    it does not prohibit unlisted executables.
  - At runtime on macOS and Windows, prompt and forbidden rules additionally
    match differently cased program names. Allow rules and arguments remain
    exact. The CLI does not enable this extra case-folding check.

## CLI

- From the Codex CLI, run `codex execpolicy check` subcommand with one or more policy files (for example `src/default.rules`) to check a command:

```bash
codex execpolicy check --rules path/to/policy.rules git status
```

- To opt into basename fallback for absolute program paths, pass `--resolve-host-executables`:

```bash
codex execpolicy check \
  --rules path/to/policy.rules \
  --resolve-host-executables \
  /usr/bin/git status
```

- Pass multiple `--rules` flags to merge rules, evaluated in the order provided, and use `--pretty` for formatted JSON.
- You can also run the standalone dev binary directly during development:

```bash
cargo run -p codex-execpolicy -- check --rules path/to/policy.rules git status
```

- Example outcomes:
  - Match: `{"matchedRules":[{...}],"decision":"allow"}`
  - No match: `{"matchedRules":[]}`

## Response shape

```json
{
  "matchedRules": [
    {
      "prefixRuleMatch": {
        "matchedPrefix": ["<token>", "..."],
        "decision": "allow|prompt|forbidden",
        "resolvedProgram": "/absolute/path/to/program",
        "justification": "..."
      }
    }
  ],
  "decision": "allow|prompt|forbidden"
}
```

- When no rules match, `matchedRules` is an empty array and `decision` is omitted.
- `matchedRules` lists every rule whose prefix matched the command; `matchedPrefix` is the exact prefix that matched.
- `resolvedProgram` is omitted unless an absolute executable path matched via basename fallback.
- The effective `decision` is the strictest severity across all matches (`forbidden` > `prompt` > `allow`).

Note: `execpolicy` commands are still in preview. The API may have breaking changes in the future.
