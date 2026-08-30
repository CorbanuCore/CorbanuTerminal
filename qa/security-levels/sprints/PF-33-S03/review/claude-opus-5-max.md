# Claude Opus 5 Max Computer Use review

## Initial review

- Chat: `89ed3157-0d5a-4178-9cfb-2510ce01d212`.
- Model shown by the desktop UI: Opus 5 Max.
- Candidate: `7fb11e9c12f94ba6a90209317bcc61a2a4451364`.
- Inline-only review material SHA-256: `04b6e1c94c538b0f27fb4f364f0aba2283c5b3b241cc4cbd7f3a1b1e27e528b4`.
- Attestation: Claude stated it used only the inline untrusted material and did not execute code, run commands, browse, read files or consult private memory.
- Verdict: `CHANGES REQUIRED`.
- Immutable UI evidence: `claude-opus-5-max-initial-verdict.png` and `claude-opus-5-max-verdict.png`.

Disposition:

1. F1/F2 and the derivative part of F6 were not accepted. The sprint input explicitly requires absent, explicit-empty and wildcard-public states to remain distinct and states that Moderate public retrieval does not need a blanket host grant. `None` therefore deliberately means no *additional* public restriction; it is not the representation of an unknown/malformed protected policy. Documentation now requires configuration loaders to preserve that distinction and fail visibly on unknown configuration.
2. F3 was accepted in its actionable part. Single-label names and `.alt`, plus conservative collision/internal suffixes, now require an exact private-service identity. The claim that subdomains of `localhost` were missed was incorrect: the existing label-boundary suffix check already covered them. Regression coverage now includes both forms.
3. F4 was not accepted. The well-known NAT64 prefix is already denied. An arbitrary operator-specific network prefix is unknowable from an IPv6 address; rejecting every global IPv6 whose low 32 bits resemble private IPv4 has no translation semantics and creates false positives. Operator-specific translation prefixes remain an explicit PF-33-S01/S02 resolver/peer/alternate-egress input, consistent with this sprint's no-SSRF claim.
4. F5 was accepted as defense in depth. Private-service identity is evaluated once, and a literal private-service identity now has explicit answer/literal-pin coverage.
5. F6's chain-history point was accepted. The module now states that redirect decisions are per-hop and consumers must retain history and enforce hop limits.
6. F7 was accepted as documentation. The resolver answer limit is explicitly pre-deduplication so repeated wire answers cannot increase the effective bound.
7. The fixture and runtime-proof notes were accepted. All six fixture cases now execute against the contract; evidence calls the literal source scan a smoke test and distinguishes it from the direct scope audit.

After these changes, the required sequence passed with 239/239 tests and zero skipped.

## Follow-up review

- Corrected candidate: `e965c522f2eff367586aa03c70426c2cd0a26282`.
- Corrected inline-only review material SHA-256: `4d04f8973d76baf2daff14c6fe7bada4f78c8fa72dc61aa5e547f7c28d66a7e3`.
- Review remained in chat `89ed3157-0d5a-4178-9cfb-2510ce01d212` with Opus 5 Max shown by the desktop UI.
- Verdict: `PASS` with no remaining in-scope defects.
- Immutable UI evidence: `claude-opus-5-max-followup-verdict.png` (SHA-256 `575b10a1fe8c0e05d9776b44665285afc5d4af1a8be32aea3a90370e41807173`).

The follow-up explicitly withdrew F1, F2 and F4, and the incorrect `localhost` portion of F3. It verified the accepted F3/F5/F6/F7 fixes, executable fixture, corrected evidence wording and deferred obligations, then returned `PASS`.
