All four issues are fixed and merged with every check green, and #587, #583, #582 and #581 are closed with evidence. An independent Opus 5.5 High review approved each PR, and none changes Permissive.

| Issue | PR | Merge commit | Review |
|---|---|---|---|
| #587 (HIGH) | #592 | `d878532cf6` | Round 2 approved |
| #583 | #596 | `398a3332e6` | Round 5 approved |
| #582 | #599 | `64387d3607` | Round 3 approved |
| #581 | #600 | `55380dfc47` | All 3 rounds approved |

**#587: a protected command can no longer move the Corbanu home aside (Linux)**
- **Fix:** the home and every folder above it inside a writable root are now mount points, which can't be renamed or removed. This applies only once the policy denies reads, the same condition the macOS rename guard uses.
- **macOS:** already blocked there; the new test passes on main.
- **On main (glitch):** under Moderate the exact attack worked with full access and with a workspace covering the home.
- **Side effect:** a `rename()` across one of those folders now fails as a cross-device move. `mv` falls back to copying.

**#583: missing home entries can no longer be created**
- **Fix:** the Corbanu home itself is read-only for protected commands. That covers `rules/`, `hooks.json`, `agents/` and any other name, and no grant lifts it.
- **`worktrees/` stays writable,** but only as a real folder, not a link. The reviews caught that my first version broke git in home worktrees; that's fixed and tested.
- **On main:** a protected command created `rules/x.rules` on both macOS and Linux.

**#582: placeholders left by a killed run are removed by the next run (Linux)**
- **Fix:** placeholders are recorded in the existing registry. A later run removes those whose runs have all ended and never fails because of this.
- **On main:** a run killed on its timeout left an empty `home/Library` for good.

**#581: the bundled bubblewrap now starts where AppArmor restricts user namespaces (Ubuntu 24.04)**
- **Fix:** it runs through `aa-exec -p rootlesskit`, only from the digest-verified copy, which is checked again just before it runs. The command never gets a handle to that file.
- **Warning:** with no system bubblewrap, it now says when the sandbox can't start and names the package.
- **On main:** the new test fails on glitch. The existing bundled-copy test now runs there instead of skipping.

**Gate evidence**
- **Regression tests:** each fails on main and passes with the fix. All four were checked on glitch, and #583 on macOS too.
- **Core security CI selection,** with the default temp folder and `TMPDIR=/tmp`: passed on glitch for all four PRs, and on macOS for #587 and #583.
- **`linux-sandbox` and `sandboxing` suites on glitch:** passed.
- **Clippy `-D warnings`:** clean on glitch for all four and on macOS.
- **Combined state:** I also re-checked #600 and #596 merged with the latest main (including #591 from the concurrent links worker) before merging them. All green.
- **CI:** one ubuntu run of `pf_23_s01_typing_a_protected_command…` failed on #600, was retried and passed. It passed on glitch every time.
- **Not run:** Windows beyond CI, and no tmux run or video (not required for these bug fixes).

**Decisions for you (both tracked in #603)**
- **Should a person's one-command approval under Moderate (#540) also stop that command creating home entries?** Today the approval lifts the read-only home along with the other rules, as on main. Holding it would block approved commands that edit the home, such as `corbanu mcp add`. I recommend keeping it as is.
- **Commands working in the Corbanu home itself aren't covered:** when the working folder, a workspace root or a writable root is the home, or a folder of it outside `worktrees/`. The option is to protect the specific names Corbanu reads (`rules`, `hooks.json`, `agents`, …) before they exist.

**Follow-ups filed**
- **#594:** a Corbanu home reached through a symlink can still be replaced on Linux.
- **#598:** commands approved under #540 get no protection against renaming folders, on either platform.
- **#603:** the two decisions above and some edge cases.
- **#607:** #582's remaining items: a host rename over a live placeholder, and another run unlinking one.

**Cleanup**
- **Removed:**
  - the four worktrees and build folders on the Mac;
  - the checkout and its 67 GB build folder on glitch.
- **Kept:**
  - glitch logs in `~/corbanu-rtx/secb-logs-archive/`;
  - review rounds and PR texts in `.codex-work/workers-20261012-secb/`;
  - the four local branches, since deleting them would need a force-delete.
- **glitch disk:** it filled up briefly during one clippy run (other workers' builds); it has about 159 GB free now.

The full report is in `.codex-work/workers-20261002/sec-misc2.result.md`.