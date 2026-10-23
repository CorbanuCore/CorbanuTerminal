I found a real sandbox escape, and it isn't specific to admins. I've fixed the most serious part (protected folders like `.git` being deleted and replaced); deletes outside the writable roots are still possible and are now documented. PR #163 is open against main and not merged, findings are posted on #158, and all `debug/sandbox-158-*` branches are deleted.

**Verdict.** The unelevated sandbox runs commands with a write-restricted token. Windows checks the sandbox's capability identities for writes and for `DELETE`, but not for "delete child" on the parent folder. When `DELETE` is denied, NTFS allows the delete or move anyway if the parent folder grants "delete child". Any folder where the user has full control grants it, so the parent's normal ACL overrides the sandbox's deny entries on the child. Upstream's openai/codex#31138 assumed this route was closed.

What this exposes:
- **Users affected:** anyone on the unelevated backend, admin or not. I reproduced it as the built-in Administrator and as a standard local user created in the workflow.
- **Outside writable roots:** a sandboxed command can delete anything whose parent grants the user that right, which is most of the user profile. This happens in both read-only and workspace-write modes. Creating or writing files there is still blocked.
- **Protected folders:** in workspace-write, `.git` (and `.codex`/`.agents`) could be deleted and recreated with planted content. I confirmed a planted `.git/config`. Git hooks or config planted this way run outside the sandbox later, so this is a code-execution escape.
- **Admins:** the Administrators group is deny-only in the sandbox. Folders granted only to Administrators, like the runner checkout, are safe. That's why the test behaved differently under cargo and under Bazel.
- **Elevated backend:** not affected in my probe, because its sandbox group has Modify, which doesn't include "delete child".

**Evidence** (hosted `windows-2022` runs):
- **37281712086:** `whoami /all` and `icacls` inside and outside the sandbox, plus access probes. Variants: test root under the checkout, under `%TEMP%`, with an explicit ACL, with an Everyone ACL, Low integrity level, and the non-admin `sb158user`. With the explicit ACL, the sandbox was denied write and permission changes on the outside file but granted "delete child" on its parent, and the file was deleted.
- **37284548433 / 37285401487:** with a root whose ACL doesn't depend on the runner, the test now fails deterministically. Read-only mode deleted workspace files, outside files and `.git`. Workspace-write replaced `.git` with the planted config. The elevated backend held.
- **37286739260 (decisive):** a file the user has full control of, in a parent without "delete child", survived a sandboxed delete. Adding a deny of "delete child" blocked every attack on `.git`, and normal deletes, sandboxed and unsandboxed, kept working.

**Fix and tests:**
- **Fix:** legacy sessions now deny Everyone "delete child" on each protected path that exists inside a writable root, on its subfolders, and on its parent. Junctions and symlinks are skipped. These deny entries are permanent and visible in the folder's security settings; the README documents this and gives a removal command.
- **New test:** `legacy_workspace_write_cannot_delete_or_replace_protected_git_dir` tries to move, delete, rename and recreate `.git` and its hooks. It checks that workspace deletes still work and that you can still delete `.git` afterwards.
- **Runner-independent tests:** test roots now get their own explicit ACL. A new positive test, `legacy_workspace_write_can_delete_inside_writable_roots`, keeps coverage of deletes inside the workspace.
- **Original test:** `legacy_workspace_write_delete_is_limited_to_writable_roots` now fails only on the outside file. I marked it `#[ignore]` as a known limitation and removed its skip from `.bazelrc`.
- **Why outside deletes stay open:** blocking them would mean denying "delete child" across your profile. The token API can't add a sandbox-only group, disabling the user identity breaks reads, and Low integrity level still lost `.git` and hung the standard-user run.

**Review:** two rounds of independent Opus 5.5 High review. Round 1 asked for changes to the docs and comments, which I made. Round 2, on the fix, returned APPROVE WITH NITS (README caveats, skip junctions, check host deletes). I addressed those three. It also flagged that this changes the sandbox ACL boundary, so you should record whether it ships as a bounded security fix.

**PR:** https://github.com/CorbanuCore/CorbanuTerminal/pull/163
- PR CI doesn't run the Windows sandbox tests, so I ran the full `codex-windows-sandbox` lib suite in debug workflows instead. Main failed 2 of 125 tests and this branch fails 1 of 127. That one is `legacy_capture_emits_output_and_preserves_descendant_after_normal_exit`, which also fails on main.
- PR checks: 29 passed, 9 skipped, 0 failed, including codespell (after a fix) and the Windows argument-comment lint. 6 were still pending when I stopped: the Linux release and ptrcomp builds, `Analyze (rust)`, and the Windows credential canary.

Follow-ups listed on #158, which stays open:
- Protect the working directory in read-only mode.
- Track and clean up the Everyone deny entries.
- Protect `.codex`/`.agents` when they don't exist yet.
- Handle renaming a writable parent of a nested protected folder.
- Decide whether the unelevated mode should warn users that it doesn't confine deletes.