# Task Node evidence record: P0SEC-TN-02

Compile the P0SEC-TN-02 Secret Output Gate Sprint Evidence Record. Task `task_1e60ecf27d3611270ba86aa46343226b`, request `req_a707b82b15b32003ee6ecf54448460e1354261dd7d85a31a8225b2b7c0a87ad3`. Every PR below is merged to `main` in
CorbanuCore/CorbanuTerminal. Each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main`
at `bb609b449a` (2026-10-07). The quoted blocks are copied verbatim from the sprint and gate records (links
re-pointed to this file's location).

Per-sprint gate (Travis, 2026-10-06): focused `just test`, a GLM 5.2 tmux run, one independent Opus 5.5 High
review and SOP demo videos, then merge behind the feature flag. **Not claimed:** the milestone code-blind VM run,
human sign-off, flag removal and the P1 hardening plan. Carried-forward items are recorded as open, not done.

| Sprint | PRs (merge commit on main) | Flag | Record status |
| --- | --- | --- | --- |
| [PF-28-S01](../../sprints/archive/p0-security-levels/pf-28-s01-central-secret-output-gate.md) Central secret and protected-output gate | [#208](https://github.com/CorbanuCore/CorbanuTerminal/pull/208) (`24a57e38c4`) | `secret_output_gate` | completed (archived) |
| [PF-28-S02](../../sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md) Reflected-secret response scrubbing and credential binding | [#216](https://github.com/CorbanuCore/CorbanuTerminal/pull/216) (`b9f215ec50`), [#225](https://github.com/CorbanuCore/CorbanuTerminal/pull/225) (`8cf46179f5`) | `secret_output_gate` | merged behind flag; record current (milestone items open) |

## PF-28-S01: Central secret and protected-output gate

Summary: tests `just test ... -E 'test(pf_28_s01)'` across secret-broker, vault, login, otel, core: 38 passed. TUI run: four GLM 5.2 TUI runs. Review: Opus 5.5 High, rounds until approved, dispositioned.

Gate record: [qa/security-levels/sprints/PF-28-S01/README.md](../../../qa/security-levels/sprints/PF-28-S01/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-28-s01-central-secret-output-gate.md`, Verification:

> - [x] `just fix -p` on every touched crate, then `just fmt`; final diff inspected.
> - [x] Focused: `just test -p codex-secret-broker -p codex-vault -p codex-login -p codex-otel -p codex-core -E 'test(pf_28_s01)'`: 38 passed. (`codex-secrets` gained no tests: its sanitizer is not used by the gate.)
> - [x] Integration: affected crate suites; results in the evidence README.
> - [x] TUI applicability: four GLM 5.2 runs recorded as SOP videos ([index](../../../qa/demos/index/PF-28-S01.md)).
> - [x] Candidate, commands and outcomes recorded; synthetic canaries only.
> - [x] Milestone qualification (isolated code-blind VM run, human sign-off) runs at "Moderate ships", not per sprint (decision 5).

Verbatim, gate record `qa/security-levels/sprints/PF-28-S01/README.md`:

> - **Gated sinks.** Model requests (turns and compaction); recorded history (tool results and model output);
>   rollout transcript; client events, both at `send_event` and again at delivery (`Codex::next_event`, which covers
>   MCP startup and elicitation events sent on a cloned sender); tool-result, prompt and error telemetry; the TUI log
>   file, the feedback ring, the log database and prompt history. Rollout traces and shell snapshots are off while
>   armed. If policy pins snapshots on, the session does not start. Memory summaries that would carry a value are
>   refused.

Demo videos (4, index [qa/demos/index/PF-28-S01.md](../../../qa/demos/index/PF-28-S01.md)):

- `pf28s01-tool-output-gated`: Secret output gate: a managed secret in command output is removed, in every encoding: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s01-pf28s01-tool-output-gated-9261bd416f0d-2026-10-06.mp4
- `pf28s01-persistence-clean`: Secret output gate: transcripts, logs and history never store a managed secret: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s01-pf28s01-persistence-clean-9261bd416f0d-2026-10-06.mp4
- `pf28s01-split-output`: Secret output gate: a secret split across output chunks is still removed whole: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s01-pf28s01-split-output-9261bd416f0d-2026-10-06.mp4
- `pf28s01-baseline-flag-off`: Control: with secret_output_gate off, the transcript and logs store the secret: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s01-pf28s01-baseline-flag-off-9261bd416f0d-2026-10-06.mp4

Carried forward (open, not claimed): Decode-and-rescan, reflected values and gate carry-overs to PF-28-S02; financial derived views to PF-39-S01 (P1).

## PF-28-S02: Reflected-secret response scrubbing and credential binding

Summary: tests `just test -p codex-secret-broker -p codex-network-proxy -p codex-rmcp-client`: 585 of 586 (the one failure also fails on clean main); core/login/vault/otel subsets 146 passed; Linux clippy clean. TUI run: four GLM 5.2 runs. Review: Opus 5.5 High, rounds until approved, dispositioned (incl. pinning review for #225).

Gate record: [qa/security-levels/sprints/PF-28-S02/README.md](../../../qa/security-levels/sprints/PF-28-S02/README.md).

Verbatim, sprint record `docs/sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md`, Verification:

> - [x] `just fix -p` on secret-broker, network-proxy, rmcp-client (core: existing PF-30-S03 `expect` stops clippy, unrelated), then `just fmt`; final diff inspected. Linux clippy (`-D warnings`) clean on the RTX box for the three crates.
> - [x] Focused: `just test -p codex-secret-broker -p codex-network-proxy -p codex-rmcp-client`: 585 of 586; the one failure (`auto_store_remains_pinned_across_session_recovery`, native keyring in the test fixture) fails the same on clean main.
> - [x] Integration: core, login, vault and otel subsets (network proxy, credentials, PF-27/28/33, schema, disclosure, redaction): 146 passed.
> - [x] TUI applicability: four GLM 5.2 runs recorded as SOP videos ([index](../../../qa/demos/index/PF-28-S02.md)).
> - [x] Candidate, commands and outcomes recorded; synthetic canaries only.
> - [x] PR #216 checks green; merged to main as `b9f215ec50` behind `secret_output_gate`.
> - [ ] Milestone qualification (isolated code-blind VM run, human sign-off) when Moderate ships.

Demo videos (5, index [qa/demos/index/PF-28-S02.md](../../../qa/demos/index/PF-28-S02.md)):

- `pf28s02-reflected-credential`: Reflected-secret scrubbing: a host that echoes the injected credential returns it redacted: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s02-pf28s02-reflected-credential-10357ba50c80-2026-10-06.mp4
- `pf28s02-reflected-baseline-flag-off`: Control: with secret_output_gate off, the echoed credential reaches the agent: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s02-pf28s02-reflected-baseline-flag-off-10357ba50c80-2026-10-06.mp4
- `pf28s02-credential-path-bound`: Credential binding: the broker sends a credential only to paths its provider uses: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s02-pf28s02-credential-path-bound-10357ba50c80-2026-10-06.mp4
- `pf28s02-wrapped-encodings`: Secret output gate: a secret wrapped across lines or encoded twice is still removed: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s02-pf28s02-wrapped-encodings-10357ba50c80-2026-10-06.mp4
- `pf28s02-brokered-pinned`: Brokered credentials are pinned: the broker connects only to the DNS answers Core's guard checked: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s02-pf28s02-brokered-pinned-ea006ee308e4-2026-10-06.mp4

Carried forward (open, not claimed): Latency targets need product authority; per-type text scrubbing and per-session stream state carried; recorded limits.
