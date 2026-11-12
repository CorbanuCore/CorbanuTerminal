# Corbanu DNS compatibility patch — issue #2127

Upstream: `rama-dns` 0.3.0-alpha.4, MIT OR Apache-2.0, by Glen De Cauwsemaecker and Rama contributors. Sources copied from the published crate; licenses copied from upstream commit `4733273a10a791762e2b7727850032b0c9b8536d` (path `rama-dns`). No attribution was removed. This is not an upstream release.

Purpose: preserve the existing Rama alpha interfaces while selecting Hickory 0.26.3, which contains the fixes for RUSTSEC-2026-0487, 0489, 0490 and 0491. The old resolver requires Hickory 0.25; no patched 0.25 release exists. A whole-Rama 0.3 stable upgrade was rejected because it changes TLS APIs beyond this bounded repair.

Intentional changes from the published crate:

- Cargo manifest: Hickory resolver minimum 0.26.3; keep Rama core/net/utils exactly at the existing alpha version. No resolver feature removal or audit exception.
- `src/hickory.rs`: adapt moved types, server configuration constructors and typed-answer extraction. Preserve absolute query names and A/AAAA/TXT results. Preserve existing system-config/default fallback behavior; do not add a new fallback.
- Hickory's builder is now fallible. Preserve the Rama builder signature by retaining any initialization error and returning it on lookup, instead of introducing a panic or silently selecting another server. The system constructor still returns initialization errors directly.
- Record extraction restricts results to the query class and original/terminal CNAME owner across answer, additional and authority sections. It ignores unrelated owners and aliases. When discarded CNAME intermediates leave no verifiable owner (including cached responses), bounded explicit alias lookup recovers the chain instead of trusting arbitrary answers. This can require an extra query with `preserve_intermediates=false`; resolver options and configured servers are not changed. Cycles fail closed.
- Corbanu integration tests in `codex-rs/network-proxy/src/hickory_compat_tests.rs`: isolated loopback A/AAAA/TXT/absolute-name coverage, mixed owners/classes, CNAME chains/additional records/recursive responses, cache replay, preserved/discarded intermediates, and an empty-resolver error control. No operator credential store or real external resolver is used. Tests run in the existing isolated network-proxy campaign. Vendor-only edits select network-proxy tests on Linux/macOS and the existing native Windows sandbox group.

Seven other upstream Rust files are copied unchanged. In `src/chain.rs`, three upstream fallback diagnostic strings have a spelling correction for repository checks; resolver control flow is unchanged. This compatibility patch should be removed after a separately qualified Rama upgrade supports patched Hickory without it. Cargo and Bazel must use the same local patched package. New source is security-code-owned. Full network-proxy qualification and exact independent review remain merge gates; this file is not a completion claim.
