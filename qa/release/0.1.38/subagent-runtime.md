# Native subagent runtime repair — 2026-09-05

## Outcome and scope

Product initiative PF-55-S04 under the active unified-provider-auth plan.
Product specification: **Shipping MVP — LIVE**, “model-aware delegation,
durable mailboxes, supervision, resume, and recovery.” No release publication.

The reported refusal exposed only Sol/Terra. The generated catalog incorrectly
treated a child's model-preferred orchestration version as spawn authorization:
the account catalog advertises Luna as V1, while Kimi has no engine preference.
The first live diagnostic then proved a second defect: the only OpenAI tool
surface with exact model/provider fields rejected OpenAI recipients. Its native
reserved counterpart has no model-selection fields, creating a dead end.

The repair separates child model selection from the inherited V2 engine and
admits explicitly plaintext assignments through the existing typed adapter,
including OpenAI recipients. It preserves the operator provider allowlist,
reasoning validation, role/permission rules, bounded 32-model descriptions,
native encrypted messages and refusal to send native ciphertext to other
providers. There are no model-name exceptions, metadata rewrites or regex routes.

## Candidate identity

- Branch: `integration/reconcile-release-0.1.38`.
- Worktree: `/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile`.
- Base: `43f4f187ba585e231b0bafed2bbcd9d9b4bffa54`.
- Runtime/test implementation: `e7cdb94359a7bdedeb6b0abdf2f17f09823d08e1`.
- Binary SHA-256: `bdc666c3098d48030e8474173ae53b169708a803dbac1b92088c44ce42e88ff1`.
- Code Mode host: `2cfd7a6b2aaf58cb216e8dcf26c5e09b2db54be6a59f2d352bb7435ac71a426c`.
- Installed bundle: `/home/pfrpc/.local/share/corbanu-debug/0.1.38-subagents-bdc666c3098d/bin/`.
- `corbanu-debug` uses the approved normal `/home/pfrpc/.corbanu` profile, not
  copied credentials. Stable `corbanu` and the older human session are unchanged.
- New human session: `tmux attach -t corbanu-agents`; Astra medium, YOLO,
  `~/repos`, with an interactive shell underneath so normal exit is not a dead pane.

Upstream: retained baseline `ba6cf9c69277caec51a4c12c5b7401a9920930e0` and the
previously reconciled Astra contract from official Codex `rust-v0.153.4`.
No upstream merge, wire-version change, credential-format change or release tag.

## Automated evidence

After `just fmt`, 180 selected Core tests passed, including actual outbound
OpenAI child requests through both model-only and exact provider/model selection,
mixed-version catalog exposure, deny-all/OpenAI-only provider policies, reserved
schemas, native encryption, interruption, mailboxes, role precedence and resume.
The final run used one test worker after an earlier parallel resume fixture
timed out. Filtered tests are not passes; full workspace tests were not run.

```sh
CARGO_TARGET_DIR=/tmp/corbanu-astra-review-phGuUE/codex-rs/target \
CORBANU_TEST_NO_NATIVE_KEYRING=1 just test -p codex-core --locked --offline \
  --retries 0 --test-threads 1 \
  -E 'test(multi_agents) | test(spawn_agent_description) | test(spec_plan) | test(multi_agent_resume)'
python3 -m unittest discover -s scripts -p 'test_*tui_acceptance.py'
```

Harness tests: 19/19. Governance, portable skills and `git diff --check` pass.
Logs: `/tmp/corbanu-subagent-live-MUKR0I/core-tests-qualified.log` and
`build-adapter.log`. Existing unrelated build warnings remain.

## Actual TUI evidence

Both repositories are applicable because this changes delegated project work.
Pinned disposable bases: TensorCash `dd6e92024254090de0f596b090bd5c74c4d97b90`;
Isometric Game `59821b7a85524f186f946c4670480c7ee96483cb`.
Worktrees and private evidence live under `/tmp/corbanu-subagent-live-MUKR0I/`.

The opt-in `scripts/subagent_tui_acceptance.py` reuses the real PTY driver,
sends text and Enter separately, and requires actual response IDs from exactly
`openai/gpt-5.6-luna` and `kimi-code/k3`. Each child executes repository inspection
and returns a README fact. The driver cancels the parent's execution, verifies
recovery, exits/restarts the parent and follows up on the same two durable child
IDs, requiring new tool results and completed provider responses. Echoed prompts,
unrelated rollouts, replacement children and model reroutes cannot pass.

Complete preliminary runs: `tensorcash-adapter/` and `isometricgame-adapter/`.
Together: eight completed child turns, 24 child responses and 15 paired child
tool calls; both runtimes retained V2. Installed-binary final rerun evidence is
recorded separately in `tensorcash-final/` and `isometricgame-final/`.

Both installed-binary reruns PASS after the final formatting pass: eight child
turns, **23 child responses and 14 paired child tool calls**, plus 31 parent
responses. Every child retains its exact provider/model, V2 engine and thread ID
through cold parent restart. Both disposable worktrees remain clean.

| Final repository | Luna child | Kimi child |
| --- | --- | --- |
| TensorCash | `01a071b9-dfe4-7a51-874e-1ff5f059244b` | `01a071b9-fe6d-7c13-a136-a2f118c8927f` |
| Isometric Game | `01a071b9-e5d8-7040-9836-5d7d2c599716` | `01a071ba-00b7-7bc2-82c8-f2c0a8a50101` |

The new human `corbanu-agents` session also passed a plain-language request to
test Luna and Kimi K3 on `17 times 19`, without tool/adapter instructions. Both
returned 323, using Luna medium and Kimi Code K3 high. This is automated human-
handoff evidence, not named-human acceptance. The running executable's hash was
checked against the installed candidate; the original `corbanu-test` pane stays live.
Human root thread: `01a071b9-bbdf-7e32-8493-b0622fbc7ed8`.

The failed first diagnostic, `tensorcash-live/`, is retained as failure, not
acceptance. Its Kimi `medium` request was a harness mistake correctly refused by
effort validation; successful Kimi runs use `high` (supported: low/high/max).

Real-credential runs use `RUST_LOG=warn` and private log directories because of
the previously documented trace/keyboard credential-logging risk. This is the
security exception to the test-tui skill's trace default. No credentials were
read, copied into tests or entered through automation. Synthetic fixtures are
separate. Generated compiler cache was moved from the task's build directory to
`/dev/shm/corbanu-subagent-cache-4vK0Zy/incremental` to relieve disk pressure; it is
rebuildable temporary data, not user source or state.
After verification, the 65 GiB temporary incremental cache and its symlink were
removed with no Rust builds active. Compiled binaries and dependency artifacts
remain; Rust regenerates the incremental cache on demand. No profile data changed.

## Remaining release gates

Named-human acceptance is pending. No claim of new Windows/macOS/remote-executor
qualification, full workspace coverage or competitor benchmark results. The
unchanged release-level security/benchmark/platform gates remain separate from
this proven debug-runtime repair. No release was published.
