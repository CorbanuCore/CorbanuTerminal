I merged two of the three PRs and closed four of the five issues with evidence comments. The third PR (#533, for #529) is not merged: it changes Permissive for configs that already have their own denials, so it needs your sign-off. All its checks pass and its review approves.

## What shipped
- **#532, merged** (`75fe515329`): closes #501 and #502 on Linux.
  - **#502:** a denial spelled through a link the command can't change is now masked where the link leads. Commands start again, and the file stays denied.
  - **#501:** under restricted reads, a denial now also hides the file through a read-only folder reached by a link. That includes a denial on the link's target or a folder above it.
- **#531, merged** (`ae3dccf1ec`): closes #518 and #519.
  - **#518:** a read-only entry no longer makes a denied file readable through a linked home. While testing, I found and fixed the same flaw in the upload check that #517 added.
  - **#519:** the upload read now opens the file once, checks where the kernel says the opened file is, and reads from that same handle. It also applies pattern (glob) denials, and denials relative to a working folder reached through a link.
  - The #519 race turned out not to be reachable today: once the protected rules apply, their denials always make the sandbox run the read. The fix hardens the remaining unsandboxed branch, and a test pins it.

## Decision needed: #533 (#529, macOS)
Each writable root now also blocks writes to read-only paths that other roots protect. Examples are a writable link to `.git`, or `/tmp` holding the workspace. This matches Linux since #511.
- **Shipped Permissive:** unchanged, because the fix only applies when the policy denies reads. A test pins this.
- **Permissive configs with user-defined denials:** read-only folders like `.git` now also stay read-only through other writable roots.
- **Options:**
  - **(A) Merge as is (my recommendation).** It only affects configs that already use denials, and it matches Linux.
  - **(B) Pass an explicit security-level flag into the sandbox** so every Permissive config stays identical. This is a larger change.

## Gate evidence
| | Before (main code) | After | Review |
|---|---|---|---|
| #531 | 5 regression tests fail; real `sandbox-exec` printed the dummy SSH key, and the swapped-link upload sent the dummy secret | core `security::` 304/304 on macOS and 305/305 on Linux, each with `TMPDIR=/tmp` and the default | Approved in rounds 1–3 |
| #532 | Real bubblewrap printed the denied file through the linked folder; every command failed with "Can't create file" | 111 unit + 35 integration tests; 115 + 37 after #521 merged | Changes requested, then approved |
| #533 | Real `sandbox-exec`: `.git` and a read-only folder were writable through the links | 95/95 with `TMPDIR=/tmp` and the macOS default; all CI checks pass | Changes requested twice, then approved |

- **Clippy `-D warnings`:** clean on macOS and Linux for all three.
- **CI:** every check was green with none pending at each merge. One unrelated flaky test (`security_revocation_one_grant_ends_only_that_grant`, shared global state) failed once in CI and passed on the next run.
- **Permissive:** unchanged by #531, because its rules only apply under Moderate or Aggressive. For #532, a policy without denials produces the same sandbox setup. With denials it only adds masks of already-denied files, and commands that #502 broke now start.

## Open
- **Filed #535 (needs your decision):** a writable link into another root's read-only folder, e.g. `.git/hooks`, stays writable on both macOS and Linux.
- **Filed #536 (existed before):** a configured writable folder that doesn't exist yet can be created as a link to somewhere else.
- **Overlaps:** #526 and #521 merged into the same files while I worked. I resolved the conflicts and retested on both machines.
- **Pre-existing local failures:** two seatbelt argument tests fail under this Mac's custom TMPDIR but pass with `/tmp` and the default. `pf_23_s01_code_mode_cell_results_taint_its_later_calls` also fails on unmodified main.
- **Not done:** no tmux run or demo videos; the brief didn't list them.
- **Cleanup:** I removed the merged worktrees, all Mac build folders, and the glitch clone and build folder (39 GB). I kept the #533 worktree because that PR is open. The local branches for the two merged PRs remain, since deleting them would need a force-delete.

Everything is in `.codex-work/workers-20261002/`:
- `sec-links.result.md`
- `sec-links-logs/`
- `sec-links-review/`

On glitch, the logs are in `~/corbanu-rtx/seclinks-logs/`.