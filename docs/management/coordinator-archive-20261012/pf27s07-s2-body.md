Slice 2a of PF-27-S07, split at ~800 lines. Slice 2b (the broker started by `spawn_protected`) follows once this merges.

PF-27-S06 left a limit: a new thread started with the token's default DACL (the user has full access) and got the protected DACL only in its TLS callback. In that window, a same-user unsandboxed process could open the thread and keep the handle.

**Core and broker (`restrict_current_process_access`)**
- This image's `CreateThread` imports are redirected to a wrapper that passes the protected thread descriptor.
- Every Rust thread (std, tokio) therefore has the protected DACL from creation.
- Hardening fails closed if the image imports no `CreateThread`, and the targets assert `thread_creation_protected()`.
- Threads Windows starts itself still get the protected DACL only in the TLS callback. Core cannot use the default DACL, because its child pipes (std's anonymous pipes are created without a descriptor and reopened) and its child processes inherit it.

**Broker containment (`contain_credential_broker`)**
- `protect_new_objects_by_default()` sets the token's default DACL to the protected thread DACL, so every thread the broker starts afterwards, however it starts, is protected at creation.
- The broker starts no children and reopens nothing it creates without a descriptor.
- The `dacl` mechanism now requires both calls to succeed.

**Probes (Windows CI)**
- Targets hold two brand-new threads in their creation window. Each is created suspended, so its TLS callback has not run: one goes through the `CreateThread` import, one through `CreateRemoteThread` on itself.
- `pf_27_s07_imported_thread_creation_is_protected_from_the_start`: the imported thread is denied `GET_CONTEXT`, `SET_CONTEXT` and `SUSPEND_RESUME` to a same-user process without privileges and to the unelevated sandbox's restricted token. The direct thread is granted to the same-user probe (positive control: the probe reaches the window).
- `pf_27_s07_default_dacl_protects_every_new_thread`: both threads are denied to both probes.
- The targets assert that a new thread keeps its own rights (priority, `ImpersonateSelf`).
- A unit test checks the import scan. The PF-27-S06 probes and the broker suite run unchanged.

Sprint record: `docs/sprints/current/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md` (#280).
