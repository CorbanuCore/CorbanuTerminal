# Explicit Astra subagent discovery — 2026-09-05

## Scope

Bounded fix restoring explicit runtime discovery, not changing spawn authorization.
Product specification: **Shipping MVP — LIVE**, “model-aware delegation,
durable mailboxes, supervision, resume, and recovery.”

Worktree: `/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile`.
Branch: `integration/reconcile-release-0.1.38`.
Base: `3bd1a684ed536849e8ca7f516886821461a28311`.
Implementation: `51185a24d404f98ce7f0dd5fc67e516deabee000`.

The catalog used automatic-allocation eligibility to hide explicitly usable
models, including Astra. Discovery now uses configured providers, the operator
allowlist and picker visibility. Entries without verified allocation economics
are labeled explicit-choice only. No invented prices, model-name exceptions,
regex routes, provider-policy changes or new wire/config schema.

The description remains capped at 32 entries and explains that omission is not
an authorization decision. Exact runtime requests remain subject to the existing
provider authorization, model resolution and reasoning validation.

## Verified candidate

- Version: 0.1.38 debug candidate; no release publication.
- Binary SHA-256: `ea7dceeeddbd0b3138568251a00f54d297ec6d953cf63cf8673c48ed2e90bc52`.
- Unchanged Code Mode host SHA-256: `2cfd7a6b2aaf58cb216e8dcf26c5e09b2db54be6a59f2d352bb7435ac71a426c`.
- Installed bundle: `/home/pfrpc/.local/share/corbanu-debug/0.1.38-astra-subagents-ea7dceeeddbd/bin/`.
- `corbanu-debug` now points to that bundle with the approved normal profile.
  Stable `corbanu`, credentials and existing tmux sessions are unchanged.
- Fresh human session: `tmux attach -t corbanu-astra-agents`, pane `%23`;
  Astra medium, YOLO, `~/repos`, with a shell underneath to avoid a dead pane on exit.

## Automated evidence

After final formatting, 183 focused Core tests pass and 22 Python harness tests
pass. The integration tests inspect actual outbound tool descriptions for Astra,
exercise explicit/model-only OpenAI child requests, and verify default,
OpenAI-only and deny-all provider policies. Generic unpriced-model fixtures,
hidden picker rows, catalog caps and retained native encryption are covered.
The Astra mock dispatch fixture disables Code Mode/Responses Lite for its mock
server; the real TUI runs below retain Astra's native configuration.

```sh
CARGO_TARGET_DIR=/tmp/corbanu-astra-review-phGuUE/codex-rs/target \
CORBANU_TEST_NO_NATIVE_KEYRING=1 just test -p codex-core --locked --offline \
  --retries 0 --test-threads 1 \
  -E 'test(multi_agents) | test(spawn_agent_description) | test(spec_plan) | test(multi_agent_resume)'
python3 -m unittest discover -s scripts -p 'test_*tui_acceptance.py'
```

Artifact root: `/tmp/corbanu-astra-subagents-oXAOm9/`.
Final Rust log: `core-tests-final-serial.log` (183/183; 3,275 filtered tests are
not passes). Earlier `core-tests-qualified.log` records a five-second worker-
completion timeout in the existing cold-resume fixture during concurrent live
tests; it is retained as a failure, not relabeled. That fixture passed both the
preceding complete run and final serial run. No timeout was increased or test
disabled. The initial `core-tests.log` also retains two corrected fixture/help-
text assertion failures. Build log: `build.log`.

## Real TUI evidence

Both applicable repositories used new disposable worktrees under the artifact
root: TensorCash base `dd6e92024254090de0f596b090bd5c74c4d97b90` and Isometric
Game base `59821b7a85524f186f946c4670480c7ee96483cb`. Both remain clean.

`scripts/subagent_tui_acceptance.py --allow-live --suite astra` ran the exact
installed binary in a real PTY with prompt text and Enter sent separately. Each
parent received a natural-language request for one explicitly selected Astra
child with fresh context and one child inheriting Astra and the parent's history.
No tool-adapter hints were included in the Astra selection prompt. Typed spawn
records prove the explicit provider/model override and native inherited route.

Each child executed repository inspection and returned a README fact. The parent
was cancelled with Escape, recovered, exited, restarted and followed up with the
same children, which executed `git status --short` and returned new answers.
All four children retained `openai/gpt-6-astra` and V2 through cold resume.

| Repository | Explicit Astra child | Inherited Astra child |
| --- | --- | --- |
| TensorCash | `01a07228-b5ea-72e1-b6cf-0a2751c06aa3` | `01a07228-d460-7f20-a0ca-4ac4204f3dc1` |
| Isometric Game | `01a07228-b0a8-7ce2-ab28-f181079d9a99` | `01a07228-d129-7303-9f1c-fa8e2fff070e` |

Final artifacts: `tensorcash-final/` and `isometricgame-final/`, including visible
checkpoints, actual keys, provider response IDs and exact child identities.
Together: **eight completed child turns, 20 child provider responses and 12 paired
child tool calls**, plus 25 parent responses. Parent responses copied into full-
history forks are excluded using parent turn/call identities, not counted as
child inference. The new full-history harness regression proves this exclusion.

The first diagnostics (`tensorcash-live/`, `isometricgame-live/`) returned actual
Astra child answers but timed out in the harness because it did not understand
copied ancestor metadata. Those runs remain failures; final acceptance used the
corrected harness and new parent/child sessions, not retroactive reclassification.

## Security and remaining release gates

Real-credential TUI runs use private sockets/log directories and `RUST_LOG=warn`
because of the documented trace/keyboard credential-logging risk; this is the
security exception to test-tui's trace default. No credentials are read or copied.
Named-human acceptance, full workspace tests, new platform qualification and
release benchmarks are not claimed. No release publication is part of this fix.
This bounded fix adds no plan or sprint allocation. Governance and portable-skill
checks pass. Existing release security/platform/benchmark gates remain unchanged.
