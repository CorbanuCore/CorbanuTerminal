# Task Node evidence record: P0SEC-TN-06

Compile the P0SEC-TN-06 Security Levels Enforcement Evidence Record. Task `task_22ba027b6f6a85dcd4cb47bf29888891`, request `req_e2242c029b999635535c05d33b5ec720f7aec22d05fb526be868f8fecf0db588`. Every PR below is merged to `main` in
CorbanuCore/CorbanuTerminal. Each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main`
at `bb609b449a` (2026-10-07). The quoted blocks are copied verbatim from the sprint and gate records (links
re-pointed to this file's location).

Per-sprint gate (Travis, 2026-10-06): focused `just test`, a GLM 5.2 tmux run, one independent Opus 5.5 High
review and SOP demo videos, then merge behind the feature flag. **Not claimed:** the milestone code-blind VM run,
human sign-off, flag removal and the P1 hardening plan. Carried-forward items are recorded as open, not done.

| Sprint | PRs (merge commit on main) | Flag | Record status |
| --- | --- | --- | --- |
| [PF-23-S01](../../sprints/archive/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md) Moderate ingress and disclosure enforcement | #223 (`f79887d182`), #233 (`64137b7189`) | `security_levels` | completed (archived) |
| [PF-23-S02](../../sprints/archive/p0-security-levels/pf-23-s02-aggressive-deny-and-grant-enforcement.md) Aggressive deny and grant enforcement | #243 (`6b1c8b8873`), #250 (`a230f20821`) | `security_levels` | completed (archived) |
| [PF-23-S03](../../sprints/archive/p0-security-levels/pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) Downgrade, restart and inheritance enforcement | #246 (`95b5f34a55`), #247 (`242f4d3bde`), #248 (`7fc064e593`), #249 (`4f09d7af99`), #256 (`213698e607`) | `security_levels` | completed (archived) |

PR links:

- #223: https://github.com/CorbanuCore/CorbanuTerminal/pull/223
- #233: https://github.com/CorbanuCore/CorbanuTerminal/pull/233
- #243: https://github.com/CorbanuCore/CorbanuTerminal/pull/243
- #250: https://github.com/CorbanuCore/CorbanuTerminal/pull/250
- #246: https://github.com/CorbanuCore/CorbanuTerminal/pull/246
- #247: https://github.com/CorbanuCore/CorbanuTerminal/pull/247
- #248: https://github.com/CorbanuCore/CorbanuTerminal/pull/248
- #249: https://github.com/CorbanuCore/CorbanuTerminal/pull/249
- #256: https://github.com/CorbanuCore/CorbanuTerminal/pull/256

## PF-23-S01: Moderate ingress and disclosure enforcement

Summary: tests slice 1 full core 3,841/3,844; slices 2-3 focused 145 pass + memories-write 45 pass, full core 3,881/3,884; Linux clippy on the RTX box. TUI run: GLM 5.2 runs. Review: Opus 5.5 High: slice 1 round 5 APPROVE; slices 2-3 round 3 APPROVE.

Gate record: [qa/security-levels/sprints/PF-23-S01/slice-1-gate.md](../../../qa/security-levels/sprints/PF-23-S01/slice-1-gate.md), [qa/security-levels/sprints/PF-23-S01/slices-2-3-gate.md](../../../qa/security-levels/sprints/PF-23-S01/slices-2-3-gate.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md`, Verification:

> - [x] Per slice: `just fix -p codex-core -p codex-memories-write`, `just fmt`, focused `pf_23_s01`,
>   `memory_stage_one`, `pf_30_s0`, `just test -p codex-memories-write`, full `just test -p codex-core`, Linux clippy
>   on the RTX box, GLM 5.2 videos, Opus 5.5 High review (slices 2-3: three rounds, APPROVE).

Verbatim, gate record `qa/security-levels/sprints/PF-23-S01/slice-1-gate.md`:

> - **Focused tests:** `just test -p codex-core pf_23_s01` (unit: matrix incl. every built-in with all features,
>   MCP classification with paraphrases and adjacent read names, typed input, the typed-input window, outbound and
>   value-transfer cases from three review rounds, bounded wrapper positions; suite: protected MCP call asks /
>   cancel / refused under `never` / ordinary and Permissive unchanged, typing a vault command asks / refused /
>   split across two writes refused / ordinary typing unchanged, a code-mode cell's later vault call refused while
>   Permissive runs it). `pf_30_s0` still passes 102.
>
> - **Full crate:** `just test -p codex-core` at `8037f122a9` and again at `eac4be8903`: 3,841 of 3,844 pass; the
>   3 failures are the known
>   baselines (`skills_append_to_developer_message`,
>   `skills_use_aliases_in_developer_message_under_budget_pressure`,
>   `remote_compact_trim_estimate_uses_session_base_instructions`).
>
> - **GLM 5.2 tmux runs and videos** at `eac4be8903`, in [qa/demos/index/PF-23-S01.md](../../../qa/demos/index/PF-23-S01.md):
>   a code-mode MCP call with `~/.docker/config.json` asks (Security level question with the arguments); an
>   ordinary code-mode MCP call runs unasked; typing `corbanu vault --help` into a running bash asks; one code-mode
>   script that reads notes.txt then runs the vault command is refused with approvals off; `curl -T notes.txt` to
>   a remote host and `solana transfer` each stop at an approval naming the kind.
>   GLM refused earlier takes itself (echoing `~/.ssh/id_ed25519`; twice the upload to an `example.com` host,
>   calling it exfiltration); one take ended with the model paraphrasing the refusal past the wait pattern. The
>   specs were reworded, not the product.

Verbatim, gate record `qa/security-levels/sprints/PF-23-S01/slices-2-3-gate.md`:

> - **Focused:** `just test -p codex-core pf_23_s01 memory_stage_one pf_30_s0 mcp_openai_file` (145 pass) and
>   `just test -p codex-memories-write` (45 pass). New: read-denial unit tests (home default-deny, full access, keep
>   roots, Claude dir, file tools after taint, upload reads through symlinks), stage-one lineage/foreign key/long
>   rollout/redaction/exact-message tests, a Moderate worker test that sends only labelled text and refuses another
>   session's file, and macOS suite tests where the sandbox (not the text check) denies a run-time-built home path,
>   a model-chosen working folder inside a denied path keeps the denial, and approval lifts it under Moderate only.
>
> - **Full crate:** `just test -p codex-core`: 3,855 of 3,858 at `18365ca3c5` and 3,881 of 3,884 at `eeb096de1`
>   (merged with main); the 3 failures are the known baselines
>   (`skills_append_to_developer_message`, `skills_use_aliases_in_developer_message_under_budget_pressure`,
>   `remote_compact_trim_estimate_uses_session_base_instructions`).
>
> - **Review:** Opus 5.5 High through `corbanu exec`, `.codex-work/workers-20261002/pf23s01-review{7,8,9}/`. Round 1
>   changes (model-chosen working folder lifted a denial; file tools unprotected; long rollouts always refused; host
>   could shape the input); round 2 changes (Codex Apps uploads; quadratic fit; grants as keep roots; redaction could
>   cut a label); round 3 APPROVE at `a58a5e18e6`, non-blocking items below; its fit-loop Low is fixed after.
>
> - **GLM 5.2 videos** at `a58a5e18e6`, in [qa/demos/index/PF-23-S01.md](../../../qa/demos/index/PF-23-S01.md): a run-time
>   home path read after untrusted content gets "Operation not permitted" with no prompt; under Moderate the approved
>   literal read goes through; the memory worker sends an aged session's labelled rollout under Moderate.

Demo videos (9, index [qa/demos/index/PF-23-S01.md](../../../qa/demos/index/PF-23-S01.md)):

- `pf23s01-mcp-needs-human`: An MCP call reaching a credential asks the human after untrusted content: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-mcp-needs-human-eac4be890364-2026-10-06.mp4
- `pf23s01-mcp-ordinary-unchanged`: An ordinary MCP call still runs unasked after untrusted content: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-mcp-ordinary-unchanged-eac4be890364-2026-10-06.mp4
- `pf23s01-typed-vault-needs-human`: Typing a vault command into a running shell asks the human: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-typed-vault-needs-human-eac4be890364-2026-10-06.mp4
- `pf23s01-upload-needs-human`: Uploading a local file asks the human after untrusted content: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-upload-needs-human-eac4be890364-2026-10-06.mp4
- `pf23s01-transfer-needs-human`: A token transfer asks the human after untrusted content: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-transfer-needs-human-eac4be890364-2026-10-06.mp4
- `pf23s01-code-mode-cell-tainted`: A code-mode script cannot act on tool output it just read: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-code-mode-cell-tainted-eac4be890364-2026-10-06.mp4
- `pf23s01-hidden-home-read-denied`: After untrusted content, the sandbox blocks reading the Corbanu home: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-hidden-home-read-denied-a58a5e18e699-2026-10-06.mp4
- `pf23s01-approval-lifts-under-moderate`: Under Moderate, approving the exact command lets that one read through: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-approval-lifts-under-moderate-a58a5e18e699-2026-10-06.mp4
- `pf23s01-moderate-memory-labelled`: Memory summarisation runs under Moderate, on labelled input: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-moderate-memory-labelled-a58a5e18e699-2026-10-06.mp4

Carried forward (open, not claimed): Aggressive grants and command-text gaps to PF-23-S02; consolidation and session level to PF-23-S03.

## PF-23-S02: Aggressive deny and grant enforcement

Summary: tests focused core set 75/75; sandboxing `renaming_a_folder` 1/1; full core 3,897/3,902 (failures recorded); Linux clippy on the RTX box. TUI run: GLM 5.2 runs. Review: Opus 5.5 High: five rounds, round 5 APPROVE.

Gate record: [qa/security-levels/sprints/PF-23-S02/gate.md](../../../qa/security-levels/sprints/PF-23-S02/gate.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-23-s02-aggressive-deny-and-grant-enforcement.md`, Verification:

> - [x] `just fix -p codex-core`, `just fmt`, focused `just test -p codex-core pf_23_s02 pf_23_s01 pf_30_s03
>   mcp_openai_file aggressive confined`, `just test -p codex-sandboxing`, full `just test -p codex-core`.
> - [x] Linux clippy (`-D warnings`) on the RTX box; four GLM 5.2 videos; Opus 5.5 High review (five rounds, APPROVE).

Verbatim, gate record `qa/security-levels/sprints/PF-23-S02/gate.md`:

> - **Focused:** `just test -p codex-core pf_23_s02 pf_23_s01 pf_30_s03 mcp_openai_file aggressive confined`: 75/75.
>   `just test -p codex-sandboxing renaming_a_folder`: 1/1 (real `sandbox-exec`). Two seatbelt arg tests
>   (`create_seatbelt_args_for_cwd_as_git_repo`, `..._with_read_only_git_and_codex_subpaths`) fail the same way on
>   main in this checkout (temp folder under `/Volumes`), not from this change.
>
> - **Review (Opus 5.5 High):** `.codex-work/workers-20261002/pf23s02-review{1..5}/`; round 5 APPROVE. Round 1 CHANGES REQUESTED: parent
>   rename and symlink bypasses on macOS, Linux bwrap failing on symlinked or worktree paths, worktree hooks, call-id
>   confinement keys, grant resurrection, upload fallback, classifier gaps; fixed in `bd9d3aa48b`. Round 2 CHANGES
>   REQUESTED: rename onto a missing parent, Linux placeholders in the real home, the enclosing repository from a
>   subfolder, dangling links, a single confinement marker; fixed in `b25eb6460b`. Round 3 CHANGES REQUESTED: git
>   could be pointed elsewhere through `commondir` or a rewritten `.git` file, Linux placeholders through dangling
>   links; fixed in `23a1818ec8`. Round 4 CHANGES REQUESTED on Linux records only: anything below `.git` (but
>   objects, refs, index) is now Persistence for the command-text net, and the Linux limits are recorded below.
>
> - **Full crate** at `93e149e066` (merged with main): `just test -p codex-core` 3,897 of 3,902. Four failures
>   fail the same way on main (`config_schema_matches_fixture` and the 3 known baselines);
>   `shell_command_snapshot_still_intercepts_apply_patch` failed once under load and passes 3/3 alone (it passed in
>   the earlier full run). `just test -p codex-sandboxing`: only the two baseline seatbelt arg tests fail.
>
> - **Linux clippy** (`cargo clippy --locked -p codex-core -p codex-memories-write --all-targets -- -D warnings`) on
>   the RTX box: clean at `93e149e066` (round 1 caught two too-many-arguments and one type-complexity error).
>
> - **GLM 5.2 videos** at `93e149e066`, in [qa/demos/index/PF-23-S02.md](../../../qa/demos/index/PF-23-S02.md):
>   Aggressive denies a hidden home read before any untrusted content; with full access after untrusted content a
>   run-time-built hook write gets "Operation not permitted" while an ordinary write works; a literal `~/.zshrc`
>   append asks first (declined, file unchanged); typing into a shell started before untrusted content asks once.
>   Earlier takes were recorded verbatim: GLM refused two hook/alias requests it read as injected.
>   No video for grants (no issuing UI until PF-25-S01) or uploads (need ChatGPT auth): unit tests only.

Demo videos (4, index [qa/demos/index/PF-23-S02.md](../../../qa/demos/index/PF-23-S02.md)):

- `pf23s02-aggressive-from-start`: Aggressive blocks Corbanu home reads from the first command: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-aggressive-from-start-93e149e066f0-2026-10-07.mp4
- `pf23s02-hook-write-blocked`: After untrusted content, full access cannot write git hooks: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-hook-write-blocked-93e149e066f0-2026-10-07.mp4
- `pf23s02-persistence-write-asks`: After untrusted content, writing a shell start-up file asks first: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-persistence-write-asks-93e149e066f0-2026-10-07.mp4
- `pf23s02-older-process-asks`: Typing into a shell started before untrusted content asks once: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-older-process-asks-93e149e066f0-2026-10-07.mp4

Carried forward (open, not claimed): Grant UI to PF-25-S01; revocation UI to PF-25-S02; known limits in the gate record.

## PF-23-S03: Downgrade, restart and inheritance enforcement

Summary: tests `security_transition security_recovery` 26/26; wider core 1,270/1,271 (one failure also on main); `codex-security-policy revocation` 11/11; Linux clippy on the RTX box. TUI run: GLM 5.2 tmux functional run. Review: Opus 5.5 High: three rounds, APPROVE.

Gate record: [qa/security-levels/sprints/PF-23-S03/gate.md](../../../qa/security-levels/sprints/PF-23-S03/gate.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-23-s03-downgrade-restart-and-inheritance-enforcement.md`, Verification:

> - [x] `just fix -p codex-core`, `just fmt`; `just test -p codex-core security_transition security_recovery` and
>   the wider affected sets per PR; full `just test` of core and touched crates (gate record).
> - [x] Linux clippy (`-D warnings`) on the RTX box; Opus 5.5 High review (three rounds, APPROVE).
> - [x] GLM 5.2 tmux functional run and videos (Z.AI balance cleared 2026-10-07).
> - [x] Full isolated code-blind VM run and human sign-off are milestone gates only (not required per sprint; deferred to the Aggressive/Moderate ship and flag removal).

Verbatim, gate record `qa/security-levels/sprints/PF-23-S03/gate.md`:

> - **Focused (per PR, final tree):** `just test -p codex-core security_transition security_recovery` 26/26; with
>   `pf_23 pf_30 memory_stage_one network_approval sandboxing mcp_tool_call control_tests config:: session::`
>   1,270/1,271 (the one failure, `config_schema_matches_fixture`, fails on main).
>   `just test -p codex-security-policy revocation` 11/11. Suite: `pf_23_s03_stored_level_survives_restart_over_a_lower_config`
>   (real macOS sandbox).
>
> - **Full:** `just test -p codex-core -p codex-memories-write -p codex-security-policy -p codex-network-proxy -p codex-config
>   -p codex-protocol -p codex-state -p codex-rollout -p codex-thread-store`: 5,524 of 5,528. Failures are the known
>   baselines (`config_schema_matches_fixture`, `skills_use_aliases_in_developer_message_under_budget_pressure`,
>   `remote_compact_trim_estimate_uses_session_base_instructions`) and the load flake
>   `shell_command_snapshot_still_intercepts_apply_patch`.
>
> - **Linux clippy** (`-D warnings`, core, network-proxy, config, protocol, memories-write, state, thread-store,
>   rollout, app-server, security-policy) on the RTX box: clean at the final tree (round 1 caught a type-complexity error).
>
> - **Review (Opus 5.5 High):** `.codex-work/workers-20261002/pf23s03-review{1,2,3}/`. Round 1 REQUEST CHANGES
>   (last-writer-wins file, kill switch refused on a failed save, approval races, downgrade left broker channels
>   open, dropped project `[security]`, unrepairable state); round 2 REQUEST CHANGES (revocations replaced instead
>   of merged, unbounded lock wait, a repository's raise persisted for the user); round 3 APPROVE. Its follow-ups M1
>   (release of an unseen kill switch), L1 (failed downgrade mirror) and L3 (repair only by a level) are fixed with
>   tests; the rest are handed to PF-24-S02 (sprint record).

Demo videos (4, index [qa/demos/index/PF-23-S03.md](../../../qa/demos/index/PF-23-S03.md)):

- `pf23s03-unreadable-state-warns`: Unreadable security state fails closed and says so: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-unreadable-state-warns-c60857fc5d47-2026-10-07.mp4
- `pf23s03-stored-level-next-session`: A stored security level is enforced at the next start: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-stored-level-next-session-a230f2082141-2026-10-07.mp4
- `pf23s03-unreadable-state`: Unreadable security state fails closed and says so: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-unreadable-state-a230f2082141-2026-10-07.mp4
- `pf23s03-project-cannot-lower`: A repository cannot lower the security level: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-project-cannot-lower-a230f2082141-2026-10-07.mp4

Carried forward (open, not claimed): commit_transition follow-ups handed to PF-24-S02.
