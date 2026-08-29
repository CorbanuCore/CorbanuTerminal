# PF-13 Outside Security Review — commit `044491b8b02b24a65a84e8da61619d3444e63fe0`

All 35 frozen files verified against `PF13_REVIEW_FILES.sha256` (35/35 OK). Review was static, read-only inspection of the exported tree, the six original diffs, and the supplied QA evidence. No code was executed.

## Scope verdict up front

I found **no reachable raw-secret exfiltration defect** in the shipped PF-13 code paths. The boundary design (opaque bearer in Core, secret-free authority object, zeroizing vault callback, one-shot proxy injection, fail-closed denial matrix) is internally sound and unusually well tested at the seams it exercises. However, three things keep this from being "ready": the strongest evidence does not cover the reviewed commit, the entire scoped path has **no production call site**, and the full Core suite is red. Details below.

---

## Review questions, answered from the trace

**1. Capability lifecycle.** Traced `CredentialCapabilityStore` (core/src/security/credential_capability.rs): 256-bit OS-entropy bearer, `CapabilityId = SHA-256(domain ‖ token ‖ request_digest)`, hard cap 1,024, atomic remove-under-write-lock consumption, revalidation of time/revocation on issue/consume/purge, forged/mismatched requests denied without consuming the valid entry. The `CredentialCapabilityRequest` contract (security-policy/src/credential.rs) genuinely binds actor chain (human-rooted, agent-current), session/task/purpose/operation, label/scope, method, canonical lowercase DNS host, nonzero port, canonical origin path (no `?`, `#`, `\`, dot-segments), expiry, and revocation generation, and is digested canonically. **This layer is solid.**

**2. Vault → transport.** The chain `store.consume → AuthorizedCredentialCapability::into_vault_ref → VaultCredentialRef → Vault::with_scoped_credential → VaultNetworkCredentialResolver → ScopedCredentialRoute → CredentialBroker::inject_request_headers_for_request → mitm.rs forward_request` was traced end to end. Raw secret touches only: `Zeroizing<String>` in the vault, the borrowed `&str` inside the synchronous callback, and the outgoing `Authorization` header. Errors, Debug, receipts, and logs are secret-free (receipt schema has no label/value fields by construction).

**3. Callback/transport assumptions.** Panic containment (`catch_unwind` + discarded payload), poisoned-lock recovery, MITM-hook Authorization collision guard, host-mismatch re-check, and one-shot `used` flag under the broker write lock all check out. Query-mutation cannot bypass the path check because `path_and_query` (mitm.rs:647) feeds the exact-match comparison. The plaintext fallback is closed: `scoped_credential_route_matches_host` blocks absolute-form plaintext to `api.openai.com` before legacy injection (http_proxy.rs:810–811), and non-TLS bytes on a CONNECT to the host are forwarded **without** injection (dummy only, upstream rejects). No redirect-follower exists in the MITM path, so the "redirect reuse" denial is accurate. One hygiene exception — see Finding 4.

**4. Moderate/Aggressive bypass closure.** The `max(persisted, requested)` gate at cli/src/main.rs:2467 correctly defeats `-c security.level=` downgrade, and the denial runs before label normalization/storage/stdout. Residual exposure is env-mediated (`CODEX_HOME`/`CORBANU_HOME`) — see Finding 3. Permissive legacy behavior is preserved (`reveal_for_programmatic_use` unchanged; broker virtualization behavior unchanged when no scoped route is installed).

**5. Harness soundness.** The 41-check canary is real but is a **harness seam, not the product seam**: the "exact outgoing request capture" is an in-process `HeaderMap` assertion in a Core unit test calling `inject_request_credentials` directly — no test drives a scoped route through a live CONNECT → MITM → upstream path. Probe selection is name-matched against source files that are hashed, which is good; but the harness's own trust anchor (it builds and runs the tests it asserts on) means a broken assertion helper inside the test tree would self-certify. See Finding 5 and Follow-ups.

**6. PF-27 interaction.** PF-13 reuses `RevocationState`, `BoundedGrant`, `ActorChain`, receipts — no duplicated policy ownership, which is good. But `AuthorityEpoch`/`EpochBoundGrant` (runtime nonce + policy revision) are **not** integrated into the credential path: a credential authority pins only `revocation_generation`, so a policy-revision change (e.g., security-level transition) does not by itself invalidate outstanding credential capabilities unless it also bumps revocations. The revocation read-guard linearization across the callback is correctly implemented.

**7. Readiness.** See verdict.

---

## Findings

### P1 — none reachable
No currently reachable P1 defect. The reason is itself the dominant issue (Finding 1): the scoped path is unreachable in production.

### P2-1 — Scoped credential path is dead code in the product; completion claims risk overstating shipped coverage
- **File/line:** `codex-rs/core/src/config/network_proxy_credential.rs:174–229` (`#[allow(dead_code)] build_state_with_scoped_openai_credential`); `codex-rs/core/src/security/credential_capability.rs:139` (`CredentialCapabilityStore`, `pub(crate)`, never constructed outside `#[cfg(test)]`); `codex-rs/network-proxy/src/config.rs:162` (`credential_broker: false` default); `features/src/feature_configs.rs:256+` (`NetworkProxyConfigToml` exposes **no** `credential_broker` field, so users cannot enable it).
- **Preconditions:** Any real Moderate/Aggressive session today.
- **Failure path:** Nothing in native (non-test) code ever calls `issue()`, `consume()`, `into_vault_ref()`, or `build_state_with_scoped_openai_credential*`. The broker flag can only be turned on via `set_credential_broker_enabled`, called only from tests. Therefore in the shipped product, Moderate/Aggressive agent OpenAI traffic still carries the **raw** `OPENAI_API_KEY` in the child environment; the label-referenced, brokered boundary the sprint records describe exists only as a tested-but-unwired library.
- **Assessment:** Per the plan, S01–S04 deliberately built internal slices and S05 owns qualification, so this is partly a **deferred PF-23 integration seam, not a hidden defect**. But the S02–S05 evidence language ("Vault now accepts…", "Core can install…", "resolved it only inside the trusted proxy injection callback") reads as shipped behavior. It is not. Any completion statement must be scoped to "the boundary exists and passes tests when invoked," not "credentials are resolved inside the trusted boundary" in the running product.
- **Minimal remedy:** Either (a) wire the issuer (human-mandated issuance → store → route install → broker-enabled proxy) under a guarded feature before claiming PF-13 behavioral completion, or (b) annotate all S01–S05 records and the plan dashboard row with an explicit "no production call path; evidence covers harness-invoked seams only" qualifier.
- **Regression test:** An integration test that starts from the real session/spawn path (not `NetworkProxySpec` test helpers) and proves a Moderate session's child env contains a dummy and that a live CONNECT to `api.openai.com:443` through the running proxy receives the vault value exactly once.
- **Confidence:** high (verified by exhaustive `rg` for call sites across `codex-rs/`).

### P2-2 — Qualification evidence does not cover the reviewed merge; full Core suite is failed
- **File/line:** `qa/security-levels/sprints/PF-13-S05/credential-canary-report-windows.json` (`source_commit: ea7d4bec72…`); macOS/Linux reports at `55025dd42…`/`27b738ab8…`; S05 evidence records 3,261/3,396 Core pass with **135 failures** at `55025dd42`, including 123 failures in the `codex-core::all` integration binary, explicitly unattributed.
- **Failure path:** The reviewed commit `044491b8` is a merge whose only post-merge evidence is six Python harness tests and structural doc checks. No canary run, no Rust test run, and no Core suite exists at the reviewed hash. The scope's own claim "Source merge unchanged from tested tip" is asserted, not evidenced by any test at this commit.
- **Minimal remedy:** Re-run the six-probe canary plus at minimum the 29 Core credential tests at `044491b8` on all three OSes, and triage the 135 Core failures to either fix or proven-unrelated attribution before qualification sign-off. Do not let the 13/13 credential-named pass stand in for the suite.
- **Regression test:** CI gate: canary workflow must run on the merge commit itself (it is `pull_request`/`push`-pathed, which is fine) and the report must be re-issued per reviewed hash; add the Core suite result as a required check for the security plan's release gate.
- **Confidence:** high (this is an evidence gap, not speculation).

### P3-3 — `auth-helper` posture read is env-mediated; protection depends entirely on keyring keying
- **File/line:** `codex-rs/cli/src/main.rs:2453–2472` — both `ConfigBuilder::default().build()` calls resolve `CODEX_HOME`/`CORBANU_HOME` from the process environment; the vault is then opened at `config.codex_home` (line 2468).
- **Preconditions:** Agent capable of spawning processes with a controlled environment and of staging a directory it controls.
- **Failure path:** An agent points `CODEX_HOME` at a directory containing a permissive `config.toml`. The `max()` gate is then meaningless because the "persisted" posture is read from the attacker-chosen home. Containment currently rests on `compute_keyring_account` hashing the **canonicalized** home path (secrets/src/lib.rs:193), so a foreign home cannot decrypt the real vault, and a symlink to the real home canonicalizes back to the real (protected) config. This holds, but it is a single, untested, undocumented load-bearing assumption.
- **Minimal remedy:** Resolve the posture from the real user home independent of env overrides (or bind the gate to the same canonical home the keyring account derives from), and document the invariant.
- **Regression test:** CLI test: protected real home + `CODEX_HOME=<tempdir with permissive config.toml>` + a copied vault directory → must fail closed (it should, via keyring miss); symlinked-home variant must also fail.
- **Confidence:** medium (bypass not demonstrated; reliance on keyring keying verified in source).

### P3-4 — Raw secret passes through non-zeroized temporaries at header construction
- **File/line:** `codex-rs/network-proxy/src/credential_broker/providers/openai.rs:50–52` (`format!("Bearer {value}")` creates a plain `String` containing the secret, copied into `HeaderValue`, then dropped unzeroed); same pattern in `providers/github.rs:50–52`.
- **Failure path:** On every scoped (and legacy broker) injection, the secret exists in at least one non-`Zeroizing` heap allocation that is freed without wiping, weakening the S02/S03 "source allocation is cleared" lifetime claim (which is about the vault allocation, not the injection temporaries). Process-memory disclosure (core dump, crash artifact) could retain it.
- **Minimal remedy:** Construct the header value inside a `Zeroizing<String>` (or write into a stack buffer moved directly into `HeaderValue::from_maybe_sensitive`/`from_str` from a zeroized buffer) and mark the `HeaderValue` sensitive.
- **Regression test:** Test-only callback that captures the allocator-visible temporaries is impractical; instead assert `HeaderValue::is_sensitive()` post-injection and add a code-level lint/review note; optionally a debug-build canary that scans freed-budget allocators is overkill — sensitive-flag assertion suffices.
- **Confidence:** high (code read directly).

### P3-5 — Dummy-value shaping leaks secret structure to the child
- **File/line:** `codex-rs/network-proxy/src/credential_broker/providers.rs:96–110` (`shaped_dummy_value` copies every non-alphanumeric byte of the real secret into the dummy at identical positions and matches its length).
- **Failure path:** The untrusted child learns the real credential's exact length and punctuation layout (e.g., `sk-proj-` segment structure). Low-value metadata, but it is gratuitous and contradicts a strict reading of "opaque reference."
- **Minimal remedy:** Fixed-format random dummies (constant prefix + fixed-length random tail) independent of the real value's shape.
- **Regression test:** Assert dummy length/punctuation is independent of real-value length/punctuation across a matrix of real-value shapes.
- **Confidence:** high.

### P3-6 — Stale doc comment: `normalize_label` does not lowercase
- **File/line:** `codex-rs/vault/src/lib.rs` — `normalize_label` doc says "trimmed and lowercased"; the implementation only trims (returns `trimmed.to_string()`). Case-distinct labels are distinct credentials (as `secret_name_for`'s comment intends), so behavior is consistent, but the comment misdescribes a security-relevant canonicalization function.
- **Minimal remedy:** Fix the comment.
- **Confidence:** high.

### Deferred (not PF-13 defects)
- No `AuthorityEpoch`/policy-revision binding on credential authorities — PF-27 epoch seam, explicitly later.
- Scoped route is OpenAI-only; GitHub/other providers remain legacy-virtualized — provider expansion is later work.
- Receipts are `tracing::info!`-only, not durably persisted — audit persistence is outside PF-13's slice.
- In-memory store ⇒ no cross-restart replay (correct), and no cross-process revocation propagation beyond the shared `RwLock` — distributed authority is later.

---

## Follow-ups (testing/architecture, distinct from defects)
1. Add an end-to-end test through a real CONNECT → MITM tunnel with an installed scoped route (current coverage invokes the broker method directly).
2. Canary harness trusts the test tree it builds; add an independent expected-assertion manifest or spot-check step so a weakened assertion can't self-certify.
3. Test the MITM-hook-collision guard (`hook_actions_touch_authorization`) at the `forward_request` level, not just as a pure function.
4. Decide and document the intended behavior when `credential_broker` config is absent vs. Moderate/Aggressive, before PF-23 wiring; today the flag is unreachable from config.
5. Consider a `Zeroizing` audit of the whole injection path (broker records are done; header temporaries are not — Finding 4).
6. Plan-level: require canary + Core-suite evidence at the exact reviewed/merged hash for any future sprint claiming "merged" status.

## Coverage actually inspected
All 35 frozen files; native call chains `cli auth-helper → vault`, `store → vault ref → resolver → route → broker → mitm/http_proxy/upstream`; proxy plaintext/CONNECT/DetectTls paths; providers (openai, github); revocation/authorization/grant/mandate contracts; canary Python harness + workflow + all three machine-readable reports; S01–S05 evidence; plan and sprint records. **Not inspected:** the 135 failing Core tests' JUnit detail (gzipped; classified by S05 as companion-binary/code-mode groups, unattributed), PF-30 browser lane (out of scope), and live runtime behavior (execution prohibited).

## Verdict
**qualified-with-gaps, leaning not-ready for PF-13 completion as claimed.** The engineered boundary is high quality and I found no reachable secret-leak defect; but (a) the boundary is not wired into any production path, so sprint-record language currently overstates shipped behavior; (b) no qualification evidence exists at the reviewed merge commit; and (c) the full Core gate is failed and untriaged. PF-13 should remain `in_progress` until the P2 items are closed; the P3 items can ride follow-up sprints.

```json
{
  "reviewed_commit": "044491b8b02b24a65a84e8da61619d3444e63fe0",
  "coverage": [
    "All 35 PF13_REVIEW_FILES.sha256 paths verified by hash and read",
    "core/src/security/credential_capability.rs: issue/consume/purge lifecycle, entropy, capacity, atomic consumption",
    "vault/src/capability.rs + lib.rs: with_scoped_credential, zeroizing lifetime, panic containment, reveal_for_programmatic_use gate, normalize_label, keyring account derivation (secrets/src/lib.rs:193)",
    "security-policy: credential.rs, authorization.rs, mandate.rs, revocation.rs, action_context.rs contracts",
    "core/src/config/network_proxy_credential.rs: VaultNetworkCredentialResolver resolve/receipt path; network_proxy_spec.rs state build",
    "network-proxy: credential_broker.rs, resolver.rs, providers (openai/github), mitm.rs forward_request/evaluate_mitm_policy/path_and_query, http_proxy.rs plaintext + CONNECT + DetectTls paths, upstream.rs (no redirect follower), runtime.rs install/virtualize/inject, config.rs defaults, features/src/feature_configs.rs TOML surface",
    "cli/src/main.rs:2453-2472 run_vault_auth_helper; cli/tests/vault.rs",
    "scripts/security_credential_canary.py + test + workflow; qa PF-13-S01..S05 evidence and all three canary reports",
    "docs/plans/active/p0-security-levels.md; docs/sprints archive/current PF-13 records; review-diffs (six original patches)"
  ],
  "findings": [
    {
      "priority": "P2",
      "title": "Scoped credential boundary has no production call site; evidence language overstates shipped behavior",
      "file": "codex-rs/core/src/config/network_proxy_credential.rs",
      "line": 175,
      "preconditions": "Any real Moderate/Aggressive session at the reviewed commit",
      "failure_path": "CredentialCapabilityStore::issue/consume and build_state_with_scoped_openai_credential are invoked only from #[cfg(test)] code; credential_broker defaults false (network-proxy/src/config.rs:162) and is not exposed in NetworkProxyConfigToml; children keep raw OPENAI_API_KEY",
      "recommendation": "Wire the issuer path under a guard before claiming behavioral completion, or explicitly qualify all S01-S05 records as harness-invoked seam evidence pending PF-23 integration",
      "regression_test": "End-to-end test from real session spawn proving dummy-only child env and exactly one live CONNECT/MITM injection",
      "confidence": "high"
    },
    {
      "priority": "P2",
      "title": "No canary or Core evidence at the reviewed merge; full Core suite failed (135) and unattributed",
      "file": "qa/security-levels/sprints/PF-13-S05/evidence.md",
      "line": 1,
      "preconditions": "Qualification decision on commit 044491b8",
      "failure_path": "Windows report is at ea7d4bec72, macOS/Linux at 55025dd42/27b738ab8; post-merge evidence is only 6 Python tests and doc checks; 135 Core failures (123 in codex-core::all) remain unwaived and unattributed",
      "recommendation": "Re-run six-probe canary and targeted Core credential tests at 044491b8 on all three OSes; triage/fix the 135 Core failures before sign-off",
      "regression_test": "CI gate binding canary report source_commit to the merged hash plus required Core suite result",
      "confidence": "high"
    },
    {
      "priority": "P3",
      "title": "auth-helper posture gate resolves config via env-controlled CODEX_HOME",
      "file": "codex-rs/cli/src/main.rs",
      "line": 2453,
      "preconditions": "Agent can spawn processes with controlled environment",
      "failure_path": "Persisted and override configs both honor CODEX_HOME; max() gate is bypassed if the agent supplies a home with permissive config; containment currently relies solely on keyring account being keyed by canonicalized home path (secrets/src/lib.rs:193), which is untested",
      "recommendation": "Resolve posture from the canonical real home independent of env overrides, or bind and document the keyring-keying invariant",
      "regression_test": "CLI test: alternate CODEX_HOME with permissive config and copied/symlinked vault must fail closed",
      "confidence": "medium"
    },
    {
      "priority": "P3",
      "title": "Secret passes through non-zeroized String temporaries during header construction",
      "file": "codex-rs/network-proxy/src/credential_broker/providers/openai.rs",
      "line": 51,
      "preconditions": "Any scoped or legacy broker injection",
      "failure_path": "format!(\"Bearer {value}\") creates an unzeroed heap String containing the secret; freed without wiping; HeaderValue not marked sensitive",
      "recommendation": "Build header value inside Zeroizing<String> and mark HeaderValue sensitive",
      "regression_test": "Assert HeaderValue::is_sensitive() after injection",
      "confidence": "high"
    },
    {
      "priority": "P3",
      "title": "Dummy values leak real secret length and punctuation layout to the child",
      "file": "codex-rs/network-proxy/src/credential_broker/providers/providers.rs",
      "line": 96,
      "preconditions": "Broker virtualization of any supported credential",
      "failure_path": "shaped_dummy_value copies non-alphanumeric positions and length of the real secret into the dummy visible to the untrusted child",
      "recommendation": "Use fixed-format random dummies independent of real-value shape",
      "regression_test": "Matrix test asserting dummy shape independence from real-value shape",
      "confidence": "high"
    },
    {
      "priority": "P3",
      "title": "Stale doc comment on normalize_label (claims lowercasing; code only trims)",
      "file": "codex-rs/vault/src/lib.rs",
      "line": 545,
      "preconditions": "Reader relying on documented canonicalization",
      "failure_path": "Misdocumentation of a security-relevant label canonicalization function; behavior itself is consistent and fail-closed",
      "recommendation": "Correct the comment",
      "regression_test": "Doc check or unit test pinning case-sensitivity semantics",
      "confidence": "high"
    }
  ],
  "follow_ups": [
    "Live CONNECT -> MITM -> upstream end-to-end test with an installed scoped route (current canary captures a HeaderMap, not a socket)",
    "Independent assertion manifest so the canary cannot self-certify a weakened test tree",
    "Test the MITM-hook Authorization collision guard at forward_request level",
    "Define intended credential_broker config exposure before PF-23 wiring",
    "Zeroizing audit of header-construction temporaries; mark injected headers sensitive",
    "Require canary + Core-suite evidence at the exact merged hash for future sprint completion claims",
    "Deferred by design (not defects): AuthorityEpoch/policy-revision binding, non-OpenAI scoped providers, durable receipt persistence, cross-process revocation"
  ],
  "verdict": "qualified-with-gaps leaning not-ready: no reachable secret-leak defect found and the boundary design/tests are strong at the invoked seams, but the path has no production call site (claims must be re-scoped), no qualification evidence exists at the reviewed merge commit, and the full Core suite gate is failed (135) and unattributed. Keep PF-13-S05 open until both P2 items close; P3 items may follow up."
}
```
