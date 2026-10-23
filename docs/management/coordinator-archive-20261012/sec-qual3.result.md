## Final report

**What shipped:** PR [#242](https://github.com/CorbanuCore/CorbanuTerminal/pull/242) — **merged** to main (merge commit `344ac33018`). Issue [#239](https://github.com/CorbanuCore/CorbanuTerminal/issues/239) filed for the files-route read gap.

**Candidate:** `corbanu 0.1.48`, SHA-256 `1a123a79c4f9a81e6da5621b19907d3b9b6be0ae171a287662a31ff0221141c`, commit `64137b71894` (origin/main tip). All protection flags on: `isolated_credential_broker`, `secretless_agent_launch`, `secret_output_gate`, `url_destination_policy`, `protected_mode_preflight`, `source_envelopes`, `security_levels` (feature enabled; level not persisted — Permissive default).

**Gate evidence:**
- **Canary harness** (`scripts/security-credential-canary`): macOS 9/9 probes passed (0 leaks); Linux 9/9 probes passed (0 leaks, prior candidate — RTX box inaccessible via SSH for re-run). This is the authoritative deterministic test.
- **Direct sandbox probes** (`direct-probes.sh`): 7/8 blocked, 1 expected leak (files route — workspace-write allows full disk read, issue #239). Deterministic probes testing each protection mechanism directly (env stripping, ZAI key stripping, nested launch, echo-back, encodings, broker socket, vault auth-helper).
- **Route matrix** (`route-matrix.sh`): supplementary agent-mediated evidence. Model non-deterministic (GLM 5.2 sometimes refuses, sometimes runs commands). Two runs documented (v3 showed files leak, v4 showed model refusals).
- **Composition test**: pass (all flags on, no crash/deadlock, real task completed).
- **Tmux adversarial**: GLM 5.2 refused all routes (model refusals, not protection denials — documented honestly).
- **Demo videos**: [composition-hello](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-13-s07-pf13s07-composition-hello-64137b71894f-2026-10-06.mp4) (23s), [env-stripping](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-13-s07-pf13s07-env-stripping-64137b71894f-2026-10-06.mp4) (24s) — published.
- **Independent review**: claude-opus-5-5-plan, read-only sandbox. Initial verdict: CHANGES REQUIRED (vacuous BLOCKED results, hand-edited JSON, overstated README). All findings addressed: direct probes added, README rewritten with honest results, route matrix documented as supplementary, SHA typo fixed, Linux caveat corrected.
- **Tests**: `cargo test -p codex-security-policy -p codex-vault` (59 passed), `cargo test -p codex-core --lib credential` (29 passed), `cargo test -p codex-cli --test vault` (4 passed), `cargo test -p codex-process-hardening` (6 passed).

**Findings:**
- **Issue #239** (known limitation, not a regression): workspace-write sandbox allows reading credential files from `$HOME` (`has_full_disk_read_access()` returns `true` for all sandbox modes). Read protection requires Aggressive/Moderate security level, not just the feature flag. No code change in this sprint (qualification only).
- **Claude pane gap**: PF-27-S02 documents Claude panes as "Not covered (Remaining)". PF-23-S01 added `contained_external_agents` behind a flag, but default still passes `ANTHROPIC_API_KEY`. Pre-existing known gap.
- **Output gate limitation**: the canary read from a file appeared unredacted in exec tool output (sent to model provider). The `secret_output_gate` only scrubs registered env values, not arbitrary file contents. Documented in README.

**Open:**
- Linux canary harness re-run on the current candidate when RTX box is accessible (prior run used commit `e72563e5f7`).
- Issue #239 to be addressed when Aggressive/Moderate read sandboxing ships.
- Sprint archived at the Aggressive milestone (not this sprint).

**Disk hygiene:** CARGO_TARGET_DIR and worktree removed.