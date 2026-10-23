# sec-links (#518, #519, #529, #501, #502): result

## Shipped
- **#531 merged** (`ae3dccf1ec`); fixes #518 and #519, both closed with evidence.
  - Read-only entries that are denied in any spelling become denials.
  - The link-resolved copy of the policy no longer reopens a denial through an allow entry.
  - Dangling links are resolved by the system, one hop at a time.
  - A home alias that links to the active home is handled.
  - Apps upload reads open the file once and check where the kernel says it is (`F_GETPATH`, `/proc/self/fd`).
  - The upload check now also applies glob denials and denials relative to a linked working folder.
- **#532 merged** (`75fe515329`); fixes #501 and #502, both closed with evidence.
  - **Linux bubblewrap:** a denial spelled through a read-only link is masked at the link's target.
  - Under restricted reads, denials are also masked at the link-mounted path, including a denial on an ancestor of the target.
  - Nested masks inside a folder that is already masked whole are dropped.
- **#533 open, needs security sign-off** (#529, macOS Seatbelt).
  - Each writable root also excludes the protected paths of other roots it reaches, gated on the policy having denials.
  - Default Permissive output is unchanged. A Permissive config with user-defined denials changes, so it's not merged.
  - Options in the PR: (A) merge as is (recommended), or (B) thread an explicit protected-level flag into the sandbox.
- **Filed:**
  - #535: a writable link into another root's `.git/hooks`. Needs an owner decision; affects macOS and Linux.
  - #536: a missing writable root can be created as a link elsewhere. Existed before.

## Evidence
- Logs are in `sec-links-logs/` (Mac) and `glitch:~/corbanu-rtx/seclinks-logs/`. Reviews are in `sec-links-review/`.
- **#531**
  - **Before:** 5 regression tests fail on main code, including a real `sandbox-exec` run printing the dummy key and the swapped-link upload sending the dummy secret.
  - **After:** core `security::` 304/304 on macOS and 305/305 on Linux, each with `TMPDIR=/tmp` and the default; `mcp_openai_file` 8/8; clippy `-D warnings` clean on both.
  - **Review:** APPROVE in rounds 1, 2 and 3.
- **#532**
  - **Before (real bwrap):** the denied file printed through the linked root; every command failed with `Can't create file`.
  - **After:** 111 unit + 35 integration tests pass, and 115 + 37 after #521 merged; clippy clean on Linux and macOS.
  - **Review:** REQUEST_CHANGES (ancestor case), then APPROVE.
- **#533**
  - **Before (real `sandbox-exec`):** `.git/HEAD` and the read-only folder were writable through the links; the round-2 test showed a missing `target` couldn't be created.
  - **After:** 95/95 with `TMPDIR=/tmp` and the macOS default; clippy clean on macOS and Linux; all 29 CI checks pass.
  - **Review:** REQUEST_CHANGES twice, then APPROVE in round 3.
- **Permissive:**
  - #531: the protected rules only apply under a protected level.
  - #532: no denials gives identical arguments; with denials, only masks of already-denied paths are added, plus #502's fix that commands now start.
  - #533: see the sign-off note above.

## Notes
- **Pre-existing local failures** (also on unmodified main):
  - two `create_seatbelt_args_*` tests fail under this Mac's `.codex-work` TMPDIR; they pass with `/tmp` and the default;
  - `pf_23_s01_code_mode_cell_results_taint_its_later_calls`.
- **CI flake:** `security_revocation_one_grant_ends_only_that_grant` (global grant state) failed once; it passed on the next run.
- **Not done:** no tmux run or demo videos; the brief didn't list them.
- **Cleanup:**
  - Removed: the A and B worktrees, all three Mac targets, and the glitch clone and target (39 GB).
  - Kept: the C worktree, since #533 is open.
  - Local branches `sec-links-{a,b}` remain, because deleting them needs a force-delete.
