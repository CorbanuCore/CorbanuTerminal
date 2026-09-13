# PF80 fresh Fable launcher candidate

Classification: product initiative, PF-80-S01 **in_progress**, under
`docs/plans/active/initiative-delivery-control.md`. Product citation:
**Internal delivery control — TO BUILD**, “fresh Fable 5.1 High management through
Corbanu/TMUX” and “Use sequential sprints per initiative”. The exact mandate is
`docs/research/tasknode-integration/coordinator-bootstrap-20260913.md`.

Implementation checkout: `/Volumes/CorbanuDrive/Corbanu/worktrees/fable-launcher-20260913`,
branch `bootstrap/fable-launcher-20260913`, starting HEAD
`1c0df3cb8517b4b578c421ca59ba215f48c7d8c3` as explicitly assigned. The allocation's
older `eb01bf006...` base is historical. Parent owns shared plan/sprint updates.
Only this receipt and `scripts/initiative_control/{fable_launcher,test_fable_launcher}.py`
are owned. No coordinator/integration changes, agent/review invocation, deployment,
real credential inspection or live inference was performed by this worker.

## Coordinator interface

```text
python3 scripts/initiative_control/fable_launcher.py \
  --briefing /absolute/redacted-briefing.json \
  --runs-dir /absolute/private-short-directory \
  --binary /absolute/corbanu --auth-file /absolute/private-auth.json \
  --timeout 300
```

`--runs-dir` must already be an owner-only directory (0700), without symlink
components. Each invocation creates a unique child; existing runs are untouched.
Use a short path: the resulting `<runs-dir>/f-XXXXXXXX/tmux.sock` must be under
100 bytes for macOS Unix sockets. No shared application home is accepted.
Briefing is one JSON object, at most 64 KiB, already redacted by the coordinator.
Timeout is 1–3600 seconds (default 300) for the interactive phase; allow preflight
and bounded shutdown overhead when supervising the CLI. SIGINT/SIGTERM request
cleanup; do not SIGKILL the sidecar during that cleanup window.

The outer owner provisions `--auth-file` as JSON with exactly one string field,
`CLAUDE_CODE_OAUTH_TOKEN`. The file must be owner-only (0600 or stricter), owned
by the caller, regular, single-linked, with no symlink path components. It is
read as data, never sourced. No actual token example is stored here. The child
rechecks the file and its private digest before injecting the value into only
Corbanu's environment. Auth values are absent from command arguments, briefing,
launcher script, manifest and stdout. The file is neither copied nor deleted.
This uses Corbanu's existing env-first Claude subscription resolver from
`codex-rs/cli/src/claude_oauth.rs`; no login/credential-discovery command is run.

Stdout is one JSON receipt. Successful exit 0 means `status="completed"`,
`decision` is the parsed **entire** final JSON object, and `run_id`, `session_id`,
`model="claude-fable-5-1-plan"`, `provider="claude-plan"`, `effort="high"` are present.
`thread_id`, `turn_id`, `response_id`, timestamps, exact private socket/session,
artifact paths and verified `shutdown` accompany them. `session_id` comes from
session metadata, not the TMUX name or a fabricated UUID.

Exit 1 returns `status="failed"` or `"timeout"`, `decision=null`, and a fixed
redacted `error` code. Cancellation is `failed`/`cancelled`. Unclean or unverifiable
shutdown fails the run even if a candidate answer exists. The receipt is retained
in its private run directory; invalid arguments/root paths may fail before that
directory can exist. A filesystem write failure is reported as failure, never
success. Syntax errors use argparse's conventional exit 2 without a run.
Parent must require exit 0, completed, clean shutdown and the exact tuple before
its own decision validation/acceptance. Artifact paths are private, not export URLs.

## Evidence and boundaries

The launcher uses interactive Corbanu in actual TMUX, no exec/resume/fork/last.
It clears inherited environment, assigns fresh HOME/Corbanu/Codex/Claude/XDG/temp
directories, disables project instructions, skills, hooks, shell, agents, apps,
plugins and web/browser capabilities through supported configuration, pins project
discovery to CWD (`project_root_markers=[]`) with a private Git ceiling, and sets
read-only sandbox/never approval. The entire bounded briefing is sent literally;
text and Enter use separate commands. `RUST_LOG=trace` and private log_dir follow
the required TUI skill. No broad permission flag is supplied.

Acceptance requires one fresh CLI session, no inherited history/parent, exact
turn model/provider/high effort and read-only permissions, a correlated provider
response identity, and a matching terminal task_complete/turn_complete event with
the full JSON answer and no terminal error/truncation. Pane quietness, tool outputs,
partial JSONL writes and JSON snippets are never completion evidence. Tool calls
and additional turns fail the attempt. The final records are rechecked after
shutdown. The launcher validates action framing; allowed kinds, authority, scope,
dependencies and expected/state revisions remain exclusively parent decisions.

Private evidence includes binary version/hash, source/launch/briefing hashes,
briefing manifest, readiness/final pane captures, complete final answer, input
hash/key journal, original session records and durable receipt. A candidate is
recorded before exit but only the final receipt is dispatchable. Known injected
secrets and recognizable token patterns are rejected/redacted on output, including
decoded JSON strings. This is not a general classifier for arbitrary private data.
Raw native trace/session files are private evidence and require owner inspection
and redaction before any export; the launcher does not publish them.

A pre-exec gate records PID/group/start identity before starting Corbanu. Cleanup
tries Escape, clear input and `/exit`, waits, then signals only observed owned
process identities, removes only this session and verifies process/session exit.
Failures retain artifacts. No global TMUX kill-server or process-name kill exists.
Fresh context/read-only configuration is **not** complete filesystem/tool/network
containment or a qualified code-blind functional executor; SIGKILL/host loss and
adversarial escaped descendants require an external supervisor/isolation owner.

## Offline verification and remaining acceptance

Exact suite: `python3 -m unittest discover -s scripts/initiative_control -p test_fable_launcher.py -v`.
Final run: **27 passed, 0 skipped, 23.032s**. Tests use synthetic
tokens/records and actual private TMUX sessions with an offline fake terminal.
The real reference binary is used only for `--help` and `--version` in fresh
empty homes. Tests skip explicitly if TMUX/reference binary is absent.

Preserved attempts: initial 24 tests / 18.001s: 23 passed, one forced-cleanup
fixture failed because Escape remained in canonical terminal input. Added C-u
before `/exit`; corrected 24 tests / 20.127s passed. Subsequent hardening adds
canonical permission checks, escaped-token rejection, credential-rotation and
late-final-record regressions: 25 tests / 20.278s and 27 tests / 23.419s passed;
these results do not erase the first attempt. Final project-root isolation pass
passed all 27 tests in 23.032s. `python3 docs/plans/check.py` passes (3/3 active),
`python3 docs/sprints/check.py` passes (115 current / 126 archived), and scoped
Git whitespace validation passes. Total allocation: 1,168 lines (599 launcher,
445 tests, 124 this evidence record), below the 1,200 target / 1,500 hard limit.

Parent still owns two actual independent Fable runs, live identity/config/auth
compatibility, full decisions, timeout/cancel shutdown qualification, material
review and combined coordinator tests. This internal engineering increment uses
the allocation's reasoned internal-only GUI N/A; later combined dashboard/Slack
acceptance still requires independent code-blind design/execution/evidence.
No TensorCash/Isometric runtime or release/benchmark acceptance is claimed; no
human sign-off, recurring enablement or sprint completion is claimed.
