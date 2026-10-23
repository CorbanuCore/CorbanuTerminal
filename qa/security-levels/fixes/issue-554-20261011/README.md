# #554: Aggressive follow-ups from #521 and #522 (2026-10-11)

These are the limits that #521 (the Corbanu home can't be listed) and #522 (the kill switch stops running work) left open. None of them needs an owner decision.

| Item | Result |
| --- | --- |
| `find ~` or `ls -aR ~` | The home is denied and now explained the way #521 explains a direct listing. |
| Windows | Listing the home is still not denied there (see below), so the approval prompt no longer says it is. |
| A Claude turn stopped by the kill switch | It ends with "Turn stopped by the security kill switch." (as in Codex panes) instead of "interrupted by user". |
| Claude Code older than 2.1.292 in a contained pane | The pane says so as soon as it opens, and again at every task. The message names the `claude` on PATH, its version or the `--version` error, and how to fix it. |

## Live check (tmux, GLM 5.3 Flash)

Setup:
- A private tmux socket and a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
- The "Aggressive ships" feature flags, with contained Claude panes on.
- The real Z.AI key and `claude-plan-test-token` from the vault helper, passed on the launch command only.
- A disposable `CLAUDE_CONFIG_DIR`.
- Each run confirms Aggressive in `/security` first, with no restart.

Scripts: [launch.sh](scripts/launch.sh), [case554find.sh](scripts/case554find.sh), [case554claudekill.sh](scripts/case554claudekill.sh) (Claude Code 2.1.296 installed in a scratch folder), and [case554claudeold.sh](scripts/case554claudeold.sh) (Claude Code 2.1.92 from Homebrew first on PATH).

Builds (macOS arm64, debug):
- **Before:** the product code of `origin/main` `eb92efdf82`.
- **After:** this branch at `9cf94e9fdb`.

| Step | Before | After |
| --- | --- | --- |
| `find <folder holding the home> -maxdepth 2 -name config.toml` | `Operation not permitted` on the home with no explanation; the model guesses ([capture](captures/before-find-above.txt)) | the same denial plus "Security level Aggressive blocked listing `…/home` …" ([capture](captures/after-find-above.txt)) |
| `ls -aR <that folder> > /dev/null` | denied, no explanation ([capture](captures/before-ls-recursive-above.txt)) | denied and explained ([capture](captures/after-ls-recursive-above.txt)) |
| Kill switch on while a contained Claude pane writes | the stop message, then "Claude pane turn interrupted. Claude pane turn interrupted by user." ([capture](captures/before-claude-kill-switch.txt)) | the stop message, then "Turn stopped by the security kill switch." ([capture](captures/after-claude-kill-switch.txt)) |
| Creating a Claude pane with Claude Code 2.1.92 on PATH | nothing ([capture](captures/before-old-claude-pane-created.txt)) | "This Claude pane cannot run yet. …" names the `claude` on PATH, 2.1.92, the 2.1.292 minimum and how to update ([capture](captures/after-old-claude-pane-created.txt)) |
| Its first task | "contained Claude panes need Claude Code 2.1.292 or later; found 2.1.92. Update Claude Code and try again." ([capture](captures/before-old-claude-turn.txt)) | the same explanation as at creation ([capture](captures/after-old-claude-turn.txt)) |

## How the parent listing is explained

The explanation is added only for a command that failed under a live Aggressive level. It never changes what runs. Each denial line that mentions the home's folder name is read for a path in one of these forms:
- BSD: `find: /x: Operation not permitted`
- GNU: `find: ‘/x’: Permission denied`
- GNU `ls`: `ls: cannot open directory '/x'`

A relative path is resolved against the command's folder, or against a folder the command names, as in `cd ~ && find .`. The path must be a folder that can't be listed and isn't itself a protected read.

## Windows

Under Aggressive on Windows, agent commands can still list the Corbanu home, meaning the names of its entries. Every protected entry in it stays unreadable through the deny-read ACLs (sign-in, vault, config, history and so on).

Enforcing the listing denial isn't done here. It would need a deny ACE on the home folder for `FILE_LIST_DIRECTORY` only, without inheritance, for the sandbox identities. The Windows deny-read machinery only places inherited full-read denies. It keys them by rule in the persistent per-session state (#304) and syncs them through the elevated setup helper. A listing-only rule would mean a new rule kind in:
- `DenyReadTargets`
- the helper's payload, which must stay compatible with older helpers
- the session state and its removal

It could only be checked on CI's Windows hosts. The `:directory_listing` entry stays ignored on Windows, and the approval prompt there no longer claims that listing the home is blocked.

## Tests (fail with the fix disabled, pass after)

- **Core:**
  - `security::protected_reads::tests::issue_554_recursive_listing_of_a_parent_is_attributed_to_the_home`
  - `issue_554_find_above_the_home_is_explained_under_aggressive`
  - the macOS Seatbelt end-to-end tests `suite::issue_506::*`, which now also cover `find` and `ls -aR` of the folder above the home and the Permissive and Moderate controls
- **TUI:**
  - `app::tests::issue_554_kill_switch_stop_does_not_say_interrupted_by_user` (a person's own interrupt still says "interrupted by user")
  - `claude_panes::tests::approval_turn::issue_554_old_or_broken_claude_code_is_explained`
  - `issue_554_uncontained_panes_have_no_launch_problem`
