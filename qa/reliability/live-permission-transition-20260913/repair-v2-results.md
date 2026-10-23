# PF83 Fable01 remediation cycle 1

Sole code owner: fresh Astra High implementation context. Product initiative,
PF-83-S01 in_progress; Permission selection confirmation — TO BUILD:
“A submitted selection is not a confirmed change.” Worktree, base and exact
commands are in `verification-command-v2.md`; parent owns plan/sprint ledgers,
fresh Fable review, packaging and independent functional execution/evidence.

## Repairs

Confirmed selection now requires the correlated Applied outcome and an observed
matching profile/policy/reviewer (an already matching snapshot handles no-op).
Either arrival order completes. App defaults and reload overrides copy the
latest effective permission bundle, so `/new` cannot silently reinstall its
old disk default after a confirmed downgrade. A newer observed selection is
preserved and displayed as superseding the applied request. Missing observation
expires as uncertain after 20 seconds; this does not retry the request.

Async completion now releases held initial input only for matching success.
Normal suppression/setup/direct-input gates remain, and active-turn/queued
work causes restoration instead of steering or queueing the initial message.
Failure, uncertainty, unsupported or superseded outcomes restore it without
submission. Existing composer merge preserves local/remote images, placeholder
ranges, mentions and pending pastes; no queue drain/auto-send is invoked.
Stale request IDs and results for a different active thread are ignored.

No Core/protocol/server behavior changes in cycle 1. V1 server/protocol/schema
files remain in the complete candidate and are rerun as affected support.
Current-turn command authority, approval handling and MCP refresh remain owned
by the unchanged Core paths. No installed-runtime repair is claimed.

## Retained attempts and diagnosis

- `tui-v2-attempt-1.log`, exit101: test-only compile errors, wrong AppEvent
  submission variant and moving Config out of an App with Drop; no tests ran.
- `tui-v2-attempt-2.log`, exit100: 5/7 passed. Native fixture refutable select
  patterns stopped polling a branch after unrelated events; use irrefutable
  receives with inner matches. New superseded-state snapshot was inspected and
  accepted; its failed attempts are retained. No timeout increase.
- `tui-v2-attempt-3.log`, exit100: 6/7 passed. Native fixture defined named
  profiles without mandatory default_permissions. Corrected its disk default
  to Full Access, making the restrictive fresh-session check adversarial.
- `focused-v2-final-1.log`, exit100: 351/352 passed, nextest
  `d8005700-0cae-41e3-8f26-6ecbf60c8a5f`; new native fixture failed twice.
- `tui-v2-diagnostic-1.log`, exit100: exact native error establishes fixture
  cause: `named-full` cannot extend unsupported built-in `:danger-full-access`.
  Existing `permissions_profiles_reject_unsupported_builtin_extends_parent`
  confirms this rule. Replace only fixture definitions with supported named
  workspace/read-only profiles. Built-in Full Access/read-only transitions
  remain. No policy/runtime relaxation or timeout change.
- Initial broad source count briefly reached1653; shared test setup removed
  duplication, preserving behavioral cases and the original allocation ceiling.
- `focused-v2-final-2.log`, exit100: 351/352 passed, nextest
  `3cffdfb8-cfdf-48de-92c1-96f16674e6f5`. Supported named selections succeeded;
  the fresh-session assertion compared unresolved ProjectRoots entries against
  their materialized `/tmp/project` paths. Existing
  `display_permission_profile_from_thread_response` returns
  `config.permissions.effective_permission_profile()` in embedded mode, and
  that accessor explicitly materializes workspace roots. Compare the complete
  effective profile, preserving all filesystem/network entries and the separate
  exact active-profile metadata assertion. No runtime or timeout change.

## Evidence and remaining gates

Final exact-tree run `focused-v2-final-3.log` exited 0: **352/352 passed**,
8695 skipped across 17 binaries; nextest execution 56.879 seconds, run ID
`66bd68a5-09e9-49eb-9ab0-8241904eaf67`. Formatting preceded this run; no source
changes followed it. All failed attempts above remain available.

Frozen full candidate: `implementation-v2.patch`, 34 paths within literal
PF-83-S01 scope. Exactly nine TUI paths changed relative to v1; all other v1
candidate hashes remain unchanged. Counts: **637 non-test + 1012 test = 1649
hand-authored Rust lines**, within the approved 750/1650 limits. Separately:
42 mechanical schema text lines plus two compressed fixtures, 32 snapshot lines,
and the API README. Per-file hashes and cycle paths: `candidate-manifest-v2.md`.

- V2 patch SHA256: `d8189e02ead3c905972e12b8ba6782d4d9ed8f06e53e9ec7988443dcd6b98e3b`
- V2 manifest SHA256: `8e4aa05eb45fcafb1094e35061aabe7acc3b4227b9b25d21fc146850d1a34706`
- Final3 log SHA256: `f4771fbffa6cf8083d8324f9e51d49174d05dc5d1ea722bcea6326b771036a9a`

Freeze verified unchanged base, no staged changes, v1 patch/manifest hashes,
literal scope, counts, and reverse applicability without applying the patch.
`git diff --check`, plans checker (3 active) and sprints checker (116 current,
126 archived) passed. Parent-owned documentation was preserved.
The formerly failing native regression passed in 1.696 seconds, without retry;
the final log contains no FAIL/TRY entries. Final process inventory contained no
cargo, rustc, nextest or native test executors; no owned source jobs remain.

Coverage: native embedded requests in both directions, same-state selection,
named workspace/read-only profiles, missing profile rejection, serialization,
new-thread startup after disk reload with old Full Access default, complete
permission/profile metadata comparison; controlled effective-snapshot/reply
ordering across both profile forms and later observations; held input success,
failure/unsupported/uncertain recovery, stale/duplicate/thread-switched results;
full restored payload/queue/running-state checks and preserved approval/interrupt
regressions. Unit/native fixtures are supporting evidence, not acceptance.

Fresh Fable review of this corrected tree is required next. V1 Fable01 findings
are addressed for review, not independently accepted. No additional review was
launched by this worker. Independent isolated execution still requires repaired
native network/IPC confinement, a rebuilt exact v2 package, original F01–F11 with
the approved next-turn amendment in both disposable live repositories, and a
separate evidence check. No Windows run, real old-server binary comparison,
human acceptance, integration, release/benchmark or live-app result is claimed.
No commit/stage/push/merge/install, live user home/profile/launcher/stable link or
credential change; no portfolio work resumed. No independent execution performed.
