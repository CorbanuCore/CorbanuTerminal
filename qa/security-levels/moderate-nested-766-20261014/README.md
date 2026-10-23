# #766: nested `corbanu exec` under Moderate (tmux check, 2026-10-14)

Debug build of `mod-766-nested-moderate`, disposable home and `HOME`, GLM 5.3
Flash on `zai`, `CORBANU_TEST_NO_NATIVE_KEYRING=1`. Config Moderate with the
milestone flags (`security_levels`, `protected_mode_preflight`,
`secretless_agent_launch`, `isolated_credential_broker`,
`secret_output_gate`, `source_envelopes`, `url_destination_policy`,
`contained_external_agents`), approval `on-request`, default workspace-write
sandbox (no network for agent commands).

| Capture | What the agent ran | Result |
| --- | --- | --- |
| `t1-same-home.txt` | `corbanu exec … hi` in the same Corbanu home | Starts: "Security level Moderate is enforced because an agent command under Moderate started this run (…/.corbanu). It cannot read or write that Corbanu home inside the sandbox, so it runs in a Corbanu home of its own (…)". On main it stopped with "Error loading config.toml … Operation not permitted". |
| `t2-same-home-end.txt` | the same, last lines | Runs at Moderate up to the model request, which Moderate refuses because the credential broker (on under Moderate, D2) cannot start inside the sandbox. Fails closed. |
| `t4-other-home.txt` | `CORBANU_HOME=$PWD/other … HOME=$PWD corbanu exec …` | Held to Moderate: the origin is found through the Moderate registry. On main it ran at that home's level (Permissive). |
| `t5-refused.txt` | `corbanu app-server` (after approving, as vault-like access after untrusted content) | Refused, naming Moderate. |

Automated evidence: `codex-rs/cli/tests/nested_moderate.rs` (mock model, real
Seatbelt/bubblewrap) and `codex-rs/core/tests/suite/issue_766.rs`.
