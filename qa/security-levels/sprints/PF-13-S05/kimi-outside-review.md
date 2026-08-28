# PF-13 Kimi outside-expert review

Date: 2026-08-27 Arizona time (2026-08-28 UTC).
Status: review completed; PF-13 qualification remains **not ready**.

## Authority and frozen source

Travis explicitly replaced the interrupted Fable High review with Kimi 3.0
through Corbanu Terminal. This is qualification/evidence for PF-13-S05 in the
existing product initiative, not authorization for implementation repairs.
Product: **Required trust boundaries** — “Credentials are referenced by label
and resolved only inside a trusted execution boundary.” Plan:
`docs/plans/active/p0-security-levels.md`; PF-13-S05 remains `in_progress`.

Reviewed source: `044491b8b02b24a65a84e8da61619d3444e63fe0` on
`feat/pf-13-s02-scoped-vault-resolver`. All work to date was merged before
review; the [prior integration record](fable-outside-review.md) identifies both
parents and the evidence preserved. HEAD at review start was
`cd048252d421558a4f18dbaaee6cc9bcdf1f4610`, a subsequent evidence-only commit;
`git diff 044491b8b HEAD -- codex-rs scripts` was empty.
Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02`.

The reviewer reads `/tmp/corbanu-pf13-kimi-review.9R9N0P`, an isolated
`git archive` export of the merged source, with original PF-13 commit diffs,
contracts and existing evidence. Repository instructions, hooks and local
configuration are excluded. The [frozen scope](kimi-outside-review-scope.md)
and [35-file manifest](kimi-review-candidate.sha256) identify the review.
All 35 hashes matched before submission and the reviewer independently verified
them. No implementation source was edited during review.

## Reviewer and interactive invocation

- Client: Corbanu Terminal 0.1.35, pre-existing local executable
  `/Users/travisgood/Documents/ChatGPT/corbanu-security-levels/codex-rs/target/debug/corbanu`.
- Client SHA-256: `9201d1a1d3623733f3ce163ba6652e1775293a9936392b009d91379583e2aa17`.
- Model: `moonshotai/kimi-k3`; provider: `openrouter`; effort: `high`.
- Session: `01a0469a-3d91-7ba1-af03-da0cc86f7c61`; tmux: `pf13-kimi-review`.
- [Visible start checkpoint](kimi-review-start-screen.txt) confirms model,
  effort, client version and snapshot directory. Session metadata independently
  records OpenRouter and the same directory/client version.
- This client is the reviewer host, not a newly built PF-13 qualification
  candidate. Its source commit is not asserted from its version alone.

```sh
env TERM=xterm-256color RUST_LOG=trace \
  /Users/travisgood/Documents/ChatGPT/corbanu-security-levels/codex-rs/target/debug/corbanu \
  --no-alt-screen -C /tmp/corbanu-pf13-kimi-review.9R9N0P \
  -m moonshotai/kimi-k3 -c model_provider='"openrouter"' \
  -c model_reasoning_effort='"high"' \
  -c log_dir='"/tmp/corbanu-pf13-kimi-logs.NKV0RO"' \
  -c project_doc_max_bytes=0 -c web_search='"disabled"' \
  -s read-only -a never \
  --disable apps --disable hooks --disable plugins --disable remote_plugin \
  --disable multi_agent --disable browser_use --disable browser_use_external \
  --disable browser_use_full_cdp_access --disable in_app_browser \
  --disable computer_use --disable image_generation --disable skill_search \
  --disable skill_mcp_dependency_install --disable workspace_dependencies \
  --disable shell_snapshot
```

The user approved the macOS keychain request and personally completed native
OpenRouter onboarding. No credential was supplied in the review prompt or
copied into the export/evidence. Raw trace/authentication logs are not committed.
The scope prompt was sent with `tmux send-keys -l`, then Enter in a separate
tool call. Read-only inspection is allowed; execution of repository code,
tests/builds/installers, access to homes/credentials and nested reviewers are
forbidden. This uses the expressly requested Corbanu TUI, not the Autoreview
helper's default engine. No fallback model is authorized.

## Result and validation

The session completed at `2026-08-28T05:11:01.037Z`, after starting at
`2026-08-28T04:22:22.002Z` (48 minutes 39 seconds). It made 91 read-only
`exec_command` calls. The controller requested a bounded final assessment after
extensive inspection; this was delivered normally after the next tool call,
without interrupting or substituting the reviewer. Kimi ran no product tests,
builds or nested reviews. Its statement “No code was executed” means no product
code or tests; its inspection did use shell commands and evidence parsing.

Preserved artifacts:

- [Verbatim reviewer response](kimi-review-response.md), including its JSON.
- [Parsed reviewer JSON](kimi-review-result.json), without controller corrections.
- [Frozen scope](kimi-outside-review-scope.md), [file hashes](kimi-review-candidate.sha256),
  and [interactive start checkpoint](kimi-review-start-screen.txt).

Response SHA-256: `0832f16dea57b6ace4b9e56faf0e515cfc11fccd70708f0fc27a8adf0521b4aa`.
Parsed JSON SHA-256: `89cb283612cbf379d62afa51a2158d4eafa01b7cc6b62123b8b0c63d4fbe452c`.

Kimi returned two P2 and four P3 findings. Its verdict is
“qualified-with-gaps leaning not-ready”; the controller records **not ready**,
not a qualified security pass. No reachable P1/raw-secret exfiltration defect
was demonstrated by this static review. That is not proof of absence, especially
where native integration is missing. Kimi's positive blanket statements about
all logs, panic containment and secret allocations are narrowed below.

### Finding dispositions

These are controller dispositions, not rewritten reviewer conclusions. Paths
and line numbers below refer to the frozen source.

| Finding | Disposition | Evidence and next action |
| --- | --- | --- |
| F1 / P2: no production scoped-route call site; overstated completion | Accept evidence correction; known PF-23 integration gap, not newly authorized implementation | `core/src/config/network_proxy_credential.rs:175` and the native `NetworkProxySpec::start_proxy` path confirm the scoped builder is not wired into a real session. S01–S05 records and the plan now distinguish invoked library seams from native profile enforcement. The separate S04 CLI raw-export denial is real. Absence of route installation does **not** establish Kimi's universal claim that every child receives a raw key; that depends on actual authentication/environment. Native issuance, child isolation and live transport proof remain at the PF-23/PF-26 join. |
| F2 / P2: no integrated-merge qualification; full Core red | Accept qualification gap, with corrections | Historical platform results retain their original commits. Hash/source comparison verifies reviewed source identity, but does not replace integrated runtime tests. The full Core result is 3,261/3,396 passed, 135 failed; 13 credential-named tests passed. The “29 Core credential tests” in the prose confuses PF-30's 29 Core security tests with PF-13. Triage, repair/prerequisites and a clean full rerun are required; attribution as unrelated is not a waiver. |
| F3 / P3: environment-selected home/policy | Accept adversarial-test and integration follow-up; no bypass demonstrated | `cli/src/main.rs:2453` honors the selected home, and `secrets/src/lib.rs:193` derives the keyring account from its canonical path. Copied-vault/alternate-home and symlink variants need explicit tests. Do not hardcode a “real user home”: legitimate custom homes must continue to work. Bind canonical vault/policy identity and inherited authority at the authorized integration boundary. The static path analysis is not a successful native bypass test. |
| F4 / P3: plaintext header temporary | Accept hardening follow-up, not demonstrated disclosure | Actual OpenAI line is `network-proxy/src/credential_broker/providers/openai.rs:40` (GitHub:50). `format!` creates a non-zeroized temporary and the header is not marked sensitive. Vault source-allocation zeroization does not cover every transport copy. Review temporary wiping and sensitive flags; `is_sensitive()` controls Debug treatment, not memory wiping or core-dump secrecy. No current product leak was reproduced. |
| F5 / P3: dummy leaks real-key shape | Reject for PF-13 scoped route | `network-proxy/src/credential_broker.rs:116` derives the scoped dummy from the fixed-length public capability id, **not** the vault secret. The shaped-real-key path is existing legacy/Permissive behavior, outside this scoped finding. JSON also has a duplicated `providers/` path segment; actual helper is `network-proxy/src/credential_broker/providers.rs`. Preserve compatibility; do not change it under this review. |
| F6 / P3: label normalization comment | Confirm inherited documentation mismatch; routine follow-up | `vault/src/lib.rs:599` claims lowercasing, but line 620 returns the trimmed case-preserving label. Available blame already contains this at pre-PF-13 boundary commit `058939597`. No authorization or normalization behavior change is indicated. JSON line 545 is stale. |

### Follow-up ownership and coverage limits

- PF-13-S05: correct evidence language; resolve C1/C2 below; repeat the
  affected final-tree qualification and clean full Core suite. A test's
  self-reported count is not independent proof that its assertions are strong.
- PF-23 with PF-26 integration qualification: real session issuance and scoped
  route enablement, dummy-only child environment, live CONNECT → MITM → upstream
  request, transport-level Authorization collision/replay denial, and canonical
  home/policy inheritance. Do not silently pull these implementation tasks into S05.
- Native shared-contract integration: bind runtime/policy epochs and revocation
  consistently when composing profiles. Other providers, durable receipts and
  cross-process authority are deferred, not implicit requirements of this review.
- F4 hardening requires an appropriately scoped implementation record; F6 is a
  routine comment correction. Neither is implemented here. An independent
  assertion manifest can aid audit, but cannot by itself prove test semantics.

Kimi's harness paragraph points to “Finding 5” for an unrelated concern; its
F5 is the dummy-shape finding. The actual transport-coverage limitation is
tracked here separately. Reviewer-host TUI use is not PF-13 feature TUI proof.

### C1 — Controller-only diagnostic: truncated output escapes scanning

This finding was independently reproduced by the controller during the review,
not supplied to Kimi. `scripts/security_credential_canary.py:246` truncates
stdout/stderr through `bounded_output` before `validate_probe_output` scans
them. With all required test names and the success summary before the 1 MiB
limit, a synthetic credential-shaped marker after that limit is discarded.
The full string fails scanning, but the retained string passes validation.
This is a qualification-harness defect, not a reproduced product-secret leak.

Reproduction run with `python3 -B` against the unchanged merged source:

```python
from scripts.security_credential_canary import (
    PROBES, MAX_CAPTURE_BYTES, CommandResult, QualificationError,
    assert_secret_free, bounded_output, validate_probe_output,
)

probe = PROBES[0]
synthetic = "".join(f"test {name} ... ok\n" for name in probe.expected_tests)
synthetic += f"test result: ok. {len(probe.expected_tests)} passed;\n"
synthetic += "x" * MAX_CAPTURE_BYTES + "\nsk-" + "review-synthetic-not-a-credential\n"
raw_detected = False
try:
    assert_secret_free(synthetic, "synthetic full output")
except QualificationError:
    raw_detected = True
count = validate_probe_output(
    probe, CommandResult(["synthetic-only"], 0, bounded_output(synthetic), "")
)
assert raw_detected and count == len(probe.expected_tests)
```

Observed: full output detects the marker; truncated output accepts all seven
expected tests. No subprocess or real credential was used. Recommended repair:
scan complete output before limiting retained evidence, preferably with bounded
streaming scans; fail closed on capture overflow/incomplete scanning. Add stdout
and stderr boundary/overflow regression cases. No repair was applied.

### C2 — Controller-only diagnostic: caught panics still reach the panic hook

`vault/src/capability.rs:130–143` catches the callback panic and returns a stable
error. It does not prevent Rust's panic hook from running first. The native
hook at `tui/src/lib.rs:1412–1416` formats the panic information into tracing and
then calls the previous hook. The existing Vault test at
`vault/src/capability_tests.rs:186` panics with static non-secret text; the Core
canary at `core/src/security/credential_capability_tests.rs:611` also uses a static
panic and scans the formatted stable error as its crash output. Neither proves
that a secret-bearing panic payload is absent from real stderr/logs.

A standalone [Rust semantic diagnostic](kimi-panic-hook-diagnostic.rs), using
only a synthetic marker, caught the panic while its hook had already observed
that marker. It was compiled with `rustc --edition=2021 -C panic=unwind` and run
from `/tmp/corbanu-pf13-panic-proof.3Kl4Ns`; output:

```text
PASS diagnostic: panic was caught, but its hook already observed the synthetic marker
```

This is a confirmed qualification/containment gap, **not** a reproduced native
credential leak or a demonstrated secret-bearing panic in the current concrete
header callback. Kimi's broad panic/log confidentiality assurance is therefore
not accepted. Add a subprocess canary with a secret-bearing callback panic under
the actual production panic-hook configuration, checking stderr and logs as
well as returned errors. Any required containment/redaction change needs an
authorized owner/sprint and concurrency-safe design; globally swapping hooks
around callbacks is not assumed safe. No implementation repair was applied.

## Final evidence-only checks

- `python3 -B docs/plans/check.py`: passed; active 1/2.
- `python3 -B docs/sprints/check.py`: passed; current 24, archived 86.
- `python3 -B -m unittest scripts.test_security_credential_canary`: 6 passed.
  These existing tests do not cover C1/C2; passing them does not close the findings.
- `git diff --check`: passed.
- Strict MkDocs build: passed, output `/tmp/corbanu-pf13-kimi-docs.B7FwD8`;
  existing excluded-archive informational messages and theme advisory remain.
- Frozen snapshot SHA-256 verification repeated after review: 35/35 matched.
- `git diff 044491b8b HEAD -- codex-rs scripts` and working-tree source diff:
  empty. Only review/evidence/plan/sprint records changed, plus the standalone
  synthetic diagnostic under QA. No product implementation fix, commit or push.

No new full Rust/Core or platform qualification run is claimed by this record.

## Qualification limits

This invocation is an outside code review, not a new full-Core, canary,
transport, live-repository or release acceptance run. Earlier Mac/Linux/Windows
reports retain their original candidate identities. In particular, the 135
recorded full-Core failures still require triage and a clean full rerun.
PF-23 profile composition, applicable integrated TUI/live-repository proof,
Travis Good's human acceptance, due benchmarks and release gates are not closed
by this review. PF-30's Windows browser gap is separate from PF-13's historical
Windows credential canary.
