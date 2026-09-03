# PF-57 Claude Fable 5.1 review record

Date: 2026-09-03 UTC

## Runtime

- Host: remote Linux test machine, user `travis`
- Worktree: `/home/travis/worktrees/CorbanuTerminal-pf57-final`
- Controller: Corbanu Terminal through TMUX session `pf57-fable-review`
- Model shown by the Corbanu footer: `Claude Fable 5.1 Plan max`
- Final reviewed commit: `a935e507b0173f4ee9c1f0aa539eea6e24ed200f`
- Full remediation base: `004556c527b69b42ab523a86822067ec86764edb`
- Immediate rereview base: `247d5bbbcb5278f7f9901de8fd7101e6a56f3491`

The reviewer was instructed to inspect and test without editing files. Its
generated TMUX artifacts were removed after the run and the remote tracked
worktree remained clean.

## First review findings and disposition

| Finding | Severity | Disposition |
| --- | --- | --- |
| Startup onboarding displaced the accepted PF-55 inactive/missing-current in-app recovery path | High | Fixed. Onboarding is now limited to an unconfigured current with no ready configured interactive alternative. Established failure states enter chat and remain blocked without replacement. |
| Onboarding recomputed full provider status, including vault/keyring access, multiple times per render | Medium | Fixed. The widget owns a cached `ProviderStatusCatalog`, initialized once and refreshed only after provider/account state changes. |
| Shared existing-provider selection could mutate completion state before resolving/persisting a model | Medium | Fixed. Model resolution uses configured model or the active widget model and occurs before `SelectExisting` dispatch; enrollment siblings use the same fallback. |

The reviewer also confirmed that keyring/vault changes remained secret-free,
command-auth validation remained lazy, and no credential values crossed the new
events.

## Final independent verification

- Lineage: both `81dcbef5d` (`origin/main`) and `06211dbfc` (PF-56 tip) are
  ancestors of the candidate.
- Remote build: PASS in 21.22s against the existing external target cache.
- Focused TUI run: PASS, 75/75.
- True-TMUX: PASS, PF-55 12/12 plus two PF-53 onboarding journeys in 147.84s.
- Remote cleanup: PASS, tracked worktree clean.

Final verbatim status token: `FINAL_REREVIEW: CLEAN`.

The reviewer reported: “No new defects introduced by this candidate were
found.”
