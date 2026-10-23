All four issues are fixed and merged with every check green, and #587, #583, #582 and #581 are closed with evidence. Each PR was approved by an independent Opus 5.5 High review. None changes Permissive.

| Issue | PR | Merge commit | Review rounds |
|---|---|---|---|
| #587 (HIGH) | #592 | `d878532cf6` | 2: changes requested, then approved |
| #583 | #596 | `398a3332e6` | 5: changes requested four times, then approved |
| #582 | #599 | `64387d3607` | 3: changes requested, then approved twice |
| #581 | #600 | `55380dfc47` | 3, all approved; the lows were fixed |

**#587: the Corbanu home can no longer be moved aside (Linux)**
- **Fix:** each folder between a writable root and a protected path (the Corbanu home and every folder above it inside the root) is now bound onto itself before the masks. A mount point can't be renamed or removed.
- **When it applies:** only once the policy denies reads, the same condition the macOS rename guard uses.
- **macOS:** already blocked. The new core test passes there on main.
- **On main (glitch):** under Moderate the exact attack (`mv home home.old; mkdir home && echo planted > home/AGENTS.md`) worked, with full access and with a workspace covering the home.
- **Known side effect:** a `rename()` across a pinned folder now fails with `EXDEV`. `mv` falls back to copying.

**#583: missing Corbanu-home entries can no longer be created**
- **The Corbanu home itself is read-only for protected commands.** This covers `rules/` (which would auto-approve commands next session), `hooks.json`, `agents/` and every other name.
- **No grant lifts it.** A person's approval of one command under Moderate (#540) lifts it together with the other rules, so approved commands such as `corbanu mcp add` work as on main.
- **`worktrees/` stays writable,** but only as a real folder; a missing one is created. The reviews caught:
  - git breaking in home worktrees;
  - `worktrees` links;
  - a linked home on macOS.
- **On main:** a protected command created `rules/x.rules` on both platforms.

**#582: placeholders left by a killed run are removed by the next run (Linux)**
- **How:** placeholders are now recorded in the existing marker registry. Each run removes those whose runs all ended, holds any it meets, and never fails because of this.
- **Registry safety:** only records owned by this user are acted on, and they are `0600`.
- **On main:** a run killed on its timeout left `home/Library` for good.

**#581: the bundled bubblewrap now starts on Ubuntu 24.04 (AppArmor userns restriction)**
- **How:** it is probed like the system copy, after digest verification and only through the verified descriptor, and runs through `aa-exec -p rootlesskit -- /proc/self/fd/N`.
  - It is verified again right before the exec.
  - The command never inherits the descriptor: the inner stage closes it first.
- **Startup warning:** without a system copy, it now says when the bundled copy can't start, and names the package and the profile.
- **On main:** the new test fails on glitch. The old bundled-copy test now runs instead of skipping.

**Gate evidence**
- **Regression tests:** each fails on main and passes with the fix:
  - glitch: #587, #583, #582, #581;
  - macOS: #583, plus #583's round-1 git regression.
- **Core security CI selection** (`security::`, `pf_23_s01`, `pf_25_s01`) with the default `TMPDIR` and `TMPDIR=/tmp`: passed on glitch for all four PRs and on macOS for #587 and #583.
- **`linux-sandbox` and `sandboxing` suites on glitch:** passed.
- **Clippy `-D warnings`:** clean on glitch for all four, and on macOS (#581/#582 for `sandboxing` and `linux-sandbox`).
- **Merged-state checks:** #600 was checked merged with main plus #599, and #596 merged with main plus #591, #599 and #600, on glitch and, for #596, macOS. All green.
- **CI:** one flaky ubuntu `pf_23_s01_typing_a_protected_command…` failure on #600 was retried and passed. It passed on glitch every time.
- **Not run:** Windows beyond CI. No tmux or video run; the bug-fix gate didn't call for one.
- **macOS test noise (unchanged):** two Seatbelt snapshot tests fail only with this worker's `TMPDIR` on CorbanuDrive. They pass with `/tmp`, as on main.

**Decisions for you**
- **#603: should a person's one-command approval (#540) also stop that command from creating home entries?**
  - Currently it lifts the read-only home with the other rules, as main does for existing entries.
  - Holding it would block approved commands that edit the home (`corbanu mcp add`).
  - **Recommendation:** keep it as is.
- **#603: commands that work in the Corbanu home itself are not covered.**
  - This means the working folder, a workspace root or a writable root is the home, or a folder of it outside `worktrees/`.
  - **Suggestion:** protect the names Corbanu reads (`rules`, `hooks.json`, `agents`, …) before they exist.

**Follow-ups filed**
- **#594:** a Corbanu home reached through a symlink can be replaced on Linux.
- **#598:** #540-approved commands get no rename protection (both platforms), plus nits.
- **#603:** the two #583 cases above, a `worktrees` swap by a planter outside the protected rules, and usability on linked workspaces.
- **#607:** remaining placeholder issues:
  - a host rename over a live placeholder;
  - another run unlinking one;
  - an empty placeholder folder that is not registered;
  - PID namespaces;
  - records name a path, not an inode;
  - symlinked parents followed when removing.

**Concurrent work**
- #591 (sbx-links2) touched `seatbelt_tests.rs`; it merged cleanly. #596 was verified merged with it.
- glitch disk was briefly full during one clippy run (another worker's builds). I re-ran it; free space is about 159 GB now.

**Cleanup**
- **Removed:**
  - all four Mac worktrees and build folders;
  - glitch's `~/corbanu-rtx/secb` checkout and its 67 GB build folder.
- **Kept:**
  - glitch logs in `~/corbanu-rtx/secb-logs-archive/`;
  - review rounds, PR texts and scripts in `.codex-work/workers-20261012-secb/`;
  - the four local branches, since deleting them needs a force-delete.
