# Task Node evidence record: P0SEC-TN-01

Compile the P0SEC-TN-01 Broker-Lane Sprint Evidence Record. Task `task_169f5d2052343f51777060e4ebb34e69`, request `req_3ec829e5ace25114ce8152ea021da7122c1a598ea0b7750ac0395258fa8045a2`. Every PR below is merged to `main` in
CorbanuCore/CorbanuTerminal. Each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main`
at `bb609b449a` (2026-10-07). The quoted blocks are copied verbatim from the sprint and gate records (links
re-pointed to this file's location).

Per-sprint gate (Travis, 2026-10-06): focused `just test`, a GLM 5.2 tmux run, one independent Opus 5.5 High
review and SOP demo videos, then merge behind the feature flag. **Not claimed:** the milestone code-blind VM run,
human sign-off, flag removal and the P1 hardening plan. Carried-forward items are recorded as open, not done.

| Sprint | PRs (merge commit on main) | Flag | Record status |
| --- | --- | --- | --- |
| [PF-27-S04](../../sprints/archive/p0-security-levels/pf-27-s04-isolated-credential-broker.md) Isolated credential broker process | #180 (`13cf4a2d0c`) | `isolated_credential_broker` | completed (archived) |
| [PF-27-S02](../../sprints/archive/p0-security-levels/pf-27-s02-secretless-agent-launch.md) Secretless agent launch and broker containment | #191 (`699bd4a82f`) | `secretless_agent_launch` | completed (archived) |
| [PF-27-S05](../../sprints/archive/p0-security-levels/pf-27-s05-model-client-auth-broker.md) Model-client auth held by the broker | #229 (`c5bdadd322`), #237 (`5656e997d2`) | `broker_model_auth` | completed (archived) |

PR links:

- #180: https://github.com/CorbanuCore/CorbanuTerminal/pull/180
- #191: https://github.com/CorbanuCore/CorbanuTerminal/pull/191
- #229: https://github.com/CorbanuCore/CorbanuTerminal/pull/229
- #237: https://github.com/CorbanuCore/CorbanuTerminal/pull/237

## PF-27-S04: Isolated credential broker process

Summary: tests `just test -p codex-secret-broker pf_27_s01` (24) and `just test -p codex-core pf_27_s01` (2); affected crates 334 passed, related Core 88 passed. TUI run: GLM 5.2 tmux cases V0-V3. Review: Opus 5.5 High: APPROVE WITH NITS, no P0/P1.

Gate record: [qa/security-levels/sprints/PF-27-S04/isolated-broker-20261006/README.md](../../../qa/security-levels/sprints/PF-27-S04/isolated-broker-20261006/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-27-s04-isolated-credential-broker.md`, Verification:

> - [x] Independent isolated execution: not a per-sprint gate under decision 5; it runs at the "Aggressive ships", "Moderate ships" and flag-removal milestones.
> - [x] `just fix` and `just fmt` on the touched crates (evidence README).
> - [x] Focused: `just test -p codex-secret-broker pf_27_s01` (24) and `just test -p codex-core pf_27_s01` (2); both ran.
> - [x] Integration: affected crates 334 passed, related Core 88 passed; Cargo lock updated in PR #180.
> - [x] TUI applicability: none; the GLM 5.2 tmux run covers the agent-visible behaviour.
> - [x] Candidate, commands, expected/actual outcomes and recording digests recorded in the evidence README; synthetic credentials only.

Demo videos (3, index [qa/demos/index/PF-27-S04.md](../../../qa/demos/index/PF-27-S04.md)):

- `pf27-broker-substitution`: Isolated credential broker: the agent holds a dummy, the broker adds the real token: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s04-pf27-broker-substitution-03e35d2e744f-2026-10-05.mp4
- `pf27-broker-direct-call-denied`: Isolated credential broker: nothing but Core can call the broker socket: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s04-pf27-broker-direct-call-denied-1a4370992f5a-2026-10-05.mp4
- `pf27-broker-crash-fails-closed`: Isolated credential broker: a dead broker fails closed: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s04-pf27-broker-crash-fails-closed-1a4370992f5a-2026-10-05.mp4

Carried forward (open, not claimed): Model-client auth to PF-27-S05; broker containment to PF-27-S02; Windows to PF-27-S06 (P1); production revocation trigger to PF-23-S03.

## PF-27-S02: Secretless agent launch and broker containment

Summary: tests `just test -p codex-core pf_27_s02` (12) plus `pf_27` tests in protocol, network-proxy and process-hardening; affected crates 722 passed, core subset 853 passed (one unrelated rmcp keyring-fixture failure recorded). TUI run: GLM 5.2 TUI runs. Review: Opus 5.5 High: changes required, then APPROVE WITH NITS after fixes.

Gate record: [qa/security-levels/sprints/PF-27-S02/README.md](../../../qa/security-levels/sprints/PF-27-S02/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-27-s02-secretless-agent-launch.md`, Verification:

> - [x] Linux and Bazel CI on the PR: all checks green at merge (PR #191).
> - [x] `just fix -p` on every touched crate and `just fmt`; final diff inspected.
> - [x] Focused: `just test -p codex-core pf_27_s02` (12) and the `pf_27` tests in protocol, network-proxy and process-hardening, all passing.
> - [x] Integration: affected crate suites (722 passed; core subset 853 passed; one unrelated rmcp keyring-fixture failure recorded).
> - [x] TUI applicability: GLM 5.2 TUI runs and five SOP videos ([index](../../../qa/demos/index/PF-27-S02.md)).
> - [x] Candidate, commands and outcomes recorded; synthetic credentials only.
> - [x] Windows verification moved with [PF-27-S06](../../sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md) to the P1 hardening plan.

Demo videos (5, index [qa/demos/index/PF-27-S02.md](../../../qa/demos/index/PF-27-S02.md)):

- `pf27s02-baseline-flag-off`: Control: with secretless_agent_launch off, agent commands can read the vault store and sign-in file: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-baseline-flag-off-03c9dfd9716d-2026-10-06.mp4
- `pf27s02-broker-contained`: Secretless agent launch: the credential broker runs contained, in a private socket directory: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-broker-contained-03c9dfd9716d-2026-10-06.mp4
- `pf27s02-protected-paths`: Secretless agent launch: vault, sign-in file, shell snapshots and Core's environment are out of reach: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-protected-paths-03c9dfd9716d-2026-10-06.mp4
- `pf27s02-secretless-env`: Secretless agent launch: agent commands get no raw secrets, and the broker still works: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-secretless-env-03c9dfd9716d-2026-10-06.mp4
- `pf27s02-unsandboxed-refused`: Secretless agent launch: a command that would run unsandboxed is refused, with the reason: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-unsandboxed-refused-03c9dfd9716d-2026-10-06.mp4

Carried forward (open, not claimed): MCP/hooks/`!` shell shown as not contained (PF-41-S01); Claude panes to TUI lane (#220, issue #218); Windows to PF-27-S06 (P1).

## PF-27-S05: Model-client auth held by the broker

Summary: tests affected crates 1312 passed; core subsets 1099 passed (2 unrelated `suite::client::skills_*` failures); Linux clippy `-D warnings` clean and Linux `pf_27_s05` tests pass on the RTX box. TUI run: GLM 5.2 tmux runs, keyring-isolated. Review: Opus 5.5 High reviews 3-4 changes requested and fixed; review 5 APPROVE.

Gate record: [qa/security-levels/sprints/PF-27-S05/README.md](../../../qa/security-levels/sprints/PF-27-S05/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-27-s05-model-client-auth-broker.md`, Verification:

> - [x] `just fix -p` and `just fmt`. Affected crates: 1312 passed; core subsets: 1099 passed. The 2 `suite::client::skills_*`
>   failures are unrelated: real `~/.agents/skills` leak into the test. Linux clippy `-D warnings` is clean on the RTX box,
>   and the Linux `pf_27_s05` tests pass.
> - [x] GLM 5.2 tmux runs as SOP videos, keyring-isolated ([index](../../../qa/demos/index/PF-27-S05.md)).
> - [x] Opus 5.5 High reviews 3 and 4 (changes requested, fixed); see [evidence](../../../qa/security-levels/sprints/PF-27-S05/README.md).
> - [x] Final re-review (review 5): APPROVE.

Verbatim, gate record `qa/security-levels/sprints/PF-27-S05/README.md`:

> - Linux clippy (`-D warnings`) on the RTX box for the six changed crates: clean.

Demo videos (4, index [qa/demos/index/PF-27-S05.md](../../../qa/demos/index/PF-27-S05.md)):

- `pf27s05-model-key-brokered`: Model-client auth in the broker: Core's Z.AI key is held by the isolated broker, which signs every model request: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s05-pf27s05-model-key-brokered-ead7691092c1-2026-10-06.mp4
- `pf27s05-broker-death-fails-closed`: Model-client auth in the broker: when the broker dies, model requests fail instead of using the key directly: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s05-pf27s05-broker-death-fails-closed-ead7691092c1-2026-10-06.mp4
- `pf27s05-env-key-handed-over`: Model-client auth in the broker: Core hands ZAI_API_KEY to the broker at startup and overwrites it in its own launch environment: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s05-pf27s05-env-key-handed-over-b0e6785fb92e-2026-10-06.mp4
- `pf27s05-vault-key-in-broker`: Model-client auth in the broker: a Z.AI key stored in the encrypted vault is read inside the broker; Core never decrypts it: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s05-pf27s05-vault-key-in-broker-b0e6785fb92e-2026-10-06.mp4

Carried forward (open, not claimed): Known limits: other features still open the vault in-process; env scrubbing at session start.
