# Current-main live TUI fixture checks

2026-11-18 UTC. **Preflight only; repeat on the final release branch.**
These checks use an already-running local model and actual TUI tool execution.
They do not qualify the canonical live repositories, production provider routes,
release packages or the benchmark matrix.

## Frozen task and model

The task reads `input.json`, copies its unpredictable marker, computes the sum
of `[17, 31, 59]`, and writes `result.json` with exactly `marker` and `sum`.
The prompt does not reveal the marker. The verifier independently checks the
result, unchanged input, persisted tool calls and a completed turn. Each case
has a fresh fixture/profile, a 240-second cutoff, actual tmux key events and
trace logs. No concurrent inference cases ran.

The local model discovery endpoint reported one ready tool-capable model:
`Qwen3.8-Flash-Next-Uncensored-MLX-Serve-mixed-4-8bit`, context 262144. No model
was loaded/switched and no server setting was changed. The TUI used a custom
Chat Completions provider without credentials; its context window matched the
reported value. The uncatalogued model emitted the fallback-metadata warning.
This is not qualification of the new production models in gate PR #2571.

## Results and boundaries

| Platform | Source | Successful replay | Tool evidence | Boundary exercised |
| --- | --- | --- | --- | --- |
| macOS arm64 | `5fd571822bef255237f963c7bc18286a9110f58b` | Four checks pass, 11.28s | Two `exec_command` calls | External Seatbelt; candidate native sandbox disabled for this fixture |
| Linux x64 | `d3fe32513ee65deb397fa228c786212fb5afacd3` | Four checks pass, 9.55s | Two `exec_command` calls | Non-root Docker; candidate native sandbox disabled for this fixture |
| Native Windows x64, High launcher | `5fd571822bef255237f963c7bc18286a9110f58b` | Four checks pass, 14.58s | Two `shell_command` calls | Candidate non-admin sandbox mode; launcher was High integrity |
| Native Windows x64, Medium launcher | same source and binary | **Fail**, 240.39s cutoff | File-read commands fail in private-registry labeling | [Measured Medium token and finding](WINDOWS-MEDIUM-FINDING.md) |

The preparation source adds only notes/evidence to the main baseline. All
executables still report 0.1.48; exact binary hashes are in the raw results.

- macOS: operator-home access and all three Keychain services denied. The
  replay's external Seatbelt profile allows writes only inside its disposable
  root (plus `/dev/null` and `/dev/tty`). Positive private-write and negative
  outside-write probes passed. The first attempt nested the candidate's native
  sandbox inside Seatbelt; tool commands failed with `sandbox_apply: Operation
  not permitted`, and no result file was produced. That failed attempt is kept.
  The replay is not a native macOS sandbox-enforcement pass.
- Linux: read-only container root, candidate and harness mounts; only the fresh
  fixture and bounded `/tmp` are writable. UID/GID 1000, all capabilities
  dropped, no-new-privileges, 4 GiB memory and 256 PID limits. No host home,
  keyring, IPC or Docker-socket mount. Host networking connects to a private
  loopback SSH forward; no provider credentials are present. Docker inspection
  is preserved. This is not native bubblewrap qualification.
- Windows: private profile and OS-home paths, debug native-keyring guard,
  non-admin sandbox selected, no UAC action. The first attempt lacked private
  AppData directories; native commands failed with `no LocalAppData folder` and
  the task reached its cutoff. The replay created those directories and checked
  the Windows known-folder lookup stayed under the disposable OS home before
  starting the TUI. Shell commands then read and wrote the fixture successfully
  from the High-integrity SSH launcher. A separate measured Medium-token replay
  subsequently failed at private-registry labeling; see the finding above.
- Windows and Linux SSH forwards bound only remote loopback and were closed
  after their cases. The source model server was left unchanged.

## Canonical repository availability

Both configured policy URLs, `https://github.com/agtico/tensorcash.git` and
`https://github.com/goodalexander/isometricgame.git`, returned repository-not-found
to clone and authenticated `gh repo view`. Neither `CORBANU_TENSORCASH_REPO`
nor `CORBANU_ISOMETRIC_REPO` was configured. Those lanes remain unavailable,
not passed. These fixtures do not replace them or reset the benchmark cadence.

## Evidence

Raw results, including failures, are `preflight/live-*.json`. Detailed captures,
rollouts, fixture inputs/outputs and logs remain at:

- macOS: `.codex-work/astra-rel149-live-current/` and `...-current2/`;
- Windows: `C:\CorbanuQA\astra-rel149\live-current` and `live-current2`;
- Linux: `glitch:~/corbanu-rtx/astra-rel149/live-qa-current/`;
- harnesses: `$W/astra-rel149-live-tui.py`, `astra-rel149-windows-live.py`,
  `astra-rel149-linux-live.py`; original failed harnesses retain `-v1.py` copies.
