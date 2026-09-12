# Human-memory fixture review scope

Requested reviewer for review3: Astra High. Review4: Fable5.1High through the
approved Corbanu/private-TMUX wrapper. Original memory track already consumed
reviews1–2; this support does not create a new five-review budget. No nested
reviewers. Invocations/results belong in qualification.md, not inferred here.

Base `b64d7390d4ef86b9604d56536e095ac29450423b`; branch
`test/security-human-memory-fixture`. The allocation packet9ceac3333 classifies
this as routine test-only support for the existing archived PF30S04 candidate.
No product binary, security boundary, config schema, authentication, storage
schema or archived sprint source changed. Shared suite/nextest registration was
coordinator-owned. Other base-to-branch humanTest/packet changes are allocation
and honest status documentation, not new product behavior.

Review the cohesive helper and test lifecycle, not just the last patch:

- Small typed TMUX owned-attachment/liveness accessors and cleanup regression.
- `memory_human_fixture.rs`: ignored opt-in manual entry, pinned candidate/runner
  identity, fresh synthetic home/history/eligibility store, two loopback mocks,
  source-specific request/DB evidence and bounded human state machine.
- Actual-key rehearsal: startup, explicit `/providers` replacement, `/model`
  effort selection, pending owner exit/restart, cancel and timeout.
- QA `rehearse-pinned.py`: subprocess invocation of frozen ignored entry after
  releasing build lock, actual startup keys and cancellation, owned cleanup.

All child Corbanu credentials are cleared and replaced by synthetic values;
unrelated provider identities are made inactive only in the disposable fixture
home. The operator can attach only via the generated owned-session command.
No watchdog is disabled, no personal home accepted, no real key or private
history is required. The manual fixture permits at most600seconds plus bounded
setup, with120seconds for pending responses; nextest manual timeout660seconds.
Machine proof never sets human_acceptance true.

Known adjacent direct-select/provider-label/catalog-sync issues are documented
in adjacent-model-picker-findings.md and intentionally unpatched. Do not demand
a product fix or claim this is direct `/model` provider-switch parity. The
approved supported journey is `/providers` replacement followed by model-effort
selection with real B routing and old-source zero-success proof.

Stages: tiny shared harness accessors/tests; one cohesive fixture module
(entry/rehearsal tests plus state logic); independent frozen-entry QA driver and
operator docs. Reject speculative architecture rewrites. Verify concrete
cleanup, timing, false-positive evidence, unsafe external routing, test-runner
pinning or command-ownership bugs inside this scope. If a proposed correction
needs a product API/config/schema, report follow-up rather than widening scope.
