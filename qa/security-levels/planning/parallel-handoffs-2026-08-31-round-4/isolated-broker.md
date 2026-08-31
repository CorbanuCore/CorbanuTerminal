# PF-27-S04 isolated-broker handoff

Owner, branch, worktree, base and literal scope are authoritative in the sprint
front matter. Shared Core registration, root manifests/locks, active-plan and
archive edits belong to the integration owner.

## Deliverable

Move raw credential resolution and substitution into a bounded trusted broker
process. Agent-accessible Core/model/proxy workers receive opaque references and
typed operations only—never a generic resolve-to-string API. Authenticate OS
peer plus session/task/run, reject replay/malformed frames, and close capabilities
on cancellation, replacement, revocation, broker death and restart.

Use the PF-27-S03 platform candidates for construction:

- Linux dedicated UID/service boundary;
- macOS launchd/XPC helper boundary; and
- Windows service SID/AppContainer with authenticated named pipe.

These are construction targets, not accepted eligibility claims. If a local
mechanism cannot be qualified, return typed unavailable and keep protected
activation off. Do not weaken the platform contract to turn same-user IPC into
a pass.

Cover fresh connections after same-run re-registration, cached TLS handlers,
open-channel revocation, uploads, broker restart, old handles, cross-run theft,
wrong peers, bounded resources and concurrent revoke. Preserve PF-13's exact
OpenAI host/method/path adapter and PF-41 durable event semantics.

## Scope discipline

Stay inside the secret-broker, credential-broker, Vault capability, new
broker-client/config leaf files and evidence listed in the sprint. Do not edit
`core/src/security/mod.rs`, Core/root Cargo/Bazel/locks or PF-22 files. The
integration owner registers and activates the Core seam only after PF-22 lands.

## Proof

- fix/format and focused/full secret-broker, network-proxy and Vault suites
- focused Core broker-client tests after integration-owner registration
- security-audit consumed-contract tests
- platform probe self-test/schema and Bazel unit target
- plan/sprint governance and diff check
- supporting TMUX smoke and TMUX/Corbanu/Claude Opus 5 Max review

Do not attempt final Linux/Windows actual-launch qualification until instructed;
the user will switch tailnets at that stage. Record local evidence under
`qa/security-levels/sprints/PF-27-S04/`, commit the handback branch, and do not
merge or push `main`.

