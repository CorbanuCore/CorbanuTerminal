# Corbanu multiple-account recovery — September 8, 2026

The installed 0.1.40 selected `secondfmaster` correctly even without a profile
config file. Its saved, locally nonexpiring token was rejected by production.
The server records revocation at 01:22:41 UTC. The TUI resolver returned that old
token before examining the pending relink. Separate legacy migration also
copied a default bearer into a matching named profile, allowing revocation in
one scope to invalidate another. Neither account names nor profile labels are
hard-coded in the repair.

## Delivered

- Shared, profile-scoped recovery checks a pending relink first, validates the
  replacement account, then promotes it. CLI status uses the same resolver.
- Per-profile file locks serialize status-triggered one-time exchanges.
- Named profiles link independently and do not import a default bearer.
- Status includes the local profile; CLI linking guidance retains `--profile`.
- Legacy authentication response codes map to current Corbanu instructions.
- Task Node's live error and completion page use Corbanu; terminal OAuth opens
  GitHub's supported account picker (`prompt=select_account`).

GitHub reference: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps

## Evidence

Corbanu source: `f20a2a7389e2baa5eaddcd00755ca93129591808`.
Task Node source: `c6c352e` (local commit; unrelated workspace edits preserved).
Production app machine: `8d4930ae156638`, image
`sha256:4595eabe67b038a679e4bcca117d11969841e38d30c9de24ed3035be4452a32f`.
The image extends the prior production image with two reviewed server files;
existing typed bearer parsing in the touched local route is included. Other
process groups and frontend assets retain the prior deployment.

48 affected Rust tests passed: 47 in the main run and the new snapshot after
review/acceptance in a focused replay. Scoped Clippy/fix and formatting passed;
existing unrelated TUI warnings remain. Backend account-selection/branding
smoke, lint and public Help manifest checks passed.

Actual PTY keys exercised revoked-token error, pending GitHub, completed relink,
two separate accounts, cold restart, cancellation and simultaneous CLI status
checks. The same matrix passed again against the installed stripped executable.
Authentication used loopback fixtures; zero model calls, no real GitHub login
claimed. Live production checks verified the rejected-session error, completion
page and OAuth account-picker redirect. Real `goodalexander` status remains
valid. Real `secondfmaster` remains revoked and requires its owner's GitHub
sign-in; no revoked session was reinstated and no records were transferred.

Private artifacts live in
`/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-multi-account-20260908`:
`installation.json`, `server-publication.json`, `server-live-checks.json`,
`pty-results.json`, `installed-pty-run.log`, affected test logs and PTY captures.
The prior installed package remains available for rollback. No public Corbanu
release was published and neither repository was pushed in this task.
