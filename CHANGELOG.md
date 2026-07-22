# PFTerminal 0.1.20

## Added

- Added an in-product `/telegram` setup flow with masked BotFather token entry,
  encrypted vault storage, exact chat and sender authorization, connector
  lifecycle controls, and automatic startup recovery.
- Added Telegram support for text, images, bounded documents, approvals,
  selectable model pages, per-chat status, cancellation, compaction, git diff,
  skills, and queued follow-up turns.
- Added durable update reconciliation so accepted Telegram messages survive
  connector restarts without duplicating completed work.

## Fixed

- Made first-message setup self-progressing: PFTerminal now waits for `/start`
  while the authorization view is open and ignores stale discovery results.
- Prevented simultaneous PFTerminal processes from starting duplicate Telegram
  pollers or stopping an unrelated process after PID reuse.
- Rejects model switches that cannot authenticate instead of changing the UI
  to a provider that will immediately fail.
- Preserved chat authorization, model selection, and queued input across the
  app-server state boundaries that previously caused silent or repeated work.

## Qualification status

- The Telegram connector suite passes 118 tests covering authorization,
  polling, durable reconciliation, media limits, approvals, model selection,
  rendering, session recovery, and failure backoff.
- Focused TUI setup tests and snapshots pass, including automatic discovery,
  stale-result cancellation, polling failure recovery, and connector status.
- The packaged-build and real-TUI CI checks passed on the release branch before
  merge. Live qualification covered encrypted token storage, connector health,
  forced-crash recovery, and single-poller behavior across two TUI processes.

Previous release: 0.1.19.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
