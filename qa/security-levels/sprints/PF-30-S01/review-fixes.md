# PF-30-S01 review-fix checkpoint

Date: 2026-08-27. Change class: existing **product initiative**, plan
`docs/plans/active/p0-security-levels.md`, sprint **PF-30-S01**, `in_progress`.
Product authority: **Moderate/Aggressive isolation and content provenance** —
“Network destinations and redirects are enforced”; “Reuse an installed Podman
or Docker runtime without replacing it or changing its global configuration.”
Travis requested the confirmed review fixes followed by a commit. No push or
release is part of this checkpoint.

## Candidate and fixes

Worktree `/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01`, branch
`codex/pf-30-isolated-runtime`, planning HEAD
`399daef151592505f4057d52bbe20fc48a41106d`. Final source is identified by the 28
entries in [candidate-files.sha256](candidate-files.sha256). The pre-fix source
and 266-test run retain separate `pre-fix-*` artifacts; they are not relabeled as
proof of these fixes.

Committed code candidate: **`322c81b88cbe8e90ba02b16937a4c7b4a792bb1c`**.
The backend was landed as reviewable stages, each below 800 changed lines and
500 non-test implementation lines where practical:

| Commit | Stage |
| --- | --- |
| `cae7c7e75` | Public-web destination policy and regressions |
| `25a5d3d30` | Engine discovery, bounded command execution and pinned images |
| `c4fc7a938` | Owned lifecycle and networkless worker |
| `11813e81a` | Bounded broker and download quarantine |
| `322c81b88` | Workspace/Bazel registration, runtime and thin Core adapter |

The new crate's intermediate source stages remained unregistered until the last
code commit. The final integrated source hashes, tests and review—not individual
intermediate build claims—provide this checkpoint's evidence. This is a committed
internal backend, not a completed or publicly enabled feature.

- **P1 fixed:** `resolve_browser_destination` applies method and native host
  policy before its bounded lookup. The shared resolver path covers initial
  requests, subresources and redirect prechecks. An injected-resolver regression
  asserts zero calls for explicitly denied plain and trailing-dot hosts; an
  allowed-host test verifies exact public answers and rejects private answers.
- **P2 fixed:** full 64-hex image IDs accept the Docker `sha256:` prefix or the
  bare Podman form, preserving the engine-native ID for subsequent commands.
  Tests reject malformed/short/overlong IDs and exercise cached-image preparation
  through the subprocess fixture for both formats and a recipe-label mismatch.
  Existing image, user, entrypoint and container ownership checks are retained.
- The withdrawn trailing-dot policy-bypass claim received no production fix;
  the new denial test also covers that normalized form. The 128 KiB output cap
  is unchanged pending observed engine payload sizes.

Only five source files changed since the initial review: browser policy and its
tests, image validation and its new sibling tests, and the test-only engine
fixture. No shared native runtime, Core authority, public API, provider wire,
history, persistence, installer or user activation change was made in this cycle.
The upstream baseline and native adapter dispositions remain in runtime-selection
and the plan's upstream-touch record. No upstream upgrade is claimed.

## Final-tree verification

From `codex-rs`, in order:

```text
CARGO_TARGET_DIR=/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01/codex-rs/target just fix -p codex-browser-isolation -p codex-network-proxy --profile ci-test
just fmt
CARGO_TARGET_DIR=/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01/codex-rs/target just test -p codex-browser-isolation -p codex-network-proxy -p codex-core --lib --cargo-profile ci-test -E 'package(codex-browser-isolation) | package(codex-network-proxy) | (package(codex-core) & test(security::))'
```

Lint/fix and formatting passed before final tests. **270 passed**: 18 browser,
223 network-proxy, 29 Core security. **2,306 other Core tests were excluded**;
this is not the complete Core suite. Existing Core dead-code warnings remain.
[JUnit](focused.junit.xml): run `2ffc8d8e-2def-4cfe-9dfd-f5f5341d5b28`,
2026-08-27T18:07:51.982-07:00. Its `skipped=0` describes only selected tests.

From the repository root, `python3 -B -m unittest discover -s
codex-rs/browser-isolation/worker -p 'test_*.py'` passed all **4** worker tests.
The new subprocess fixture runs on Unix; parser validation is platform-neutral.
Neither fixture is a real Podman/Docker or Chromium qualification run.

`just bazel-lock-update` passed with no `MODULE.bazel.lock` delta. Plan and sprint
checkers, `git diff --check`, final source fingerprint validation and the existing
documentation environment's strict MkDocs build also passed. No source changed
after the final affected test run; staging and committing preserved those bytes.

## Independent review

Fable 5 High fix-cycle review was requested in the same logged-in Claude desktop
session via Computer Use, with Manual permissions and the same read-only scope.
The original candidate is preserved in `before-fixes/` inside the isolated
review folder. The reviewer must re-read current source and its interactions;
no code execution, nested reviewer or fallback model is allowed.
Final verdict: **clean scoped review; no remaining actionable findings** in
fix cycle 1. Fable verified both fixes, their call sites, all 28 candidate hashes
and the five-file delta. [Captured verdict](fable-gui-fix-review.txt), app session
`local_11570510-8846-4178-8837-761f35b2aa20`. No fallback or nested reviewer was
used. Tests completed successfully separately; the review did not assert or
replace their outcome. The carried qualification/follow-up items below remain.

## Remaining gates

The unchanged native `host_blocked` path still performs its private-IP DNS check
before rejecting names absent from its allowlist. This is a separately tracked
integration-owner follow-up outside S01's current write scope; explicit-deny
ordering is fixed, but full DNS-boundary qualification remains blocked until the
residual is resolved with properly allocated scope.

Broker redirect fixtures, worker request-handler branches, engine endpoint JSON
coverage and measured engine payload sizes remain on the sprint checklist.
Mac/Linux runtime prerequisites and real containment/egress/lifecycle evidence,
then Windows qualification, are still pending. S01 is internal and unactivated,
so no S01 TUI workflow is applicable; S02/S03 own the actual-key user flows.
Live-repository, human acceptance (Travis Good), benchmark and release gates are
not passed by this development commit. Do not archive S01 or start dependents.

Subsequent [platform qualification](platform-qualification-2026-08-27.md)
recovered Mac Docker and installed rootless Linux Podman with Travis's explicit
authorization. Both unchanged candidates then failed with `HealthCheckFailed`:
Docker rejects the private PID/UTS arguments; Podman's expanded dropped-capability
list is rejected by the verifier. A Mac diagnostic also exposed unverified
disabled seccomp. Windows is unreachable. All platform qualifications remain open;
the earlier scoped review does not supersede these newly observed failures.

The later [platform repair checkpoint](platform-fixes.md) fixes those three
findings in four browser-crate files. Final tests passed: 272 focused Rust tests,
six worker tests on each host, real Mac/Linux backend smokes and fail-closed
seccomp/capability checks. Fable 5 High found no in-scope defects in that frozen
patch. Full platform/lifecycle qualification, the residual native DNS issue and
Windows remain open; neither clean scoped review closes those gates.
