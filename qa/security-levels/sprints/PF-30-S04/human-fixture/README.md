# Human memory fixture — ready for assisted testing

This disposable test lets an operator observe actual background-memory requests
without exposing private history or needing a provider account. Do not mark
human checks21–23 passed until the operator has performed them. Combined
qualification passed Rust16/16 and Python2/2, all five actual-key scenarios,
pinned startup/cancel and final Astra review5. [Exact evidence](combined-67fec6a6/README.md).

The product is the immutable RTX candidate `b12e32db3`, not a newly built Mac
shortcut. The test-only helper seeds synthetic history and runs two loopback
fake providers. It clears inherited credentials for Corbanu and disables all
unrelated providers only in the fresh disposable fixture home. “Rented GPU” is
not used or spoofed. No protected-mode inference is enabled or claimed.

## Lifecycle

The coordinator compiles and pins the test runner under the shared RTX build
lock, then releases that lock. Run the pinned runner directly, not Cargo, for
human waiting. The runner owns the fake services, private TMUX server and
temporary home. It allows ten minutes per run; delayed memory responses allow
120seconds for the switch/exit action. Missed windows are not passes.

`ready.json` contains exact candidate/runner hashes and disposable paths.
Use only the generated `attach.sh` for that run. `status.json` reports source
identity, canary-request count, successful source-output count, endpoint/model
routing, and time left in the delayed-response window. No authorization headers
are logged. `touch <evidence>/cancel` cancels the owned fixture; `finished.json`
reports the result after normal cleanup. Do not disable the cleanup watchdog,
suspend the runner, or attach to an unrelated user's session.

## Operator actions

Always enter this exact harmless foreground prompt, then press Enter separately:

```text
HUMAN_FOREGROUND synthetic fixture
```

### 21 — startup

Attach to a new `startup` fixture. Send the prompt above. Wait for
`canary_requests:1` and `source_outputs:1`, and observe “A foreground complete”.
Type `/exit`. Inspect `outcome.json` and `finished.json` after the runner exits.
Launching ordinary chat without the seeded rollout is not this check.

### 22 — provider replacement during pending memory

Attach to a new `provider-switch` fixture. Send the prompt, wait for one pending
A canary and zero source outputs, then act within the120second window:

1. `/providers` → **Memory Fixture A** → **Deactivate**.
2. In “Choose replacement”, select **Memory Fixture B — memory-fixture-a**.
   This persists B before deactivating A, in the disposable fixture only.
3. Escape back to chat. `/model` → **Memory Fixture b model** → confirm
   effort. Then send the harmless prompt again.
4. Observe “B foreground complete”. The status must show a foreground request
   at endpoint B with model `memory-fixture-b`, the old binding's provider-change
   denial, and zero successful outputs for the original canary source.
5. `/exit`, then inspect the final outcome.

This is provider replacement plus model-effort selection, **not proof that
direct `/model` custom-provider switching has UX parity**. See
[unpatched adjacent findings](adjacent-model-picker-findings.md).
The fixture supplies its own synthetic `model_catalog_json`, with distinct
model names and exact provider metadata; no provider/model identity is inferred
from row numbers or “current” labels. These are fake models, not live GPT calls.

### 23 — owner exit and explicit restart

Attach to a new `pending-exit` fixture. Send the prompt and wait for one pending
canary request with zero source outputs. Type `/exit` before120seconds elapse.
From the coordinator shell, `touch <evidence>/restart`; the runner creates a new
owned session in the same disposable home and refreshes `attach.sh`/`ready.json`.
Attach again, run `/status` only, then `/exit`. The source must still have zero
successful outputs and exactly one canary request. Do not send another ordinary
prompt in this restart phase: that would deliberately trigger a new memory job.

## Qualification and limitations

`rehearse-pinned.py` is machine rehearsal of the ignored manual entry: it sends
actual startup keys using only the child-generated owned TMUX coordinates, then
checks normal exit and separate cancellation cleanup. The Rust rehearsal covers
provider replacement, pending exit/restart and bounded timeout as well. Neither
script claims named-human acceptance.

The current runtime may display a code-mode-host warning in this synthetic
configuration; the fixture never asks for tool execution. That warning is not
memory success or failure and is not changed by this test-only support.

No local builds, release benchmark, TensorCash/Isometric application workflow,
or Mac shortcut update is part of this routine QA support. Product fixes and
release acceptance stay with the coordinator.

## Frozen RTX launch command

Use this only when the coordinator arranges human testing. The executable
interface was rehearsed automatically; independent reviews found no remaining
landing blocker. The original owner-termination false-negative is corrected:
only the exact expected owner-denial reason is exempt, while mixed/other
failures, outputs and missed windows still deny. Preserve any failure evidence;
never bypass a check. Combined integration is complete; named-human acceptance
remains pending. No fixture is left waiting unattended.
Run as travis on RTX, from the preserved fixture source worktree. No build lock
or Cargo invocation is needed. Model metadata is bundled in the pinned runner;
the source worktree remains the explicitly trusted synthetic working directory.

```bash
cd /home/travis/worktrees/security-human-combined-67fec6a6
export TMPDIR=$(mktemp -d /home/travis/security-round5/memory-tmp/human.XXXXXX)
export CARGO_BIN_EXE_codex=/home/travis/security-round5/evidence/anchor/combined-b12e32d/candidate/codex
export CORBANU_MEMORY_CANDIDATE_SHA256=c567826ff5f15fccd71f8294c93210a158217a2ba31224c55d7d78269b1d2bea
export CORBANU_MEMORY_HUMAN_OPT_IN=1 CORBANU_TMUX_REQUIRED=1 RUST_MIN_STACK=8388608
export CORBANU_MEMORY_HUMAN_CASE=startup
# Choose a NEW path; do not create it. The helper refuses an existing path.
export CORBANU_MEMORY_HUMAN_EVIDENCE=/home/travis/security-round5/evidence/human-memory/operator-combined-startup-1
/home/travis/security-round5/evidence/human-memory/combined-67fec6a6/remediated-2/candidate/all \
  --ignored --exact suite::memory_human_fixture::human_memory_fixture --nocapture
```

For the other cases change `CORBANU_MEMORY_HUMAN_CASE` to `provider-switch` or
`pending-exit` and choose a fresh evidence directory. While the command runs,
use another SSH terminal to run `bash <evidence>/attach.sh`. The runner hash is
`dea1c0286c4c1e3cf23f3954a28cb6c9274939dd7d5e4033ca296ceb5e748673`.
An existing path, missing opt-in or wrong product hash is not a completed check.
Never replace the pinned candidate path with an arbitrary binary silently.
