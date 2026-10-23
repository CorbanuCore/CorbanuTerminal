**Verdict: APPROVE.** Nothing blocks the merge. I found no denial that gets weaker and no host `/dev` exposed more than before. Two should-fix items are follow-ups, plus nits.

I reviewed the diff and code read-only. I couldn't read `gh issue view 493` or `gh pr view 496` because GitHub was unreachable from the sandbox, so I haven't seen the PR body itself. I didn't run any tests, so I haven't confirmed that they pass, or that they fail without the fix.

**1. Ordering: no blocking findings**
- **Where the fix fires:** `bwrap.rs:591-595` only triggers when `mount_root == "/"`. That covers a real `/`, a writable root that is a symlink to `/`, and the fallback that adds a writable `/` for a full-write policy with deny globs (`:385-393`).
- **Masks applied before the `/` bind:** there are none.
  - Unreadable ancestors (`:547-564`) need a path outside every allowed write path, which can't exist once `/` is writable.
  - No mount-target parent dirs are recreated for `/` (`:575-583`).
  - Roots are sorted by depth (`:532-533`), so the `/` pass comes before any `/dev/*` root.
- **Masks inside the pass:** read-only subpaths, snapshot dirs, the registry ro-bind (`:684-697`) and nested denies (`:698-704`) are all pushed after `--dev`. A deny under `/dev` would land on top of the fresh `/dev`, not underneath it.
  - Mount points for host-only nodes are created in the private tmpfs.
  - Synthetic-target cleanup already ignores paths that don't exist on the host (`linux_run_main.rs:1182-1185`).
  - Side benefit: placeholders are no longer created on the host's `/dev`.
- **Masks after the loop:** none apply once `/` is writable (`:714-727`).
- **Real profiles:** none deny anything under `/dev`. `rg '"/dev'` over core/protocol/sandboxing/config sources, including `protected_surface/`, found no deny entries.
- **Read-only binds under `/dev`:** these use the host node, mounted `nodev`. Before, the whole host `/dev` was already visible, so this exposes no more.

**2. Root edge cases**
- **`/dev` as a writable root:** `mount_root != dev` skips the extra mount. Its later `--bind /dev /dev` still covers the fresh `/dev` with the host one, mounted `nodev`. That bug predates this PR (see the `mounts_dev_before_writable_dev_binds` expectation at `:2390`; its comment "device nodes remain writable/usable" is wrong). Nit: fix the comment or open an issue.
- **`/dev/shm`, `/dev/pts`:** rebound from the host after `--dev`, and their own nested denies are re-applied in their pass. Correct.
- **Symlink to `/`:** depth sorting uses the symlink path, so shallower roots are bound earlier and the later `--bind / /` undoes their masks. That predates this PR and `--dev` doesn't make it worse, but it deserves an issue.
- **Workspace at `/` (not full-write):** it now also gets a fresh `/dev`. That's an improvement; make sure the PR body says so.
- **Restricted-read (tmpfs `/`) with a writable `/`:** handled the same way. No new bwrap prerequisite, because the base args already use `--dev`.
- **Should-fix (follow-up): I disagree that leaving `create_bwrap_flags_full_filesystem` (`:268-280`) unchanged is fine.** It's the same `--bind / /` with no `--dev`. That path runs for full-write with the proxy-only or restricted network, and `/dev/null` should break there too. The current test (`:1540-1572`) locks in those exact args, and nothing tests `/dev/null` on that path. Adding `--dev /dev` there carries no mask risk because there are no denials. Choosing `--dev-bind /dev /dev` instead would be a product decision. Open an issue rather than calling it intentional.

**3. Behaviour changes to flag in release notes**
- **`/dev/shm`:** previously the host's (shared and persistent); now a private directory unless it's configured as a writable root. Shared memory with host processes and files left there between commands no longer work.
- **PTYs:** a private devpts instance and a working `/dev/ptmx`. Previously ptmx and `/dev/tty` failed under `nodev`, so this is an improvement.
- **No longer visible:** `/dev/log`, `/dev/mqueue`, `/dev/hugepages`, `/dev/disk/by-*` and `/dev/serial/by-*`. `logger` will fail. Device serials/names are no longer disclosed, which is a plus.
- **GPU, `/dev/kvm`, `/dev/fuse`:** these failed before (`nodev` gave EACCES) and now fail with ENOENT. Not a regression, but full-write-with-denial users get no device access, which is a product question.
- **Safety:** nothing fails open. Underneath the new mount is either the host `/dev` mounted `nodev` or the first minimal `/dev`, the same as before.

**4. Tests**
- **Unit test (`bwrap.rs:2112-2160`):** the `["--bind","/","/","--dev","/dev"]` window check fails without the fix, and so does the updated exact-args test. Should-fix (low):
  - The deny under test is a temp path outside `/dev`, so `root_bind < deny_mask` doesn't prove the risk you named.
  - Add a deny on an existing `/dev` node (e.g. `/dev/full`): assert its mask comes after `--dev`, and that it's unreadable in the integration test.
- **Integration test (`landlock.rs:1343-1405`):** it covers null, urandom, deny read/write and a normal write. It should fail without the fix, given bwrap adds `nodev` to every submount. It skips silently when bwrap is unavailable, so the evidence should show it actually ran.
- **Seatbelt test (`seatbelt_tests.rs:1591-1638`):** a regression guard, not a fail-first test. The "no Seatbelt equivalent" claim is supported because Seatbelt filters by path, with no mounts, and `seatbelt_base_policy.sbpl:18-21` explicitly allows writing `/dev/null`.
- **Windows "no equivalent" claim:** plausible, since Windows has no mount layering and `NUL` isn't controlled by filesystem ACLs. But nothing tests it, so the PR body should say it's reasoned, not tested.