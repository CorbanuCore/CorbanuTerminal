# Scope and unresolved expectations (after proposal freeze)

The original 26 cases and all questions remain unchanged. This record does not
waive a case or claim product-authority acceptance of an exclusion.

Existing user intent supplies these answers: provider selection and credential
health must be distinguishable; the affected credential should offer `r` recovery;
Claude subscription and Anthropic API credentials are independent; no silent
billing/provider fallback is authorized. The user explicitly wants ordinary
startup without repeated completed native consent. No native consent is bypassed.

The supported-provider inventory, exhaustive model/effort combinations, minimum
terminal size, supported migration span, and timeout/healthy-status semantics
need a bounded qualification contract. We have not invented those answers.
In particular, a stored key labeled “configured” has not proved authentication:
the new invalid-key probe demonstrates that distinction, not a live server
accepting an invalid key. F08 is unresolved against the designer's stronger
expectation; deciding whether to validate on save or explicitly label unverified
credentials is a product/UX decision, not silently assumed by this QA pass.

Synthetic endpoint success is useful regression evidence but does not certify
live OpenAI/Claude/Anthropic onboarding, browser handoff, every supported
provider, billing route, or macOS consent. The original cases combine several
conditions; partial assertions do not make the whole case pass. Platform records
retain all original IDs and enumerate missing variants in each summary. F20 is
macOS-only as written; the Linux record carries it as a blocked cross-platform
handoff prerequisite rather than fabricating an approved scope exclusion.

This packet covers PF-58 provider/auth/model UX. It does not newly qualify every
security/memory feature in the wider humanTest.html; their existing prerequisites
remain visible. The case numbers PF58-Fxx are not the human checklist's 26 IDs.
