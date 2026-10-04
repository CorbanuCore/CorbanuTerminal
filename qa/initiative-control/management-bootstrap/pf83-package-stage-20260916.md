# The PF-83 harness now stages the whole product, and a case runs

**Fable, 2026-09-16.** The receipt before this one established that the sealed
guest could run functional work, and named the cause of every earlier failure as
one line in our own build. This closes that: the harness itself now produces and
stages a package that works, proven by executing against it.

## What changed

Two hardcoded single-binary assumptions, both in
`.codex-work/functional-pf83.20260915/`:

- `build_native.py` built `cargo build --release -p codex-cli --bin corbanu` and
  packaged that one file as `format: "cargo-cli-only"`. A successor builds
  `corbanu`, `codex-code-mode-host`, `corbanu-acp` and `corbanu-walletd` from the
  same pinned source — commit `e3bd579bf`, tree `dd4ea6fc1`, clean, asserted
  before and after the build — and takes its output directory as a parameter
  instead of editing a finished increment in place.
- `macos_adapter.py:stage()` staged a fixed two-member list and set the mode by
  comparing the archive name to the literal string `package/corbanu`. Members and
  modes now come from the manifest inventory that `verify_package` has already
  checked, so the archive always matches the package rather than a list that has
  to be kept in step by hand.

## What review found, and what I did about it

Independent Opus 5.0 High review of the staging boundary: no source disclosure,
no escape, no host path can reach the guest. `verify_package` builds its
inventory from `rglob` and `relative_to`, so manifest keys are exactly on-disk
relative names — a `..`, an absolute prefix or a host path cannot survive the
equality check and become an archive member. Both sides mask modes with `&0o777`,
so a setuid or setgid entry fails the comparison and aborts.

Two P2s, both closed before this was accepted:

- The attestation path, and the package and manifest paths it names, were
  unconstrained. Members are now derived from the manifest, so an arbitrary
  attestation would have chosen the entire archive. All three are confined to the
  harness, exactly as the output directory already was.
- `corbanu-debug` was a 322 MB byte copy of `corbanu`, in no binary list and in
  no allocation, justified by a comment citing a "pinned Darwin packaging policy"
  that does not exist anywhere. It is removed. The real release archive does ship
  that name, as a symlink, and if it is ever wanted here it should be a symlink
  and should say so — but inventing a policy to explain an artifact is the part
  that mattered.

Three P3s recorded and not fixed: `verify_package` validates a symlink's
*resolved* target on the host while the archive carries the literal linkname,
which a different guest root depth could exploit if the package ever contained
one (it does not); there is no mode allowlist, so world-writable would pass; and
the build seeds its cargo target directory from a previous increment, which
trades a guaranteed-from-scratch rebuild for speed.

## Proof

Rebuilt and restaged by the corrected scripts, then run in the sealed guest:

| | |
| --- | --- |
| Package | `codex-code-mode-host`, `corbanu`, `corbanu-acp`, `corbanu-walletd`, all `0555` |
| Pin | commit `e3bd579bf`, tree `dd4ea6fc1` |
| Remote root | `/Users/agent/pf83-preflight-4f53adade244434c91e983aa580e3f6b` |
| Case | sum the numbers in `seed.txt` into `result.txt` |
| Result | `result.txt` contained `40`, exit 0 |
| Denials after the case | `github.com`, `raw.githubusercontent.com`, `codeload.github.com` all `000`/denied |

## What is still true and unfixed

`macos_boundary.serve` asserts `op == "preflight"` and refuses cases, so the
preflight's own 380-verdict suite still cannot be driven through the service.
The worker hit that and stopped rather than building past it, which is correct.
Under the owner's standard that machinery is not the gate: the executor can run,
can do real work, and cannot read the product source, and all three are now
demonstrated against the harness's own artifact rather than a hand-assembled one.
Wiring cases into the service remains available to a later sprint that actually
needs OS-level denial evidence, and it would be re-opened deliberately.
