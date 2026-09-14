# YuE2 Real Audio — additive Facilities registration

User-authorized operator registration/deployment, received September 14, 2026
from the installation owner. No Corbanu model behavior, training runtime,
service installation or management-policy change. Existing registry/control
semantics are reused with one explicitly authorized unit, not broadened to
arbitrary commands or hosts. This is an additive operator configuration change,
not a new Corbanu product sprint or release qualification.

Product reference: **Internal delivery control — TO BUILD**, requirement:
“Facilities is the private static operator link index for the existing media
interfaces on RTX PRO 6000 and Drone, with upstream repository links.”

Source worktree: `worktrees/facilities-yue2-realaudio-20260914`, branch
`fix/facilities-yue2-realaudio-20260914`, base
`76efe32ebc336b44ada37bb38b31283e382f2181`. Receive into
`integrate/management-workstreams-20260911` using its single-writer Integrator;
one Luna Extra High publisher, preserving the paused timer.

## Exact registration

- ID/name: `yue2-realaudio` / YuE2 Real Audio, RTX PRO 6000.
- Interface: `http://100.99.88.49:7867/`.
- SSH target/user unit: `travis@100.99.88.49`, `yue2-realaudio.service` only.
- Upstream: <https://huggingface.co/Mothersuperior/yue2-mothersuperior-realaudio-tokenizer-v4>.
- Stock `yue2` remains at `:7861`, unit `yue2-ui.service`; every existing entry
  remains unchanged. Music Studio remains link-only.
- Eight interface cards; seven managed interfaces.

The installation owner's private receipt is
`/Volumes/CorbanuDrive/Corbanu/yue2-realaudio/deploymentReceipt.md`, tested source
`a057507d45ffc106d0077e404ebbdeb34253132b`. It reports a short GPU round-trip,
browser upload/download/cancel/recovery, eight unit/HTTP checks and exact-unit
stop/start verification. Those are installation-owner results, not independent
dashboard execution. No artist training, full-length or perceptual-quality claim.
NC/authorized-audio and short-clip smoke-test limits are visible in the card.

## Frozen code-blind cases

Fresh designer `01a0a242-8743-7573-8aac-6b0b187905aa` received intent only,
not code, prior tests or results. Original prioritized cases:

1. P0 Start/Stop isolation: new card controls only travis's
   `yue2-realaudio.service`; stock `:7861` and all other services retain state
   and availability, verified through service-manager observations.
2. P0 Additive registry: fresh before/after page comparison adds exactly one
   card and one to the count; all existing links/controls remain functional.
3. P0 Private publication/paused timer: existing private access works; timer
   remains paused with no intervening activation; all other entries survive.
4. P1 Identity/destinations: exact title, machine, UI URL and upstream; deployed
   interface reachable through the card.
5. P1 Scope: visible NC, authorized-audio and short-audio tokenization/reconstruction
   limits; no artist-training implication.

## Evidence boundaries

Final candidate checks: `test_control.py` 37 passed;
`test_facility_control.py` 8 passed; sprint checker 116 current/126 archived;
`git diff --check` passed. Structured autoreview (`--mode local --engine codex
--model gpt-6-astra --thinking high --no-web-search`) returned clean, no findings
(confidence 0.94). No review-triggered source changes. Private review and
receiving/publication receipts are under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/facilities-yue2.Sp8AUM/`.

Focused automated checks cover rendered identities/counts/links, unchanged
link-only authority, real HTTP GET/HEAD and unsafe host/path rejection,
atomic publish/last-good preservation and mocked exact-unit command isolation.
Publication must return commit/generation/manifest and timer-state receipts.
Do not interpret this as a full independent binary-only, permission-isolated
functional pass: real dashboard-button lifecycle execution is not performed by
this patch's automated tests. Live service stop/start is intentionally not
repeated merely to publish an entry. Original installation evidence is retained.
No TUI changes; Rust, TensorCash/Isometric and benchmarks are not affected.
