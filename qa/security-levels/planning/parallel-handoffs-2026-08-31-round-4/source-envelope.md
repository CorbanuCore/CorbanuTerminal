# PF-30-S01 typed-source-envelope preparation handoff

PF-22-S02 is integrated and archived. This lane is allocated to
`/root/pf30_source_envelope` at
`/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-source-envelope` on
`feat/p0-security-source-envelope`, base `b457249aa29c912cb2d5404f0939e8b2386f4e5e`.
Stay inside the recorded sprint write scope; shared registrations remain with
the integration owner.

## Frozen preparation result

Extend the existing `codex-security-policy` provenance types rather than creating
a competing model. Attach host-owned source/authority/lineage envelopes at
trusted ingress and lower them only at provider adapters. External labels,
quoted human text, forged roles/tokens, Unicode wrappers, tool/MCP/hook output,
files and child messages cannot mint authority. Authenticated human control
continues through its separate typed channel.

Permissive must remain byte-compatible. Repeated serialization must be
deterministic so the envelope does not cause cache-prefix churn. Unknown or new
provider/tool/ingress variants reject or become conservative untrusted content
before model admission.

The integration owner allocated a fresh CorbanuDrive worktree and audited the
declared scope against PF-27. The expected worker-owned
surfaces are the existing security-policy provenance module/tests, protocol
provenance re-export/serialization, already-registered Core ingress leaves,
focused tool/MCP/hook/child/file/provider adapter tests, and
`qa/security-levels/sprints/PF-30-S01-typed-source-envelope/`. Shared module
registries, manifests/locks, active plan/index/MkDocs and archive transitions
remain integration-owner-only.

Do not write into `qa/security-levels/sprints/PF-30-S01/`; it already contains
superseded browser-isolation evidence and must remain immutable.

## Future proof

Plan named `pf_30_s01` tests for malformed/unknown envelopes, forged role and
human markers, Unicode/complete-before-clipped markers, synthetic unregistered
routes, tool/MCP/hook/child/file coverage, Responses/Chat/Anthropic request-body
capture and deterministic repeat serialization. Run full affected suites,
PF-21 combined compatibility, a supporting TMUX smoke and TMUX/Corbanu/Claude
Opus 5 Max review after allocation.

Final macOS/Linux/Windows provider evidence waits for the user's tailnet switch.
