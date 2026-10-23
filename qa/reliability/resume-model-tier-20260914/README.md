# Remembered resume model and service tier — September 14, 2026

Bounded repair under **Shipping MVP — LIVE**: “Multi-provider inference”,
“durable mailboxes, supervision, resume, and recovery”; named providers are
“durably owned by that profile, restored after restart”. User explicitly asked
for fixes, tests, integration merge/push, signed local rebuild and launcher wiring.
No new config/schema, authorization, credential or provider-fallback policy.

Branch `fix/resume-model-tier-20260914`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/resume-model-tier-20260914`, base
`a826527bbc0029db4f5e9e81a097e15d2a2215f6`. Receiving branch:
`integrate/management-workstreams-20260911`. No initiative/sprint slot required.

Ordinary interactive resume remembers the server-acknowledged model/provider,
effort and effective tier using the existing profile-owned configuration writer.
It does not wait for an inference response. Cancellation and failed resume do
not save defaults. Explicit invocation model settings, parent-owned sessions,
and remote connections do not write remembered defaults through this path.
Existing `/model` selection persistence remains unchanged. CLI overrides remain
temporary rather than silently rewriting launch defaults.

Global new-session tiers are not explicit resume overrides. Resume restores the
latest matching durable tier, or standard routing for legacy/no-tier sessions.
An explicit tier or model override remains authoritative; unsupported explicit
tiers retain normal warning-and-omission behavior. Standard routing is saved
explicitly so a subsequent fresh launch cannot resurrect an unrelated fast tier.

Private raw attempts, reviews, test logs, packages and install receipts:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/model-tier-fix.5BeVtZ`.
The reused PTY regression accepts `--remember-model-tier` and optionally
`--startup-select`; it runs a synthetic GPT/fast profile, resumes a synthetic
Fable model/provider, checks saved settings, exits normally and launches fresh.
No live operator profile, credentials, model inference or running session is used.

## Frozen review boundary and limitations

TUI resume/default persistence plus existing server resume-tier restoration;
seven Rust files and focused QA/docs. Review budget five including independent
design, code closeout and evidence check. Design identity:
`01a0a120-e9c7-73e1-b6b1-5fdb995ac102` (fresh, instruction-only isolation).
Original cases preserved in `design-original.md`; clarifications are additive.

Owner-run synthetic PTY evidence is not the mandatory independent, code-blind,
permission-isolated execution. That executor prerequisite, real-provider
inference, native GUI invocation, Windows/Linux runs, both live-repository
release workflows, named-human acceptance and full benchmark cycle remain
unqualified. User-authorized local installation proceeds with these limitations;
no unqualified functional/release acceptance is asserted.

Final test, review and exact-package results are appended before handoff.

## Review and attempt ledger

- Design pass 1: independent frozen cases and additive clarifications.
- Code review 1: accepted tier-only CLI override persistence finding; guarded
  invocation-specific tiers and added an unchanged-config assertion.
- Code review 2: accepted missing durable tier before a paginated compaction
  cutoff. Existing store history API supplies missing metadata without replaying
  full history. A cold paginated resume lacking settings in its suffix can read
  full history; no new public API or history schema is introduced.
- Two-cycle scope audit: both findings are introduced by this repair within the
  resume boundary, not new product goals. Continue bounded corrections.
- Code review 3: clean, Astra High, no actionable findings.
- Tests 1/2: interrupted with broken pipe/exit 241, not counted as passes.
- Tests 3: 16 passed, one embedded-resume test timed out on both allowed tries.
  A process sample located the wait in native FSEvents skill-path registration.
  The generic rollout fixture used `/`; the test now pins a disposable cwd and
  credential-free provider. Native credential denial remains enabled.
- Code review 4: clean after the test-only fixture correction (Astra High).
- Baseline 1: real Fable identifier in a synthetic provider fixture stalled;
  not a live-provider qualification. Baseline 2 uses an explicitly synthetic
  model identifier and reproduces the unsupported global-tier warning.
- Early signed candidate: both in-app and startup-resume synthetic PTY
  workflows pass remembered model/provider/default tier and normal quit/restart.
  This package predates final paginated/test-fixture changes and is not installed.
- Unselected full-suite snapshot artifacts from a concurrent external nextest
  job are preserved under the private `unrelated-snapshots/` tree, not accepted
  or committed as this repair's UI changes.
- Review allowance: five used including design. Under Travis's delegated
  integrator review discretion, authorize one additional independent evidence
  check (total six), solely to audit final package/case coverage; no gate waiver.

## Final source-tree checks

- `tests-4.log`: 111 focused Rust tests passed, 4,175 unrelated tests excluded.
  Both changed crates were selected through `just test --lib --locked --offline`;
  coverage includes tier restoration/override precedence, real embedded-server
  resume, acknowledged-default persistence, invocation-only guards and the resume
  picker keyboard/snapshot suite. The corrected embedded resume completed in
  8.152 seconds. This is not a claim that the full Rust suite passed.
- 28 canonical package-builder tests passed; sprint checker reports 116 current
  and 126 archived records. Formatting preceded final tests; unrelated formatter
  changes were reversed only in this initially clean repair checkout.
- Final source review: Astra High autoreview pass 4, clean, no actionable findings.
- Receiving-build/signing/launch and exact signed-package repetitions are recorded
  in the private installation receipts. Live Fable inference and strict independent
  executor qualification remain explicitly unclaimed.
