# PF-26-S01 review scope

Original request: ensure PF-27 is committed/pushed, then complete PF-26.
PF-27 was pushed at `cb808c30c0058c101597ab2ada3da16238565c5e`.
Only dependency-ready PF-26-S01 is executable; final qualification, actual-key
TUI and human acceptance remain PF-26-S04/S02/S03 dependencies, not this patch.

Target: `codex/pf-26-security-harnesses`, isolated worktree, base `cb808c30c`.
The initial review baseline is 17 changed files, 1,033 added non-test Python
lines (including the compatibility extension and entrypoints), plus fixture,
documentation and test data. No Rust, dependency, native registration, provider,
transport-reconnect, authorization or Windows implementation changes.

Review correctness of the early fixture runners and evidence checker: incomplete
or mixed-candidate proof must not become a pass; fixture preparation is explicitly
pending. All observations are collected by trusted host-side test harnesses.
The checker does not authenticate producers or establish binary build provenance;
PF-26-S04 must collect trusted commands and raw proof for independent review.
Do not broaden this into a signed-attestation service or native runtime consumer.

The local capture sink is synthetic loopback HTTP with no forwarding. A future
PF-13 transport harness owns test TLS and actual-key tmux proof; this patch must
not claim HTTPS/browser/platform qualification. The existing compatibility
runner's full build/test path is retained with frozen-manifest validation; this
sprint exercises its new preparation path, not an expensive final Rust build.

Focused proof before review: Ruff format/check; 37 Python harness tests; plan
and sprint validators; `git diff --check`. Native PF-27 contracts/source selectors
are checked at their pinned historical commit, not claimed as newly executed.

Review cycle 1 accepted one in-scope blocker: native-adapter reports required
the PF-27 expected assertions but omitted their exact contract-test selectors.
The check now requires both. A regression covers the omission on all seven
adapters. The sibling automated-control reports already required those selectors.
No owner boundary or contract was expanded; 38 focused tests pass after the fix.

Review cycle 2 scope audit: one remaining finding is an in-scope scanner bug,
not a new policy or evidence protocol. Hex canary detection was lowercase-only.
Sink scanning and request capture now share case-insensitive hex detection;
raw/base64 matching stays exact. Uppercase and mixed-case regression inputs
cover sinks, request bodies and headers. This is the same owner boundary and
documented encoding contract, well below the 2x scope threshold. No broader
arbitrary-encoding detection is claimed. There are now 39 focused tests.

Review is the standard Codex Autoreview closeout gate. It is not the separately
required named independent security review or human release acceptance.
