# Round-four rolling security handoffs

Dispatch base: `43d2d86488d5c1b2eb5cbc401ee8371dbdb76bf4`.

Integration owner: Codex ingress/classifier lane.

PF-35 external qualification continues independently and is not modified by
this packet. It still occupies one formal active-sprint slot. PF-22-S02 and
PF-27-S04 started in parallel. PF-22-S02 is now integrated and archived, and
PF-30-S01 has rolled into the released slot from base `b457249aa`.

| Lane | Sprint | Owner | Handoff |
| --- | --- | --- | --- |
| protected-runtime — completed | PF-22-S02 | `/root/pf22_protected_runtime` | [Protected runtime](protected-runtime.md) |
| isolated-broker | PF-27-S04 | `/root/pf27_isolated_broker` | [Isolated broker](isolated-broker.md) |
| source-envelope | PF-30-S01 | `/root/pf30_source_envelope` | [Typed source envelope](source-envelope.md) |

All build, cache, temporary, review and TMUX output belongs beneath
`/Volumes/CorbanuDrive/Corbanu/.codex-work/<lane>/`. The shared checkout's
unrelated large-image work is outside this packet and must remain untouched.

## Integration order

```text
PF-22-S02 candidate ──► integrate/archive ──► allocate PF-30-S01
PF-27-S04 candidate ──► rebase on PF-22 ────► register/integrate
PF-30-S01 candidate ────────────────────────► integrate after its own proof
PF-35 external campaign ────────────────────► independent evidence handoff
```

PF-22 exclusively owns the shared Core security module/manifest/lock surfaces
during candidate development. PF-27 does not edit them; its Core registration
is serialized after PF-22 lands. The integration owner alone changes active-plan
allocations, global indexes, MkDocs, archive transitions, `humanTest.html`,
`securityProgress.html`, root Cargo/Bazel files and shared locks unless a sprint
front matter explicitly assigns one of those paths.

## Common closeout

- Run fix/format before final affected tests and inspect the final diff.
- Run plan/sprint governance and `git diff --check`.
- Use the Rust TMUX harness with `RUST_LOG=trace`, explicit CorbanuDrive
  `log_dir`, cache and temporary roots; send text and Enter separately.
- Run a read-only Claude Opus 5 Max review through Corbanu Terminal in TMUX.
  Verify findings, fix only in scope, rerun proof and rereview to no actionable
  P0/P1/P2 findings.
- Keep raw credentials, private corpus data, signing keys, production secrets
  and funds out of Git, prompts, transcripts and evidence.
- Do not attempt final Windows or Linux qualification until the integration
  owner confirms the user has switched tailnets. Local macOS construction and
  deterministic tests continue meanwhile.
