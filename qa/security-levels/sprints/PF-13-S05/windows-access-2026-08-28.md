# PF-13 Windows access and publication chronology

Initial Mac recheck: 2026-08-28 at 08:19 UTC. Later entries distinguish the
remote agent's report from locally verified publication evidence.

Classification: qualification evidence for the existing product initiative,
PF-13-S05, still `in_progress`. Product authority: **Required trust boundaries**
— “Credentials are referenced by label and resolved only inside a trusted
execution boundary.” No implementation or authorization-boundary change.

## Initial Mac identity and result

- Travis supplied the corrected Windows endpoint `100.111.98.11` and authorized
  reuse of the previously provided credentials.
- Local evidence checkout: `a9ebfcc2ff965346c4507888addf5ce0fa8fc45b`, branch
  `feat/pf-13-s02-scoped-vault-resolver`, worktree
  `/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02`.
- Pending qualification candidate remains
  `f6ec1c75f6c389f68c5350df795a7f3d30e7fde4`, with the final source manifest in
  `repair-final-source-files.sha256`. Later commits change evidence only.
- Tailscale reports `postfiat1` online at the corrected address. TCP/SSH is
  reachable; server banner is `OpenSSH_for_Windows_9.5`.
- The observed ED25519 fingerprint is
  `SHA256:AL/bFpl1DX/lsk94alfnPXX20Fgx+PgIXyzsF9FATZ0`, exactly matching the
  fingerprint previously supplied by Travis. A dedicated temporary known-hosts
  file pins this verified key; strict host checking was retained.
- `postfiat@100.111.98.11` with available default Mac keys in batch mode was
  rejected. The local SSH agent has no loaded identities.
- One interactive password attempt using the previously supplied password was
  also rejected. Authentication output was
  `Permission denied (publickey,password,keyboard-interactive)`; both SSH
  commands exited 255. No repeated password attempts or alternative usernames
  were tried. No password is stored in this evidence.
- The earlier `/home/pfrpc/.ssh/id_ed25519` route belongs to `productionrpc`,
  not this Mac. `productionrpc` does not resolve here and is not present among
  the relevant Tailscale peers. No private key was copied, and no tailnet or
  SSH server configuration was changed.

## Initial gate status (historical)

Network reachability and host identity were verified. Authentication was
blocked, so there was no remote session, source transfer, prerequisite
installation, candidate build, canary, or Windows junction test. Existing
historical Windows results are not relabeled as repaired-candidate proof.

## Later remote-agent report

Travis relayed the Windows agent's report in this task:

- The Windows endpoint moved to `100.111.98.12`; the expected fingerprint
  matched and SSH worked from that agent's machine.
- Build Tools/SDK were present; Rust 1.95 and Python 3 still needed setup.
- The agent stopped before checkout: candidate `f6ec1c75f` was unavailable on
  GitHub or in a supplied bundle, and the branch still ended at `434635dd`.
- No qualification tests ran and no Windows qualification evidence was produced.

These are agent-reported host observations, not a new SSH/prerequisite check
from this Mac. They supersede the earlier login blocker for that remote route,
not the earlier evidence itself. No private key or password was copied here.

## Publication verified

Read-only `git ls-remote origin refs/heads/feat/pf-13-s02-scoped-vault-resolver`
confirmed the old GitHub tip `434635dd23b7a35944524cf9fa2b069312a94236`.
Required candidate `f6ec1c75f6c389f68c5350df795a7f3d30e7fde4` existed locally.
Travis authorized publishing the existing committed branch. The non-force push
advanced it to `a9ebfcc2ff965346c4507888addf5ce0fa8fc45b`; a subsequent
`git ls-remote` returned that exact tip, and the required candidate was verified
as its ancestor. Uncommitted changes and the separate PF-29 lane were excluded.

The subsequent commit of this record is documentation-only and does not change
the candidate to be qualified. This publication is not a product release.

## Historical pending gate

Candidate availability is resolved. The remote Windows agent should fetch the
published PF-13 branch, use a clean isolated checkout at
`f6ec1c75f6c389f68c5350df795a7f3d30e7fde4`, prepare Rust 1.95/Python and required
test tools, and run the final credential canary including directory-junction
posture coverage. Record the source commit, executable hash, commands and actual
results. Windows remains unqualified until that proof exists. The recorded 19
complete-Core failures and other release gates are not changed by publication.

## Final authenticated execution — 2026-08-29

The stored gitignored SSH profile authenticated from the Corbanu Mac to
`postfiat1` without exposing or copying private-key material. The host reported
Windows `10.0.26200.9168`, Python 3.13.15, and more than 800 GiB free on `D:`.
Rust/Cargo 1.95.0 and MSVC Build Tools were already present under `D:\rustqa`.

Published integrated repair commit
`be8153f2e29c360d83776441aed50deb204eafa7` was cloned from GitHub into fresh
checkout `D:\w13-be8153`; both `git rev-parse HEAD` and the empty porcelain
status were verified. Build, Cargo, Rustup and temporary paths remained on
`D:`. The initial canary invocation failed before probes because CMD's unquoted
environment assignment retained a trailing space in `RUSTUP_HOME`. Repeating
the exact build exposed that path error; no source change was involved. The
canonical `set "VAR=value"` syntax resolved it, and the candidate built with
Rust 1.95/MSVC.

The final canary passed all nine probe groups / 47 tests. Its four-test protected
raw-export group executed
`vault_auth_helper_symlink_home_cannot_downgrade_persisted_posture`; on Windows
that test creates an unprivileged directory junction using `mklink /J` and runs
both home-variable variants. Report
`repair-credential-canary-windows-integrated.json` is bound to clean source
`be8153f2e`, candidate version `corbanu 0.1.35`, and executable SHA-256
`37e0ac06e3f7cab75c684737c7d33e38453578dad5d5d8788ee8221ad8e23737`.
An independent remote `Get-FileHash` returned the same digest. Report SHA-256 is
`1fe4f74bf8ae55645012f189dbdcf665175c0b1cb960788f7ce32e456382d4aa`.

Windows component qualification is therefore passed. This does not certify the
future PF-23 native profile wiring or PF-26 integrated release candidate.
