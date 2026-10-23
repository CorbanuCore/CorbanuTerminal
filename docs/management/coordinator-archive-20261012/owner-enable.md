You are the operations worker who turns the Corbanu management loop fully on.

## Authority
Travis (the owner), 2026-10-02, answering "Should I turn the dashboard manager on (let it dispatch work by
itself)?": **yes**. Record that decision as `owner-manager-enable-20261002` (revision 1) wherever an activation
authority record needs a decision id and an authority text. This authorizes moving the live owner from
`fixture-only` scope to `tmux-workers` with `manager_enabled: true`. It does not authorize pushes to main,
Slack sends beyond the system's existing receipted paths, credential-store reads, or changes to product source.

## Current state (verified 2026-10-02 by the coordinator)
- launchd job `com.corbanu.initiative-owner` (user/501, Background) ticks every 30 s and has 42k+ ticks. It is
  armed at generation 1, scope `fixture-only`, decision `owner-recurrence-domain-20260917` rev 3.
- Live runtime: `~/Library/Application Support/corbanu-owner-live/` (runtime/, private/config.json,
  private/authority.json, private/transport.json, schedule/). `config.json` has `manager_enabled: false` and no
  `transport`, so every tick is fixture-only and dispatches nothing.
- `private/transport.json` (prepared 2026-09-17) is stale. Its `binary` points at an old Sept-10 candidate, and
  its `runs_dir` is `/private/tmp/owner-runs`, which no longer exists: /tmp is cleared. Never use /tmp for
  durable state.
- `arm_owner` requires `requested_mode == "off"` and that the meta config/package digests equal the live
  config. A config change therefore needs the documented disarm, restage or reinstall, then arm sequence.
- Coordinator: `/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/state/coordinator`. Its last
  activity is from September. `slack-receiver-02` is overdue and stall_reported.
- The dashboard is published by `.codex-work/initiative-control.oGQGyA/sync-source.sh`; it was last run on
  Sept 27. The shared checkout it reads,
  `/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911`, is at the integration tip but has
  4 uncommitted files in scripts/initiative_control/ from another session. Do not commit, revert or discard
  them; a backup is at `.codex-work/management-checkout-uncommitted-20261002.patch`.

## Required reading first
In /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911, read
qa/initiative-control/management-bootstrap/: owner-armed-20260917.md, bridge-tmux-transport-20260915.md,
owner-daemon.md, owner-tmux.md, owner-arming-65.md, owner-cutover-85.md, bootstrap-deliverables-audit-20260916.md,
plus any newer owner-*.md. Read the runtime's owner_daemon.py and owner_tmux.py. Follow the procedure those
records establish; do not invent a shortcut around a guard.

## Do
1. `--activation-status` while quiescent; record it.
2. Prepare a durable transport:
   - binary = a COPY of the current developer build
     (`/Volumes/CorbanuDrive/Corbanu/.codex-work/integration-build-20260922a/integration-package-dev/bin/corbanu`,
     integration commit 704650ccd8) at a stable path you create, e.g.
     `/Volumes/CorbanuDrive/Corbanu/.codex-work/owner-worker-bin/`, with its sha256.
   - runs_dir = a private 0700 directory outside /tmp.
   - auth_link and tmux unchanged unless invalid.
3. Disarm generation 1 (keep the receipt). Then perform the documented receipted restage or reinstall so meta
   matches the new config (transport + `manager_enabled: true`), preserving old records under archive names as
   before.
4. Write the new authority with scope `tmux-workers` and the next generation. Run `--arm --dry-run`, inspect the
   preview, then arm.
5. Verify from state, not intent: interval-attributed ticks, `hold: null`, routing `tmux-workers`, watchdog ok,
   and the manager state. If a hold appears, diagnose and recover it with evidence, or disarm and report.
6. Run `.codex-work/initiative-control.oGQGyA/sync-source.sh` to republish the dashboard. Check that
   http://127.0.0.1:8769/health.json shows a fresh `collected_at`. If the dirty shared checkout makes the sync
   refuse, report it; do not clean the checkout.
7. Write a record, `qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md`, in the shared
   checkout. Do not commit it; the coordinator will.

## Never
Never use unscoped `tmux kill-server`. Never read or print credentials. Never push. Never edit product Rust
source. If any step needs an owner action or is refused by a guard you cannot satisfy, stop at a safe state
(armed fixture-only, or disarmed) and report exactly what is needed.

## Report (your final message)
The state before and after, every command you ran that changed state, the new generation, digests, the binary
and runs_dir, tick evidence, the dashboard result, and anything left open.
