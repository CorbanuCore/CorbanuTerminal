# Codex GPT-5.5 Autoreview

## Initial candidate

- Commit: `c4884bacca4997aa8cefb72b9e7242f44f07ad74`.
- Autoreview thread: `01a0517b-642c-7200-8311-107c7197f492`.
- Engine/model: Codex / `gpt-5.5`, high thinking, web search disabled.
- Verdict: patch incorrect, overall confidence 0.90; two P1 findings.

Disposition:

1. Accepted security finding: an exact private-service identity with an all-public DNS answer set could reach the absent public-policy branch instead of enforcing its pinned addresses. The candidate now selects an exact private-service identity before public-policy evaluation and rejects every unapproved all-public answer set as `PrivateAddressSetMismatch`; a regression assertion covers the rebind case.
2. The claimed move/compile failure in `match rule.host` was contradicted by the clean 238-test compile. The discriminant match was nevertheless changed to `match &rule.host` so ownership is explicit and the ambiguity is gone.

The required fix/format/test order passed after both changes: 238 passed, zero skipped (one unrelated existing test was reported leaky by nextest in that run).

## Final candidate

The first amended full-commit candidate `a15750eb40dd2c314614283a955e14acfa0c72fc` was reviewed in thread `01a0517d-a529-7231-9c6c-69199e412184`. It found one P2: a policy host containing `/..` could have its dot-segment path normalized away by `Url::parse` and become an unintended host grant. Accepted: policy-host validation now rejects path, query, fragment and userinfo delimiters before URL parsing, with regressions for `/..` and `user@host`.

The second amended full-commit candidate `90ffa6596c1d0c487f4b47c5a39930346db615a6` was reviewed in thread `01a05182-1fb4-7932-b366-6c3387eb95a5`. It found one P2: multiple trailing root dots were all trimmed, aliasing `example.com..` to `example.com`. Accepted: normalization now removes at most one root dot and then validates DNS name and label lengths/characters. Related empty/embedded port ambiguity is rejected before later consumers can reinterpret the authority.

The third amended candidate `7fb11e9c12f94ba6a90209317bcc61a2a4451364` was reviewed in thread `01a05186-8089-7da2-85ca-601c3acda186`. Verdict: patch correct, no findings, confidence 0.86.

The final full source candidate `e965c522f2eff367586aa03c70426c2cd0a26282` was reviewed in thread `01a05192-7d97-7600-84d5-37444a079559` after the Opus-driven hardening. Verdict: patch correct, no findings, confidence 0.86. The reviewer confirmed the contract remains isolated from runtime registration and fails closed across the covered private-address, private-name, mixed-answer, literal-mismatch and redirect-replay cases.
