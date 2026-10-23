# Tart evaluation evidence (2026-10-16, #864)

Raw outputs behind [docs/qa/tartEvaluation.md](../../docs/qa/tartEvaluation.md). Tart 2.41.1;
TART_HOME `/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-home`; one Tart VM at a time while UTM
"macOS 3" ran qual-mod2 (never touched).

- `pull.txt`: image pull start, duration in seconds, end.
- `prepare-base.out`: the final base build (`corbanu-qa-base`).
- `cycles/`: one JSON row per `cycle.sh` run (memory and `clone_private_mb_*` in MiB, CPU in % of
  one core). `final-base.jsonl`
  holds the reported median. The others come from the two earlier base builds: `base1-*` (Gatekeeper
  still off) and `base2-ssh-fda-off`. `base1-a-trial` and `base1-b` come from an earlier `cycle.sh`
  that recorded free-space deltas (`disk_mb_after_*`, noisy) instead of the APFS clone cost. In
  `base1-a-trial`, row 1 is a single run; rows 2 and 3 are runs 1 and 2 of a 3-run attempt made while
  the host's Corbanu was being reinstalled: row 2's model turn failed (the vault lookup failed), and
  run 3 stopped at a binary hash mismatch and left no row. That led to the private binary copy and
  the retry in `cycle.sh`. Binaries: `cb42614c…` in `base1-a-trial`, `ecc044fb…` (the build
  installed at 10:53Z) in the rest.
- `base2-ssh-fda-off` and `fidelity-ssh-fda-off.txt` come from a base built by an earlier revision of
  `prepare-base.sh` that always removed SSH's Full Disk Access; the committed script does that only
  with `TART_QA_SSH_FDA=off` (the variable reaching the guest: `fda-env-check.txt`).
- All runs recorded here used the scripts as first committed (`c650e22b38`, plus the fidelity
  password display change). The round-1 review fixes came after them and have not run in these
  records: the process-count `guard_slot`, the bounded wait in `stop_vm`, the prep-VM deletion on a
  failed base build, `TART_QA_SSH_FDA` reaching the guest (checked on its own, above), and the
  "key file deleted" wording (`erase.txt` still says "erased"). `guard-check.txt`: the final
  `guard_slot` refusing a second Tart VM while one runs, with `TART` as the bundle path and as plain
  `tart` on PATH (UTM "macOS 3" up as well). Not run in any record: the moved prep-VM trap and the
  `utmctl` failure branch (log text only). Every recorded run had UTM "macOS 3" up; before round 4,
  `cycle.sh` stopped right after cloning on a host with no other guest running (fixed in `vm_pids`;
  the stand-in `pgrep` checks are at the end of `guard-check.txt`).
- `fidelity.txt`: `fidelity.sh` on the final base; `fidelity-final-base-boot1.txt`: an earlier boot of
  the same base. `fidelity-ssh-fda-off.txt`: the same checks on a base built without Full Disk Access
  for SSH. In the two older files the throwaway keychain password is replaced by `<throwaway>`.
- `sizes.txt`: download, cache, base, binary and suspend-state sizes.
- `utm-observation.txt`: UTM "macOS 3" disk, configuration, memory and CPU, read from the host while
  qual-mod2 ran it (nothing in the VM changed).
- `harness-used.txt`: SHA-256 of the harness files `case-run.sh` used; all ten match the committed
  `qa/security-levels/milestones/moderate-20261014/harness/`.
- `suspend.txt`: `tart suspend`/resume and clone-of-suspended timings (manual test, then deleted).
- `case-runs/GNN/`: per group, the host timeline, isolation gate summary, lane preflight, the
  executor's own probe (`preflight-executor.json`), its report (`result.md`), the guard count, the
  executor exit and the key check (`erase.txt`: the file was deleted with `rm -P`, which on APFS
  does not overwrite; the clone holding its blocks was deleted right after). Run with:
  `HARNESS=.codex-work/moderate-first-20261014/h PACKAGE=.codex-work/moderate-first-20261014/candidate2/package PACKET=.codex-work/moderate-first-20261014/stage/packet scripts/qa/tart/case-run.sh G05 G15 G10 G22`.
  Screens, PTY logs and product logs stayed outside the repository.
