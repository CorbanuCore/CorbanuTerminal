# Round-70 attempts

- Brief hash verified before reading; checkout HEAD matched the assigned base and
  initial status was clean.
- Initial `prepare_bundle.py` attempt exited 1 at the package directory mode
  check: `cp -cR` produced mode 0500 under the inherited umask, while the
  manifest requires directory mode 0555. No binary ran. Preserve the rejected
  copy at `attempt-1-artifacts/before-move`. The copy argv was corrected to
  `cp -cpR`; a fresh preparation passed. The failed attempt is not a pass.
- Fresh preparation verified four binaries before and after a real directory
  rename; attestation bytes/digest stayed unchanged. Local archive/extraction
  verification then passed. No path under the live harness or guest was written.
- All 288 selected packets were regenerated additively. Original and all
  non-candidate fields were checked unchanged. Historical harness total is 380.
- Eight isolated verifier rejection cases passed: changed attestation, manifest,
  binary, extra package file, package symlink, wrong mode, absolute path, parent
  path. Original package reverified after these disposable-copy checks.
- Prerequisite build passed, then guarded app-server lane passed, then guarded
  TUI lane passed. No retries; exact counts/log digests in checks.json. No native
  credential prompt or live-profile access was observed. One slow TUI test passed
  at 42.554s; the full TUI lane finished in 42.577s.
- Python syntax and all four shell command blocks in the new runbook passed
  parse-only checks. The guest-check script was parsed, never executed; guest
  identity and functional isolation are not claimed tested.
- No Rust edits or formatter/fix invocation, no packaged binary launch, no guest
  staging/contact, no case, subagent, commit or push. The independent functional
  execution/evidence gate remains open, with no invented approval or acceptance.
