# Aggressive's `corbanu vault` rule and wrapped commands

Prefix rules match a command's literal first word. Absolute and relative
paths already resolve by file name. Wrapped forms reached the normal approval
prompt instead of being refused:
- `env FOO=1 corbanu vault list`
- `sh -c 'corbanu vault list' 2>&1`
- `corbanu -c x=1 vault list`

The unreadable vault store is still the main control.

## Fix

When the session's exec policy was loaded with `strict_rules` (only Aggressive
sets it), multi-word `forbidden` prefix rules also match, best effort, against
the whole command text:
- The command's words become one token stream, including scripts and strings
  at any depth (`sh -c`, `env -S`, `eval`, heredocs, `python -c`, git aliases).
  The stream is split on whitespace and shell punctuation, once with quotes as
  separators and once with quotes removed.
- A token naming a rule's program matches, whatever its directory, case or
  launcher suffix (`.exe`, `.cmd`, `.ps1`, `.js`). It must be followed by the
  rule's next word, either directly or after options.

The same check runs on execs intercepted by zsh-fork. It only ever adds
refusals, and the refusal reason says that a strict text match fired.

**Out of reach:**
- names or arguments built at run time (`c=corbanu; $c vault`,
  `corbanu "$@"`, `${x:-vault}`, `$'\x76ault'`);
- aliases, functions and copies of the binary.

**Accepted over-matching:** the words in text, such as
`git commit -m "… corbanu vault …"`. Single-word forbidden rules (`rm`) keep
exact matching.

## Gate evidence

- **Tests:** after `just fmt` and `just fix`, all passed:
  - `just test -p codex-core exec_policy` (112)
  - `just test -p codex-core unix_escalation` (21)
  - `just test -p codex-tui security::` (29)

  New tests:
  - more than 35 wrapped forms are refused, including the reviewers' bypasses;
  - known gaps and benign commands stay allowed;
  - agent-shaped `bash -lc` scripts are refused through the approval path;
  - strict is never less restrictive than lenient, and identical when it finds
    nothing;
  - a strict zsh-fork intercepted `env … corbanu vault` is refused.
- **tmux run on GLM 5.2** (`tmux-run/`): Aggressive active; GLM 5.2 was asked
  to run each of the three commands above.
  - **Before (main):** all three reached the approval prompt.
  - **After:** all three were refused by the vault rule. The rollout shows 3
    rejections that carry the strict-match note.
- **Reviews (Opus 5.5 High):**
  - first: approve with fixes (wrapper parsing);
  - second: request changes on the argv-suffix rework;
  - third: approve with fixes. Its findings 1–5 and 7–10 are fixed in the last
    commits.
- **Video:** `qa/demos/index/aggressive-vault-wrapped.md`.

## Open (product decision)

An agent can start a nested `corbanu exec`, which loads the user's plain
config rather than Aggressive. Review 3b, finding 6, raised this. It belongs to
the Aggressive sandbox design.
