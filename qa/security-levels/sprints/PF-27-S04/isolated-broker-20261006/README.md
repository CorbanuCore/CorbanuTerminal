# PF-27-S04 isolated broker process: gate evidence (2026-10-06)

Candidate: `cf852aa347` (tests, tmux run) and `1a4370992f5a` (demo videos; adds the spawn-log fix below) on
`feat/pf27-s04-broker-20261006`, macOS arm64 debug build, Rust 1.95.0. Later commits change only comments and docs.
Feature flag: `[features] isolated_credential_broker = true` (default off). Synthetic credentials only.

## What ships

With the flag on (and the managed network proxy active), Core's proxy starts
`corbanu --codex-run-as-credential-broker` and hands it the provider tokens it
virtualizes (GitHub `GH_TOKEN`/`GITHUB_TOKEN`/`GH_ENTERPRISE_TOKEN`, `OPENAI_API_KEY`).
Core keeps only dummy values and opaque references. Agent commands see dummies.
For a request that carries a dummy, Core sends a signed, typed frame (host, port,
method, path, reference) over a private Unix socket; the broker checks the OS
peer, the frame, the run generation, the replay window and the host binding,
substitutes the real header and makes the upstream HTTPS request itself.
With the flag off nothing changes.

## Tests (`just test`)

| Command | Result | Log |
| --- | --- | --- |
| `just test -p codex-secret-broker pf_27_s01` | 24 passed | `focused-secret-broker.log` |
| `just test -p codex-core pf_27_s01` | 2 passed | `focused-core.log` |
| `just test -p codex-network-proxy -p codex-secret-broker -p codex-arg0 -p codex-features` | 334 passed (12 new isolated-broker tests) | `affected-crates.log` |
| `just test -p codex-core -E 'test(network_proxy) \| test(credential) \| test(pf_27) \| test(schema)'` | 88 passed | `core-related.log` |

New network-proxy tests re-execute the test binary as the broker, so the
process boundary is real: substitution only inside the broker (including an
HTTP/2 client against an HTTP/1.1-only origin), wrong OS peer, forged, tampered
and replayed frames, other-run binding, unbound host, method/path mismatch,
revocation closing an open stream, crash fail-closed, restart rejecting old
handles, bounded registrations, controller exit, and external SIGTERM cleanup.
`just fmt` and `cargo clippy --fix --tests` ran clean on the touched crates.

## GLM 5.2 tmux functional run

Driver: `/Volumes/CorbanuDrive/Corbanu/.codex-work/broker-lane-20261006/` (`launch.sh`, `run1.sh`,
`run3.sh`, `fixture/`). Each case uses a disposable home and workspace, `-m glm-5.2 -c model_provider="zai"`,
and a synthetic GitHub Enterprise host (`127.0.0.1.nip.io:8443`, throwaway CA) that reports only whether the
request carried the real synthetic token and a SHA-256 prefix of what it received (real: `946ae98e9fbe`).

| Case | Expected | Observed |
| --- | --- | --- |
| V0 flag off (baseline) | agent env holds the raw token | env sha `946ae98e9fbe`; no broker process |
| V1 flag on | agent env holds a dummy; server still authorized | env sha differs; server `authorized: true`, sha `946ae98e9fbe`; broker is a child of corbanu |
| V2 direct call | agent cannot use the broker socket | agent curl exit 7 (sandbox); same-user host curl `(52) Empty reply` (wrong peer); server log empty |
| V3 crash/restart | dead broker fails closed; restart gets a fresh broker | after `kill`, same and next command `Request blocked by network policy`; after `/quit` and relaunch a new broker pid serves an authorized request |

No raw synthetic token appears in any recording (checked by search).

## Demo videos (SOP: `qa/demos/README.md`)

Recorded with `scripts/demo_video.py` from `qa/demos/specs/pf27-*.toml` and published to the `demos`
prerelease. The links are in [`qa/demos/index/PF-27-S04.md`](../../../../demos/index/PF-27-S04.md).
1. Substitution: the broker is running, the agent's token hashes differently, and the server reports `authorized: true`.
2. Direct call denied: the agent's curl exits 7, and the user-shell curl gets `(52) Empty reply`.
3. Crash fails closed: after `pkill` of the broker, `Request blocked by network policy.`

The secret scan of the first take flagged a **product finding**. Core's `spawn_child_async` trace logged the child
environment, including raw credentials, before virtualization. Fixed in `1a4370992f5a`, which now logs variable
names only. The later takes are clean.

Earlier tmux recordings (asciinema, before the SOP landed) are kept outside git in
`/Volumes/CorbanuDrive/Corbanu/.codex-work/broker-lane-20261006/casts/`:

| File | SHA-256 |
| --- | --- |
| `v0-baseline-flag-off.cast` | `e57f04c6a488d84c3659538d476938c0e7f33efa6fe47907f39e45b4158b727c` |
| `v1-isolated-substitution.cast` | `8411fc7686cb517a5bb74bd740d90180bb30a09e4adfbbba0458bc0dea14acc6` |
| `v2-direct-broker-call-denied.cast` | `474a09d4c4c4a6292dfa847c065c702e722db2bb99499d4d88db1bbbd66127c8` |
| `v3-broker-crash-fails-closed.cast` | `bd9c4971c2b7f031e9cf772bd2726f80db07b73a50c8085c2eb196b6b9512105` |

Labelled earlier attempts are in `casts/attempts/` (pre-review binary). In V3 attempt 1, GLM refused to write the
token variable to a file. Its words: "never print or persist secret values to files." Attempts 2 and 3 were driver
timing errors, not product failures. The V3 case was then reworded to keep the token in the process environment.

## Review

One independent Opus 5.5 High review of `1c1ab846a2` returned **APPROVE WITH NITS** (no P0/P1). A re-check of
the fixes (`review-opus-2.md`) also returned APPROVE WITH NITS. Its nits are a comment wording fix (done) and two
timeout limits, now recorded in the sprint file. Fixed in `cf852aa347`:
- P2: the broker now hardens itself (no debugger attach, no core dumps).
- P2: the broker starts with the proxy, before any agent process exists, and is never respawned.
- P3: accept-error backoff, idle response timeout, spawn cleanup gaps, Unix-only arg0 dependency, Windows dead-code
  warning, exhaustive provider mapping, HTTP-version comment.

Recorded as open in the sprint file:
- the macOS close-on-exec window on other threads' spawns (PF-27-S02);
- a writable socket directory, which allows denial of service only (PF-27-S02);
- no production revocation trigger yet;
- method/path limits and private-IP GitHub Enterprise hosts.

The full report is in `review-opus-1.md`.

## Not claimed

Core still holds env-sourced values in its own environment, and its own model-client auth still runs in process.
The broker runs as the same user without an OS sandbox. Native privileged bootstrap, the code-blind VM run and human
sign-off remain milestone gates.
