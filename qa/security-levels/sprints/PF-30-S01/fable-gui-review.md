# PF-30-S01 Fable High desktop review

Date: 2026-08-27. Historical result: **review completed; changes required**.
Subsequent fixes and final-tree evidence are recorded in [review fixes](review-fixes.md).
This is evidence for the existing product initiative, not a release or platform
certification. Product authority: **Moderate/Aggressive isolation and content
provenance** — “Network destinations and redirects are enforced”; “Reuse an
installed Podman or Docker runtime without replacing it or changing its global
configuration.”

## Method and candidate

At Travis's explicit request, Computer Use operated the logged-in Claude desktop
app's local Code mode. The UI showed **Fable 5**, **Effort: High**, **Manual**
permissions throughout review. No fallback model or nested reviewer was used.
This is a GUI review, not a successful Autoreview CLI-helper run. The previous
CLI OAuth failures remain historical facts; the app route worked.

- App session: `PF-30-S01 security code review`.
- App session ID: `local_11570510-8846-4178-8837-761f35b2aa20`.
- Source worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01`.
- Branch: `codex/pf-30-isolated-runtime`.
- Planning HEAD: `399daef151592505f4057d52bbe20fc48a41106d`.
- Reviewed candidate: the uncommitted implementation identified by all 27
  entries in [pre-fix-candidate-files.sha256](pre-fix-candidate-files.sha256), not the planning
  commit alone. All source hashes matched before and after review.
- Isolated copy: `/private/tmp/corbanu-pf30-review.9sEDqj`. It contained candidate
  files, the tracked diff, scope/evidence records, and unchanged network-proxy,
  security-policy and Core effective-policy dependency context. No repository
  hooks, agent instructions, MCP configuration or credentials were copied.

The prompt restricted Claude to read/search within that copy, with no edits,
code execution, installations, containers, network/MCP use or other folders.
One compound command requested a recursive home-directory search. It was denied,
and the same review continued using the included dependency sources. Only a
scoped source listing was explicitly approved once. No implementation changes,
test executions, commits or pushes were made during this review follow-up.

## Findings and validation at initial review

| Finding | Disposition | Evidence and remaining work |
| --- | --- | --- |
| P1: DNS lookup before composed policy denial | Confirmed, open | `browser_policy.rs:72-106` resolves before method/host policy. Move explicit policy denial ahead of lookup and add a deterministic no-resolver-call regression. Redirect prechecks must retain the same ordering. |
| P2: Podman image-ID format mismatch | Confirmed, open | `image.rs:20-22,73-79` requires `sha256:` plus 64 hex characters, while Podman's documented `Id` is bare 64-hex. Accept validated engine-appropriate forms consistently with ownership checks; add image-inspect fixtures and real Podman qualification. |
| P1: trailing-dot deny-policy bypass | Withdrawn by reviewer | `runtime.rs:535` uses `crate::policy::Host::parse`, which calls `normalize_host` and strips trailing dots before deny/allow matching. The initial review confused this wrapper with `url::Host`; no code change is warranted for that claim. |
| P2: 128 KiB engine JSON cap | Unconfirmed qualification risk | Keep bounded output. Measure actual engine payloads before changing limits; no observed failure establishes a defect. |

The Podman format validation used the official
[image-inspect documentation](https://docs.podman.io/en/latest/markdown/podman-image-inspect.1.html),
whose JSON example and ID placeholder distinguish a bare image ID from a
`sha256:`-prefixed digest. This confirms a format incompatibility in the code,
not a completed real-host run.

The unchanged `NetworkProxyState::host_blocked` also performs its private-IP DNS
check before rejecting a host absent from the allowlist. Reordering the new seam
alone fixes explicit deny-list leakage, not that residual behavior. Track its
resolution with the integration owner before claiming DNS-boundary qualification;
`network-proxy/src/runtime.rs` is outside S01's current write scope. Any expanded
implementation scope must be allocated in the plan/sprint before editing.

Additional coverage identified: broker redirect handling, worker request-handler
non-GET/deny/sequence/body-limit branches, and engine endpoint-selection JSON.
The reviewer assumed the unchanged native authority-epoch implementation and
module registration; this review does not replace native-adapter evidence.

## Review artifacts and next gate

- [Initial review accessibility transcript](fable-gui-initial-review.txt):
  superseded where the correction withdraws the trailing-dot finding.
- [Corrected review accessibility transcript](fable-gui-corrected-review.txt):
  final advisory result, no P0, one confirmed P1 and one confirmed P2.

The working implementation was not changed to address these findings in this
review follow-up. After fixes, run formatting/fix tools before affected tests,
refresh candidate fingerprints and obtain Fable High review of the final tree.
The existing 266 Rust and four Python results remain evidence only for the
unchanged draft; they do not demonstrate that the open findings are fixed.
Mac/Linux prerequisites, real containment/egress/lifecycle tests, Windows
qualification, native-adapter evidence and required human/release gates remain
pending. S01 stays `in_progress`; dependent sprints remain gated.

Documentation-only handoff checks passed: plan checker (one active plan), sprint
checker (24 current / 86 archived), `git diff --check`, all 27 candidate source
fingerprints, and the existing docs environment's `mkdocs build --strict`
(output `/tmp/corbanu-pf30-docs.vFsQiK`). No Rust/Python suite was rerun in this
follow-up because the implementation and test source remained unchanged.
