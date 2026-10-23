# RETURN — pf83-accuracy-77

Allocation digest `86d48f4d6a090479917de88878e4aec2faed1567f196e7a46683c8fa863a9ae3`;
claim `88426466-708c-4f6d-9684-d4a379794ba7`; runtime `gpt-6-astra high`.
Brief SHA-256 verified before work:
`2844e34d8137ef6fabf82e9fb4ae159d8f64b2de5785562b79da995d369bdf26`.
Starting HEAD was the assigned clean base
`0323f9b598fbc7ab063a98feef6bc670b12a8d53`.

Routine internal QA preparation supports **Permission selection confirmation — TO
BUILD**, “A submitted selection is not a confirmed change,”
`p0-security-levels.md` / `PF-83-S01`. Product behavior and authorization are
unchanged. Code-blind functional execution/TUI/live-repository qualification are
not applicable to these internal QA-only edits; the later independent packaged
functional/evidence gates remain open. Integrator acceptance of that internal
N/A is not supplied. No live profile, native credential prompt, guest contact,
guest staging, product launch, live harness edit, subagent, formatter or push.

## Corrected isolation statement and actual exposure

The corrected [runbook paragraph](../pf83-failclosed-73/runbook.md) reads:

> Sidecar attestation, provenance, manifest, normalized packets, runtime source and gate-log files remain on the trusted host; the collector is streamed and never stored under the actor root. The packaged binaries still carry host build paths: these reveal the local account name `Neo`, volume/worktree and QA-round names, internal Rust source/module filenames, Rust 1.95.0 / aarch64-apple-darwin toolchain details, dependency names/versions and build-object names.

The immediately following text states that sidecar exclusion does not remove
implementation metadata or establish code-blind isolation and calls for the
owner to assess the exposure before staging. Frozen package bytes were unchanged.

[Read-only scan](embedded-paths.json), reproducible with [inspect_paths.py](inspect_paths.py),
found 618 matching distinct path strings in code-mode-host, 4,667 in corbanu,
10 in corbanu-acp, and 753 in walletd. Examples include
`agent-graph-store/src/lib.rs`, `keyring-store/src/lib.rs`,
`anstream-0.6.21`, `addr2line-0.25.1`, and the toolchain/account path.
These are implementation/dependency and local identity disclosures beyond
directory layout. This bounded path scan is not a full metadata/secrets audit;
no credential files or authentication data were read or printed.

## Dispatch status and check counts

[dispatch_guard.py](../pf83-failclosed-73/dispatch_guard.py) no longer passes
`check=True` to its dispatched subprocess. The original candidate refusal still
raises before invoking it.

[Real process demonstration](controls.json): child exits **0, 7, 23** produced
wrapper exits **0, 7, 23**, respectively, with empty stderr and **288** packet
hashes/candidates checked on each invocation. The command was only an isolated
Python exit fixture; no product binary/case was launched. The existing stale-pin
control still passed with zero dispatch calls.

Removed unmeasured `python_syntax_files=9` and `bash_syntax_blocks=2` from the
round-73 generator and its checks record. New [measured syntax evidence](syntax-checks.json)
enumerates **13 Python files** parsed and **2 Bash blocks** checked with
`bash -n`, all passing, via [measure_syntax.py](measure_syntax.py).

## Self-verifying staging preflight and its actual result

[preflight.py](../pf83-failclosed-73/preflight.py) is called by the runbook after
host relocation/regeneration and the separately authorized pin update, before
payload preparation or the first guest staging. It emits every check and exits
nonzero unless all pass. It reuses actual package attestation, candidate-agreement,
public host-key and guest identity checks; the existing staging transport repeats
identity immediately before upload.

[Actual existing-harness run](preflight-existing-harness.json), with
[command and process exit records](preflight-commands.json), exited **1**:

| Precondition | Observed result |
| --- | --- |
| Receiving locations/fresh outputs | Failed: receiving allocation absent |
| Package attestation after move | Passed: original already-relocated four-binary bundle |
| Actor asset allowlist | Passed: four binaries plus two neutral assets |
| Candidate and 288-packet agreement | Failed: real harness pin remains stale |
| Public host-key pin | Passed: frozen digest |
| SSH identity metadata | Failed: no identity supplied/read |
| Owner checker identity | Failed: no authorized pinned verifier supplied |
| Executor/mediator interface | Failed: verifier unavailable |
| Owner admission | Failed: verifier unavailable |
| Live-pin authorization/before-after hashes | Failed: verifier unavailable |
| Independent reviewer/staging authorization | Failed: verifier unavailable |
| Live SSH host/UUID/OS/architecture/account/uid | Failed: no guest-contact authorization; no contact attempted |

The no-argument diagnostic separately reports **3 passed / 9 failed / 12 total**,
exit **1**. Neither is a staging approval.

Owner-dependent evidence needs a trusted, allocation-pinned verifier with the
explicit per-check command contract in the runbook. That verifier is **not
supplied or fabricated here**. Missing/failed checks prevent guest contact.
This is an executable check interface for outstanding prerequisites, not a claim
that owner admission/independence can be established from a success boolean.

The local positive control verified the real cloned/moved bundle and all packets
with a synthetic matching pin and identity file, a clearly synthetic owner
verifier, and mocked SSH identity. **12/12** checks passed in that control only.
A wrong UUID failed; a wrong verifier digest failed and suppressed SSH.
No actual guest or owner evidence was substituted by these controls.

## Exact gates and raw evidence

Prerequisites built first, exit **0**. Both Rust commands ran from this
checkout's `codex-rs` via guarded `just test`, using the shared dedicated
round-67 QA `CARGO_TARGET_DIR`, isolated allowlisted environment,
`NEXTEST_TEST_THREADS=4`, `INSTA_UPDATE=no`,
`--locked --offline --retries 0`.

| Lane | Run | Passed | Failed | Skipped | Exit |
| --- | ---: | ---: | ---: | ---: | ---: |
| `just test -p codex-app-server thread_settings` | 26 | 26 | 0 | 1082 | 0 |
| `just test -p codex-tui permission_confirmation` | 12 | 12 | 0 | 4161 | 0 |
| Local guard/preflight/exit controls | 13 | 13 | 0 | 0 | 0 |

Exact failure names: **none**. Gate elapsed summaries: **31.798s** and **41.889s**.
The TUI F10 test was slow and passed. No native credential prompt or live-profile
access was observed. All raw prerequisite, Rust and local-control logs are
included alongside command/environment/exit records.
[checks.json](checks.json) derives counts from raw summaries and individual PASS
lines and binds the raw log SHA-256 values. `git diff --check` passed.

## Changed lines and brief qualifications

Changed lines: **+2,838 / -11 across 25 files**, including **1,185 raw-log lines**.
All changes are under `qa/initiative-control/management-bootstrap/`;
zero Rust lines. Historical raw logs and frozen package identities remain intact.

None of the three reported accuracy findings was wrong. “Return path is always
0” applies to normal subprocess completion; launch errors/signals have their own
behavior. The regression demonstrates ordinary nonzero process exits requested
by the brief.

A successful **live** preflight cannot be claimed without the outstanding
authorization, admitted interface and evidence. This allocation can prepare the
checks, run the actual local refusal and demonstrate controlled positive/negative
paths; it cannot supply missing owner decisions or independently certify them.
The new owner-verifier contract remains a concrete missing prerequisite for a
future fully automated staging pass. No approval, acceptance or release readiness
is claimed.
