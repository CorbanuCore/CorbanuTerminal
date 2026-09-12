# PF13 compatibility and patch-release recommendation

Fetched origin on 2026-09-12 before comparing branches.

- PF13 remote branch: `feat/pf-13-s02-scoped-vault-resolver`, `c25e2825a2fe3fe63c8e1d58cc1e3aa82b0d1d04`.
- Published base: `rust-v0.1.42`, `5f3a0ad7d7`.
- PF13 is already an ancestor of the published base and this patch candidate.
- Candidate branch: `fix/provider-replacement-release-candidate`.
- Patch implementation: `8ba39d10f1`, copied from tested `94b042de5b`.
- Entire `codex-rs`, `.codex/skills`, and `.agents/skills` trees match the
  tested and host-installed hotfix source exactly. No PF13 code was removed.
- Existing evidence exercises the PF13-integrated application: 30 focused
  tests plus cancellation, rejected replacement, same-process recovery, and
  restart in both live-repository fixtures. No separate new PF13 binary run
  is claimed. The older PF13 branch has no new Providers manager to patch
  directly and was left unchanged.
- The user confirmed the production fix with “works” on 2026-09-12.

Recommendation: package this narrow patch as 0.1.43 from the published 0.1.42
base. Keep the broader `feat/provider-reauth-health` branch separate: it includes
substantial additional security, memory, and qualification work.

This branch is prepared locally. Version remains 0.1.42 pending an instruction
to publish. No tag, push, public release, or cross-platform packaging was run.
The existing incomplete benchmark qualification remains disclosed in the
0.1.42 release record; preparing this patch does not resolve it.
