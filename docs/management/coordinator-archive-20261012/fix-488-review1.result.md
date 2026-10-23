**Verdict: APPROVE.** Nothing blocks merge. I list three should-fix items below, but none of them is a regression from this branch. All three can be follow-ups.

Two things changed while I was reviewing. The branch has three commits, not two (`2984b5928e`, `b6af858612`, `ba9656817f`). And HEAD moved to a merge commit, `18438706da` ("Merge origin/main"). I checked the branch's diff against `origin/main`: it is the same seven files I reviewed, and the #488 bubblewrap change sits correctly next to main's #473 snapshot logic (`bwrap.rs:623-686`). I couldn't reach GitHub, so I didn't read issue #488 or PR #479 themselves.

## What I checked

**1. Only denied (or read-only) paths gain exclusions, and nothing allowed becomes denied.**
- In Aggressive, Permissive-with-denials and read-only, reads always go through the per-root branch. Any read denial makes `has_full_disk_read_access()` false (`protocol/src/permissions.rs:666-674`). So the full-disk-read branch with exclusions (`seatbelt.rs:434-443`) is effectively unreachable, and it behaves as before anyway.
- On the read side, the only new input to exclusions is `get_unreadable_roots_with_cwd` (`seatbelt.rs:448-456`). On the write side it is `read_only_subpaths` plus denied paths (`seatbelt.rs:922-933`). The full-disk-write branch is unchanged apart from extra spellings.
- `reached_through_root` (`seatbelt.rs:413-421`) only attaches a denied path X to a root R when X sits at or below R. Three cases follow:
  - **Allow inside a denied path:** a readable or writable root R deeper than X never gets X, through a link or not. Seatbelt combines clauses with OR, so carve-outs survive.
  - **Root is `/`:** it gets every denial, as before. `/` itself is filtered out of the denied list (`permissions.rs:1153-1156`).
  - **Missing, dangling, relative or `..` paths:** the denied paths are already absolute. A dangling link resolves to its parent plus its own name. When a path can't be resolved, the code keeps it as written. That can only add a harmless extra exclusion, never remove one.
- **Semantic change to disclose (not a bug):** a denied link now also denies its target through every root. If the target *is* a root (for example a denied `~/.config/x` pointing at the workspace), that whole read and write clause is switched off. The same happens on bwrap: a denied `real` with a writable `link` gets a `000` tmpfs over the workspace. That matches "a denial must hold", but the change record should say it.
- The old code already resolved each exclusion to its real path (`normalize_path_for_sandbox`, `seatbelt.rs:172-186`). So "deny a link, exclude its target" isn't new. What's new is attaching it across roots.

**2. The denial now holds for every spelling.**
- Read and write clauses now use both the fully resolved spelling and the "nearest existing folder resolved" spelling (`seatbelt.rs:397-407`). That covers links whose targets are themselves links, `/tmp`, TMPDIR, and roots that are links.
- The write side really did leak before: `real/.env` under a writable `link` was in no `read_only_subpaths`. It is fixed by `seatbelt.rs:929`.
- The rename guard is a pure refactor (`seatbelt.rs:589-618`). It still covers the as-written spelling, so a denied path spelled through a link can't have its link unlinked or renamed away.
- Paths remaining open, all outside this diff: hard links; a denied dangling link's future target; and case or Unicode-normalization differences on APFS for paths that don't exist yet (unverified).

**3. `seatbelt_allows_file_read` (`seatbelt.rs:471-486`) errs safely.**
- It returns "readable" whenever platform defaults apply, ignores glob denials, and evaluates exactly the clauses that `create_seatbelt_command_args` emits (`seatbelt.rs:945-962`).
- The only base-profile read allows it doesn't model are `/dev/ptmx` and `/dev/ttys*`, which it documents. Unix-socket rules are network rules only.
- Aggressive extends `:workspace` (root read, `aggressive.rs:137-144`), so it never uses platform defaults. If it ever did, the launch check would report every path and refuse to start, which fails safe.
- The database-glob handling in `verify_isolation` (`preflight.rs:224-247`) is unchanged.
- **Nit:** the helper evaluates the launch-time policy and cwd. Later per-turn policy changes aren't covered, which is acceptable since Aggressive disables permission requests.

**4. Bubblewrap (`bwrap.rs:627-641`).**
- It only adds masks for denied paths, and only when the root resolves through a link. Ordinary roots return `None` and are unchanged.
- Those paths were previously in neither the "ancestors of writable roots" pass nor the "rootless" pass, so nothing gets masked twice. Dedup is in place and the test checks a single mask.
- Masks still apply after the bind. Missing paths go through the #473 snapshot/placeholder logic.
- The fail-closed check for writable links still applies, because paths are remapped before `append_unreadable_root_args`. Writable roots under a linked parent folder are covered because `canonical_target_if_symlinked_path` checks every component.

**5. Windows** is unaffected by this kind of bug. It puts a deny ACL entry on every denied path, with no per-root attachment (`windows-sandbox-rs/src/deny_read_resolver.rs:39-43`). I didn't verify how those entries behave for links versus their targets.

## Findings

**Blocking:** none.

**Should-fix (follow-ups, none introduced by this branch):**
1. **Remaining Linux leak when reads are restricted.** Without `:root` read, a readable root that is a link and not writable is mounted by its link path (`bwrap.rs:498-510`). A denial spelled with the real path is then masked by the "rootless" pass at a path that doesn't exist inside the sandbox, so the file stays readable through the link. Aggressive isn't affected because it mounts `/` read-only, but `:minimal` profiles and the default read roots could be (for example `/lib -> /usr/lib`). The non-macOS `verify_isolation` would report such a file as protected.
2. **Linux: a denial spelled through a link stops every command.** If the workspace is the real path, bubblewrap fails with "Can't create file", so nothing runs (`landlock.rs:1414-1419`). This is safe but unusable: under Aggressive, a credential path with a linked folder in it would break all agent commands. Consider resolving links that aren't writable before masking. The test also depends on an exact bubblewrap error message, which can change between versions.
3. **Test gaps.**
   - Nothing directly tests the riskier direction: a denied folder with a more specific allow under it, spelled through a link. Nor a denied link that points at a root.
   - In the real sandbox-exec run, the read checks for the "missing denied paths" scenario prove nothing (`seatbelt_tests.rs:1865-1869`): `cat later.key` fails because the file doesn't exist. Creation is covered separately.

**Nits:**
- Each run resolves every (denied path × root) pair twice; resolving each denied path once would be cheaper.
- Note in the change record that a denied path pointing at a root now switches that root's clause off.

## Test quality

The tests would fail on the old code:
- On macOS, the nested-link workspace has no exclusion on the `real` clause, and `real/.env` is writable.
- A missing `link/never.key` was emitted as written, so Seatbelt never matched it.
- The Linux test failed with the secret in stdout, as you stated.

The helper's verdicts are checked against independently parsed arguments, and each claimed exclusion is checked in the policy text.

Flakiness risk is low. Tempdirs have their real paths resolved, and the `/tmp` case is intentional. The Seatbelt tests are macOS-only (`lib.rs:7-8`). As with the existing sandbox-exec tests, they can't run inside another Seatbelt sandbox.

I made no edits and ran no builds, tests or TUI.