# PF-83: the sealed guest ran real functional cases, and why it could not before

**Fable, 2026-09-16.** Travis set the standard: the sandbox exists so the
executor cannot read the product source, and it exists so functional tests can
be run without that access. Both halves are now demonstrated, not asserted.

## What ran

Three cases, executed by me over SSH to the sealed guest `agent@192.168.64.3`,
each in a fresh disposable profile (`CORBANU_HOME`/`CODEX_HOME`/`TMPDIR` under a
`mktemp -d`), using the subscription credential staged at
`/Users/agent/.pf83-auth/auth.json` by owner decision. Model `gpt-6-astra`,
effort low.

| Case | Ask | Result |
| --- | --- | --- |
| F-A | Read `sample.txt` and reply with its line count | `exec` ran `awk 'END { print NR }'`, answered `3` |
| F-B | Sort the unique lines of `seed.txt` into `result.txt` | `result.txt` contained `alpha`/`beta`/`gamma`; workspace-write sandbox honoured |
| F-C | Run a shell command writing `uname -s` and the user, then read it back | `Darwin-agent` — confirms an unprivileged `agent` account on macOS |

Anti-cheat was re-checked from inside the same runs: `https://github.com/` and
`https://raw.githubusercontent.com/` both returned nothing and failed, while the
model endpoints served the turns. The executor did real work and could not have
read the product source while doing it.

## Why every earlier attempt failed

The guest was not running the pinned package. Its staged `package/` directory
held a single `corbanu` binary, `3e99a6cb6901…`, while the pinned package
recorded in `package-manifest.json` is `57b1915392…` and ships five binaries,
including `codex-code-mode-host`.

With the host binary absent, every tool call failed:

```
ERROR codex_core::tools::router: error=failed to spawn code-mode host
  .../package/codex-code-mode-host: No such file or directory (os error 2)
```

and the model answered "Unable to read sample.txt because the execution tool
failed." That is why all 380 verdicts sat blocked: not a macOS ceiling, not a
missing actor runtime, not the control count — a staging defect in our own
harness. Extracting the pinned package and pointing a run at it made the same
case pass first time.

Two further observations, recorded rather than acted on:

- Setting `features.code_mode = false` did **not** avoid the failure. Reading
  the source afterwards explains it: `requested_tool_mode` takes
  `model_info.tool_mode` first, so a model that declares code mode ignores the
  feature flag. That is deliberate.
- `effective_tool_mode` is supposed to fall back to `ToolMode::Direct` when the
  host is unavailable and `disable_in_process_fallback` is false, which is the
  default. It did not fall back; the router kept trying to spawn the host on
  every call. A missing host binary therefore bricks the product instead of
  degrading. Worth a product fix, filed here rather than claimed as qualified.

## What this closes and what it does not

Closed: the executor runs, performs inference, executes tool actions and writes
files inside the sealed guest, with source hosts denied. Under the owner's
standard that is the gate, and it is met.

Not closed, and not claimed: these are three cases I wrote, not the frozen
F01–F11 original set, and I am not an independent code-blind executor — I have
read the source. What this proves is that the environment can host that work,
which is precisely what was in doubt. The harness still stages runs itself and
must be corrected to extract the pinned package rather than one binary; until it
is, the 380-verdict suite stays blocked by the same defect this receipt names.
