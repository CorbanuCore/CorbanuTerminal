# Claude Opus 5 Max review evidence

Review UI: Claude desktop app, fresh chat
[`203390a3-c556-4535-9cda-77b1ae781746`](https://claude.ai/chat/203390a3-c556-4535-9cda-77b1ae781746)

- Visible model: `Opus 5`
- Visible effort: `Max`
- Initial candidate: `1cc3ad92e4de7cbeac40105c4e8d0e9d35ac30c0`
- Canonical packet SHA-256:
  `323a83ff3ca303a14ff3ec6e1e4f207353d6e3b56843b148de2eb8c699f95854`
- Uploaded full-patch text SHA-256:
  `ef8cce2d6a63e138e55b14a3705c9b3226052d0a71d1c86ad02bc62eb1afd334`
- Initial verdict: `Changes required.`

The reviewer attested that it treated the packet as untrusted data, did not
execute the validator, did not contact a network, and did not access anything
outside the uploaded patch. Its UI command panels were used to hash and inspect
the uploaded text in the review sandbox; the declared patch hash matched.

## Initial findings and disposition

| ID | Finding | Disposition |
| --- | --- | --- |
| H-1 | Index-only image verification discarded platform manifest/config identity. | Accepted. The evaluator now binds host architecture to resolved platform, manifest digest and config digest; fixtures cover wrong platform and config. |
| H-2 | Auto-selection could hide preferred-engine tamper/integrity failures. | Accepted. Auto skips only availability/API states; tamper, digest and architecture failures return immediately. |
| M-1 | Preparation blockers, provenance and engine promises were not mechanically enforced. | Accepted. Five mutation checks now prove hard blockers, pending licenses, no engine start, engine-list consistency and provenance digests fail closed. |
| M-2 | Evidence paths permitted traversal/symlink escape. | Accepted. Canonical POSIX syntax, lexical/resolved containment and final-symlink rejection are enforced with six path checks. |
| M-3 | Worker mutation could be recommended before a lock-bound reinspection; repeated evaluation only proved determinism. | Accepted. Worker actions and reuse require a Corbanu lock bound to the exact post-lock owned-worker snapshot; stale reinspection is blocked. Evidence now calls the repeat a deterministic replay check. |
| M-4 | Duplicate PF-31 workers across Podman and Docker were invisible. | Accepted. Installed supported engines are inventoried before selection and cross-engine duplicates fail visibly. |
| M-5 | Evidence named an older candidate than the review target. | Accepted. Final evidence will name the reviewed/final commits and cross-host rerun bundle explicitly. |
| L-1/L-2 | License prose understated missing-license/Rust/GPL scope and conflated catalog records with unique packages. | Accepted. The inventory now separates records from approximate unique packages and names cargo and reciprocal-license gaps. |
| L-3 | Runtime list omitted Node/Debian and included an uncorroborated ffmpeg revision. | Accepted. Node and Debian are recorded; the uncorroborated ffmpeg revision is removed. |
| L-4 | Resource metrics were not directly comparable. | Accepted as documentation. Metric kinds remain explicit and activation still requires PF-31-S01 workload measurements. |
| L-5 | Exact-version validation accepted range syntax. | Accepted. Dependency versions now use name-aware exact numeric patterns. |
| L-6 | Out-of-tree CLI inputs could raise an uncaught exception or access outside scope. | Accepted. Manifest, fixture and snapshot inputs resolve strictly inside the repository. |
| L-7 | Upstream image contains package installers and scripting runtimes. | Downstream hardening input. Protected activation remains disabled; the locked minimal Corbanu rebuild remains a PF-31-S01 hard blocker. |

`submission.png` records the uploaded packet and selected model/effort before
submission. `initial-verdict.jpeg` records the completed initial verdict.

## Corrected-candidate follow-up

Follow-up UI: Claude desktop app, chat
[`32b7703c-d0f0-49ca-8875-e5e189ac2c10`](https://claude.ai/chat/32b7703c-d0f0-49ca-8875-e5e189ac2c10)

- Visible model/effort: `Opus 5` / `Max`
- Reviewed candidate: `9ab30bc79ccdaf65a65f03fcf85c0464016a6d67`
- Baseline: `8162a8cfb2ccc31021c3fa3492c5e7db33674415`
- Inline follow-up SHA-256:
  `dbec365285367258bebc6b3d7c1693d278446f975e6f5e78c06fa11ec4d2a8d5`
- Verdict: `CHANGES REQUIRED`

The reviewer again attested that it executed nothing, used no network or tools,
and accessed no private files or memory. It withdrew its earlier F1 malformed-
JSON claim after the transport-only `TESTENTRY` corruption was explained; all
repository JSON had parsed and the full fixture suite had run on three hosts.
`corrected-submission.jpeg` and `followup-verdict.jpeg` preserve the visible
submission/review state.

| ID | Finding | Final disposition in `cf05164e7d5c21a8e716a36c70455c5cff26f5ca` |
| --- | --- | --- |
| R1 | Explicit selection could recommend creating a second worker while another engine owned one. | Accepted. Explicit selection now returns `owned_worker_on_other_engine`; fixture 25 proves the deny-without-fallback result. |
| R2 | Auto precedence depended on a reason-code remediation allow-list. | Accepted. Apart from the narrow availability/API skip set, auto classification is status-driven; remediable results are remembered while all engines are scanned. |
| R3 | An explicitly unheld lock object was misreported as a broken lock. | Accepted. Missing and `corbanu_owned != true` lock encodings both produce acquire-lock guidance; fixture 26 proves the object form. |
| R4 | Canonicalization branches lacked real leaf/ancestor symlink coverage and the message was too narrow. | Accepted. A shared canonical resolver rejects both forms, with real symlinks created only in an OS temp directory so read-only checkouts remain supported; ten path checks pass on all hosts. |
| R5 | Dependency version parsing inferred schemes from names. | Accepted. Every dependency declares `package`, `os` or `revision` and validation keys off that field. |
| R6 | Fixture JSON loading sat outside the path-context error boundary. | Accepted. Loading, shape checks and evaluation now share the same `ContractError` context. |

The subsequent Codex review found and closed two related regressions: auto mode
now prioritizes remediation of an existing stopped worker before any create
action (fixture 27), and the symlink tests require no checkout writes. Final
Autoreview is clean at confidence 0.84. The exact final bundle
`5753260131b1c4fada152843a9ed792bbc5edb4b6d3e0634e90ee49d05661c0b`
passed 27 fixtures, 27 deterministic replays, five manifest mutations and ten
path checks on macOS, Ubuntu and Windows, plus a read-only extracted checkout.
No clean Opus verdict beyond the recorded `9ab30bc79` follow-up is claimed; the
integration-owner disposition is based on fixing every recorded finding and the
independent clean final review/cross-host evidence.
