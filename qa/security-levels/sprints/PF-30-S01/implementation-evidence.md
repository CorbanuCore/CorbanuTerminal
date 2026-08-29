# PF-30-S01 implementation checkpoint

Date: 2026-08-27. Owner: Jim Ricketts. Requested reviewer: **Fable High**.
Historical status: **uncommitted backend draft; sprint incomplete, not platform-certified**.
The subsequent fix/commit checkpoint is [review fixes](review-fixes.md); this
initial record retains its original 266-test and pre-fix candidate evidence.

## Candidate

- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01`.
- Branch: `codex/pf-30-isolated-runtime`.
- Base: `9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c`.
- Planning HEAD: `399daef151592505f4057d52bbe20fc48a41106d`; implementation is the
  local changes over that commit, **not code contained in that commit**.
- Product authority: **Moderate/Aggressive isolation and content provenance** —
  “Support Windows, Linux, and macOS with containerized Scrapling.”
- Existing pinned dependency/image inputs: [runtime selection](runtime-selection.md).
- No public tool, installer UI, model-visible context, provider, persistence or
  history activation. Permissive exits before engine discovery or image setup.

## Implemented draft

The `codex-browser-isolation` crate owns engine selection, the fixed image recipe,
container ownership/lifecycle, the request broker, Scrapling worker and in-memory
download quarantine. Core adds only a native live-policy/inspector adapter.
The network-proxy module composes existing denials without approval overrides.

- Existing Docker/Podman is selected without changing global contexts. Engine
  commands use checked absolute executable paths, a bounded environment and
  neutral working directory. Host CLI credentials/environment are not passed
  into the worker. No installer or elevation password handling is implemented.
- The fixed derivative recipe starts from the recorded immutable Scrapling
  digest, relocates the packaged browsers for UID 65532, and copies only the
  fixed worker. Building uses `--network=none --pull=false`; running uses the
  inspected immutable output image ID. No runtime package resolution occurs.
- Each acquisition gets an owned, networkless, read-only container, fresh tmpfs
  profile, dropped capabilities, no-new-privileges, private namespaces and
  bounded memory/CPU/processes. There are no caller-specified mounts or flags.
- Stopped owned containers start; a failed real Chromium probe permits one
  restart. Ownership is checked before destructive actions. Normal cancellation
  awaits cleanup; dropping the entire operation uses best-effort cleanup, and
  the worker has a bounded lifetime if the host exits. Abrupt host/daemon failure
  and delayed-create cleanup still require real qualification.
- Worker requests cross a bounded stdio protocol. The host allows only public
  HTTP(S) GETs on standard ports, rejects credentials/private/special addresses,
  checks every redirect, pins actual connection addresses with **no resolver
  fallback**, disables proxies/automatic redirects, and retains TLS validation.
  Caller/browser cookies and authorization headers are never forwarded.
- Limits: 64 requests, eight redirects, 2 MiB per response/DOM, 16 MiB aggregate
  fetched data, eight quarantined artifacts, 6 MiB wire frames, 75-second
  operation budget plus bounded cleanup. The worker has a 90-second service
  lifetime. No download is written to the workspace.
- Promotion consumes an opaque artifact and requires a synchronous trusted-host
  approval callback bound to digest, size, destination and current authority.
  That callback must compose native write policy and explicit human consent;
  S02 owns this integration. This is **not an already-wired user approval flow**.
- Core reads live policy/epoch/kill-switch facts; browser health does not assert
  content-firewall, confidentiality or protected-action readiness. S02 must
  connect the raw acquired bytes to PF-29 before model-visible use.

## Verification

Initial-checkpoint source fingerprints: [pre-fix candidate files](pre-fix-candidate-files.sha256),
relative to the worktree root at that checkpoint. These identify the historical
pre-fix source, not the current tree; current hashes are linked from review fixes.

| Check | Result |
| --- | --- |
| `just fix -p codex-browser-isolation -p codex-network-proxy -p codex-core --profile ci-test`, then `just fmt` | pass before final affected tests; existing Core dead-code warnings remain |
| `just test -p codex-browser-isolation -p codex-network-proxy -p codex-core --lib --cargo-profile ci-test -E 'package(codex-browser-isolation) \| package(codex-network-proxy) \| (package(codex-core) & test(security::))'` | **266 passed**: 16 browser, 221 network-proxy, 29 Core security; 2,306 other Core tests filtered out; not the complete Core suite |
| `python3 -B -m unittest discover -s codex-rs/browser-isolation/worker -p 'test_*.py'` | **4 passed**; protocol/callback tests, not live Chromium |

The Rust run used
`CARGO_TARGET_DIR=/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01/codex-rs/target`
to reuse the existing build cache. JUnit is [pre-fix focused results](pre-fix-focused.junit.xml),
run `48157b84-3115-419f-9100-0e7bbbbea9d5`, 2026-08-27T16:40:22.993-07:00.
The JUnit `skipped=0` describes the selected tests; the nextest filter excluded
the 2,306 other Core tests before execution. These tests do not establish live
containment, runtime portability or final acceptance.

`just bazel-lock-update` completed successfully; no `MODULE.bazel.lock` change
was needed. Cargo lock changes add only the workspace crate/dependency edge.
Plan and sprint structural checks passed (one active plan; 24 current sprints).
`mkdocs build --strict` passed using the existing documentation environment at
`/Users/travisgood/Documents/ChatGPT/corbanu/.venv-docs/`; output is
`/tmp/corbanu-pf30-docs.vFsQiK`. Final source fingerprint verification and
`git diff --check` also passed. The expired temporary SSH control directory was
removed after confirming it was empty; no test credentials were retained there.

The real `qualify` example was built and invoked on macOS. It stopped at engine
discovery because Docker Desktop's server was unavailable; no container/image
qualification pass, image pull or image build is claimed. A Docker client-only
JSON response exposed an error-classification issue, now covered by a regression.
The final-tree retry of `cargo run -p codex-browser-isolation --example qualify
--profile ci-test` built successfully and exited 1 with `RuntimeUnavailable`.
This is a blocked real probe, not a successful acquisition or containment test.

## Blocking evidence and required next steps

The following is the historical checkpoint's blocker ledger. The latest
[review-fix checkpoint](review-fixes.md) records the resolved findings and commit.

1. **Fable High review:** Autoreview used Claude Code 2.1.169 from an isolated npm
   cache, `--engine claude --model claude-fable-5 --thinking high`. Both attempts
   returned HTTP 401, invalid OAuth access token, with zero model tokens. Neither
   is a completed review or a clean finding set. The installed global CLI was
   not replaced by this task. No fallback reviewer is authorized.
   After Travis reported signing back in, the review was retried on the same
   unchanged candidate with both cached CLI 2.1.169 and the now-installed CLI
   2.1.248. Both returned the same HTTP 401, with zero model tokens and no
   review findings. Both `auth status` checks reported `loggedIn: true`,
   `authMethod: claude.ai`, `apiProvider: firstParty`; no Claude/Anthropic auth
   environment override was present. Only non-secret authentication metadata
   was inspected. The reason the API rejects the saved login is unresolved.
   At Travis's subsequent request, Computer Use ran the review through the
   logged-in Claude desktop app with Fable 5 / High effort. The review completed
   on the unchanged candidate; the GUI route is no longer authentication-blocked.
   See [desktop review and corrected verdict](fable-gui-review.md). One P1
   DNS-before-policy defect and one P2 Podman image-ID incompatibility remain
   open. The reviewer withdrew a trailing-dot false positive after code-based
   reconciliation; the engine-output cap remains an unconfirmed risk. No fixes
   are claimed by this review, and the final corrected tree still needs review.
   Candidate fingerprints match. Docker was not requalified during this GUI run;
   the last daemon check remained unavailable.
2. **macOS:** complete Docker Desktop startup/approval so the existing
   `desktop-linux` engine is reachable. Then build the pinned derivative and run
   real Chromium, egress/bypass, redirects, quarantine, timeout/cancel,
   crash/restart and cleanup probes. Record the actual output image ID.
3. **Linux:** the authorized host had neither Docker nor Podman, and lacked
   rootless prerequisites. Its `sudo -n` check required interactive
   authentication. Travis was asked to install `podman uidmap passt slirp4netns`
   via the signed distribution package manager and enter any sudo password
   himself. The SSH password was never supplied for elevation or persisted.
4. **Windows:** request Travis's instructions after Mac/Linux pass. No Windows
   qualification or all-platform support claim is made at this checkpoint.
5. Run the remaining real PF-26 isolation fixtures and final Fable review/fixes.
   Keep the implementation split into reviewable stages before landing it.
   Final acceptance and any required human/release evidence remain outstanding.

TUI applicability for S01 remains none: the backend is not exposed. S02/S03 own
interactive acquisition/setup flows and their actual-key qualification. No
live-repository, human, benchmark or release gate is being relabeled as passed.

Do not archive S01 or start its dependent sprints from this checkpoint.
