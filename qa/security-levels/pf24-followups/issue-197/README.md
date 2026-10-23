# #197: a rules file broken after launch drops the vault rule

Under Aggressive, a `.rules` file that stopped parsing after launch made every
later thread (`/new`, resume, fork, children with their own config) fall back
to requirements-only rules with a warning. Those threads had no
`corbanu vault` rule while `/status` still said Aggressive.

## Fix

- New core setting `strict_rules` (default `false`). When on, a `.rules` parse
  error is fatal for the session being started, and the message gives the file,
  the line and what to do.
- Aggressive sets it through its `-c` overrides, which reach the embedded app
  server. Custom roles may not set it, and the launch check requires it.
- A child that wants strict rules no longer reuses a parent policy loaded
  leniently.
- `thread/start` skips the "custom rules not applied" warning when strict, since
  the thread does not start at all.
- From the review: choosing Permissive in `/security` no longer deletes the
  vault rule file right away. The session stays Aggressive until restart, and
  the next launch removes the file.

Permissive is unchanged: `strict_rules` is off and the fallback runs as before.

## Gate evidence

- **Tests:** `just test -p codex-core exec_policy` (110),
  `just test -p codex-tui security::` (29),
  `just test -p codex-app-server thread_start_` (45), after `just fmt` and
  `just fix`. The new tests:
  - strict load fails on a broken file, and the lenient load drops the vault rule;
  - a strict child does not reuse a lenient parent's policy;
  - `thread/start` fails after the file breaks, with `-c strict_rules=true`
    overriding a user `strict_rules = false`, and sends no fallback warning;
  - the Aggressive overlay forces the setting, and verification fails without it
    or when a role sets it;
  - saving Permissive keeps the rule file until launch.
- **tmux run on GLM 5.2** (`tmux-run/`): Aggressive was chosen and Corbanu
  restarted. Another process then wrote a broken `mine.rules` file, and `/new`
  was run.
  - **Before (main):** `/new` started a thread with "custom rules not applied",
    and `corbanu vault list` went to the normal approval prompt
    (`before-05-vault-attempt.txt`).
  - **After:** `/new` fails and names the file and line
    (`after-04-after-new.txt`). Once the file was fixed, `/new` worked and
    GLM 5.2's `corbanu vault list` was refused (`after-07-vault-after-fix.txt`).
- **Review:** Opus 5.5 High, approve with fixes; see `review/`.
- **Video:** `qa/demos/index/issue-197.md`.
