# RETURN — pf83-failclosed-73

Allocation digest `aa7588ac294cfc665d5fc27db39c6654746e413609cbe9a1ac8e9eb060dab736`;
claim `fbaa8037-11e8-4740-9d0d-abcf57d2ab5c`; runtime `gpt-6-astra high`.
Brief SHA-256 verified before work:
`6b20c978f8a2b570ad307a48e8e42f4da512dec3d406930d091d7e1b7c20708f`.
Clean starting HEAD matched assigned base
`8d555171c4c12d0f7f77b24f0fa68c064d53890a`.

**QA guard revision complete; functional dispatch remains blocked on the owner
inputs below.** Routine internal preparation supports product heading
**Permission selection confirmation — TO BUILD**, excerpt “A submitted
selection is not a confirmed change,” active plan `p0-security-levels.md`,
sprint `PF-83-S01`. No product Rust changes, guest contact, product staging,
functional cases, live harness edits, credentials read, formatter/fix, subagents
or push. This is not independent acceptance, human-test readiness or release
qualification; the later functional/evidence gates remain open.

## Returned changes and evidence

- [Staging precondition](stage.py) pins the supplied public host-key bytes,
  uses strict SSH without a global host-key fallback, requires a successful
  identity query with closed stdin, and explicitly checks UUID, macOS 26.2,
  arm64, account and uid before opening an archive or invoking upload.
  Exceptions and checked subprocesses abort regardless of caller shell flags.
- [Negative controls](guard-controls.json): deliberately wrong public host key
  caused **zero SSH calls, zero upload calls, zero bytes and zero files
  transferred**; deliberately wrong UUID in a **local mock identity reply**
  caused one identity call with closed stdin, **zero upload calls, zero bytes
  and zero files transferred**. A simulated SSH exit 255 also aborted.
  Positive mock transport and valid identity controls passed.
  These prove local ordering/refusal; no live guest rejection is claimed.
- [Candidate guard](dispatch_guard.py) checks the real package, frozen packet
  inventories, every selected packet hash/candidate and exact live
  `fixtures.py:CANDIDATE`, then permits only the separately supplied admitted
  command. A stale pin raises an error before dispatch. The test's stale pin
  invoked **zero dispatch calls**; a matching temporary pin verified **288**
  packets. A [read-only live-pin check](live-pin-refusal.log) also refused with
  exit **1**; the live source pin remains unchanged.
- P3 path fix: packet regeneration requires `--harness`; no absolute harness
  path is embedded in the regeneration implementation. [Regeneration proof](regeneration-check.json)
  reports **288**, byte-identical inventories: F05 **8**, F06 **8**, F07 **16**,
  F08 **40**, F09 **216**.
- P3 exposure fix: [future payload builder](prepare_payload.py) selects exactly
  four binaries and two neutral packet assets plus a relative inventory.
  Attestation, absolute-path build provenance, manifest, runtime source and
  logs remain host-only; the collector is streamed, not stored in the actor root.
  Original bundle/attestation remain unchanged. The old builder refuses and its
  staging runbook is superseded. No product payload was generated/staged here.
- P3 logs fix: this round's raw prerequisite and both Rust gate logs are tracked,
  and the original round-70 raw logs are also tracked with their recorded
  SHA-256 values verified unchanged in [checks.json](checks.json).
- [Runbook](runbook.md) includes the owner-supplied real host key and UUID
  `F9AABE1C-BA2C-55D7-9859-A6ED560D3218`, explicit aborting commands,
  required live pin update, guarded dispatch and host-only provenance handling.

## Exact remaining owner asks

1. Admitted executor/mediator seam: supply or authorize the separate work for an independently allocated executor with an actual approved case interface and mediated inference.
2. `owner_admission` record: supply fresh package/runtime/policy/session-bound admission with actual actor-and-child denial probes and positive package/PTY/profile/inference controls.
3. Live harness pin permission: explicitly authorize updating the allocated `fixtures.py:CANDIDATE` to the frozen rebound binding and recording before/after hashes.
4. Independent reviewer and guest staging: name an evidence reviewer independent of implementer and executor and explicitly authorize the collector's guest/account/snapshot, staging paths and read-only actor access.

## Exact checks

Prerequisites built first, exit **0**; from this checkout's `codex-rs`,
shared dedicated round-67 QA target, `NEXTEST_TEST_THREADS=4`,
`INSTA_UPDATE=no`, allowlisted environment, guarded `just test`,
`--locked --offline --retries 0`.

| Lane | Run | Passed | Failed | Skipped | Exit |
| --- | ---: | ---: | ---: | ---: | ---: |
| `just test -p codex-app-server thread_settings` | 26 | 26 | 0 | 1082 | 0 |
| `just test -p codex-tui permission_confirmation` | 12 | 12 | 0 | 4161 | 0 |
| Local guard controls | 8 | 8 | 0 | 0 | 0 |

**38 Rust tests passed; exact failure names: none.** One TUI slow pass:
`app::permission_confirmation::tests::permission_confirmation_f10_request_does_not_optimistically_apply_or_persist`,
38.687 seconds. Gate summaries: 31.142 seconds and 38.709 seconds respectively.
No native credential prompt or live-profile access observed.
Raw logs and command/environment/exit records are adjacent to this return.
Python syntax, prepared Bash syntax and `git diff --check` passed.
The final guest collector hash loop uses ordinary hashlib streaming, compatible
with the guest system Python without requiring `hashlib.file_digest`.

Changed lines: **+3308/-149 across 29 files** (2,362 added raw-log lines); all changes remain in
`qa/initiative-control/management-bootstrap/`; zero Rust lines.

## Brief qualifications

No requested fix was based on an incorrect stale-pin or provenance finding.
The original staging excerpt was guarded if run in the same shell as its earlier
`set -euo pipefail`; it was unsafe as a standalone block, and now uses explicit
preconditions independent of that context. The two trusted identity inputs are
now present, but have not been checked against a live guest. The no-contact
instruction limits the wrong-UUID/SSH-error proof to local transport doubles;
it does not permit claiming live VM tests. The existing `owner_admission`
helper's reachability exception does not itself prove actor independence,
successful generation or full 175-control admission. The brief's four remaining
owner categories are correct; supplying them still must lead to actual passing
qualification. No approval or missing dispatch seam has been invented.
