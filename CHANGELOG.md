# PFTerminal 0.1.12

## Added

- Added `/gpu`, a durable GPU-rental control surface for Vast.ai and RunPod with explicit hourly,
  total-spend, and duration authorization before any billable resource is created.
- Added curated DeepSeek V4 Flash recipes and experimental Huihui DeepSeek/GLM fine-tune recipes,
  including a 300,000-token Huihui GLM 5.2 configuration for 2×H200 systems.
- Added authenticated rental endpoints, crash-safe reconciliation, provider-confirmed teardown,
  persistent spend state, and live rented-model availability in the model picker.

## Fixed

- Replaced unstable public quick tunnels with private SSH forwarding for Vast rentals.
- Refreshed recovered rental endpoints in both the durable provider catalog and already-open
  PFTerminal sessions, preventing resumed turns from reconnecting to stale local ports.
- Retried transient vault reads without duplicating provider creates, handled expired funding and
  capacity races without dead retry loops, and made rental-limit prompts identify their units.
- Added a per-device CUDA runtime gate so an unusable assigned GPU is rejected before a large model
  download, even when `nvidia-smi` still reports the device as present.
- Normalized child mail before Claude turns to avoid unsupported assistant-prefill requests.

## Qualification status

- GPU-market automated suite: 71/71 passed; state suite: 173/173 passed.
- Live Vast qualification covered capacity races, concurrent PFTerminal processes, authenticated
  readiness, tool use, endpoint recovery, spending-cap/TTL cleanup, and provider-confirmed manual
  termination across multiple 2×H200 rentals.
- A preserved real TUI conversation completed at 167,471 retained server tokens with zero
  truncation, exact distant-sentinel recall, and one successful shell-tool round trip.
- The Huihui GLM 5.2 path remains labeled experimental despite this passing qualification. The
  Huihui DeepSeek fine-tune recipe did not finish its model download within the authorized cap and
  is not claimed as live-qualified.
- The full `codex-core` suite was not green in this workspace; the exact partial result and known
  unrelated TUI failures are recorded in `GPU-RENTAL-QUALIFICATION-20260716.md`.

Previous release: 0.1.11.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
