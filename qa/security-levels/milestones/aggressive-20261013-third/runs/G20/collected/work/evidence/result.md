# Group G20 functional run report

This is an automated executor observation report, not human acceptance.

## Preflight and identity

- Time started: 2026-10-13T21:13:39Z approximate run start.
- Candidate binary SHA-256: `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`; expected value matched.
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; expected value matched.
- Amendments/new cases SHA-256: `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9`; expected value matched.
- Coordinator preflight summary `preflight-summary.json`: `ok: true`, 85 probes, `failed: []`.
- Independent executor `agms-probe all` command was run first, returned rc `0`, and its output records `ok: true`; saved as `preflight-executor.json`.
- Launcher and tmux child preflight files are present beside the summary. Exact positive observed values included `"corbanu 0.1.48"`, private evidence directory writable, and all negative access/network probes marked `ok: true`.

## Setup observed before AGG-31

- Time: 2026-10-13T21:17:25Z.
- Fresh product profile was reset with only `product-home/config.toml`. The config enables security levels and starts Permissive; it does not set an active security level.
- Fixture project exists at `$AGMS_RUN/fixture/workspace` with the supplied README, source file, notes, `.gitignore`, synthetic `.env`, and outside notes file. Git status was clean before launch. File contents were not used or sent as test input.
- Product launched via `agms-tui start` through the required launcher with default route/model. UTC sequence: trust prompt seen at 21:16:30Z, Enter pressed, main UI visible by 21:16:32Z.
- On the initial main screen, no security-level label was displayed. In the permissions UI, option 1 was visibly marked `Ask for approval (current)`, which is the visible current-level indicator used for this case. The screen did not use the word “Permissive”.

### State and fixture notes

- Product profile state after the case: `product-home` contained `config.toml` plus the product’s runtime/profile files generated during launch (state, logs, memories, goals, installation ID, source-origin key, locks, `.sandbox_migration`, and a secrets vault lock). No configuration file was manually edited during the case.
- Post-case tracked fixture hashes exactly matched the pre-case hashes listed above. Git status remained clean at 21:17:15Z. No test fixture files were changed.

## AGG-31

### Requested actions

1. `/permissions`, choose Full Access.
2. At `Enable full access?`, cancel.
3. Reopen `/permissions`.

Expected: confirmation appears; cancelling shows nothing changed; level unchanged.

### Actual actions

All prompt text and Enter were sent as separate actions. The approval/confirmation prompt was never typed into.

- 21:16:35Z — typed `/permissions` (12 characters), then pressed Enter at the same UTC second. `PERMS-FIRST` screen showed `Update Model Permissions` with options 1–3; option 1 was visibly marked `(current)`.
- 21:16:48Z — pressed `3` to choose the visible `Full Access` option. `PERMS-CHOSE-FULL` screen showed the `Enable full access?` confirmation with two options: `Yes, continue anyway` and `Cancel`. The `Cancel` option was visibly selected (`› Cancel`).
- 21:16:54Z — pressed Enter to confirm the highlighted `Cancel` option. `AFTER-CANCEL` screen showed verbatim: `Full access was not enabled; permissions are unchanged. To enable it, choose Full Access, then "Yes, continue anyway".` The permissions menu remained visible.
- 21:16:59Z — pressed Escape; `BEFORE-REOPEN` showed the main input with the unchanged-permissions notice retained.
- 21:17:03Z — typed `/permissions`, then pressed Enter. `PERMS-REOPENED` showed the same permissions menu with option 1 again marked `Ask for approval (current)`.
- 21:17:15Z — stopped the product session after fixture/state checks.

### Checkpoints

- 21:16:50Z, `PERMS-CHOSE-FULL`: confirmation appears with exact heading `Enable full access?`, warning text, and `Yes, continue anyway` / `Cancel` controls.
- 21:16:55Z, `AFTER-CANCEL`: cancellation produced a visible unchanged message and the `Cancel` control had been the highlighted option.
- 21:16:55Z and 21:17:04Z: permissions level remained visibly `Ask for approval (current)` in both the post-cancel and reopened menus.
- 21:17:15Z: fixture remained unchanged (git status empty; tracked-file hashes unchanged).

### Security level at transitions

- Before opening permissions: no security-level text visible on the main UI.
- First permissions menu: `Ask for approval (current)`.
- Confirmation prompt: no security-level label shown.
- After cancel / reopened permissions: `Ask for approval (current)`.

### Refusals or declines

None observed; no model refusal occurred because no model prompt was issued.

### Ambiguities

- The UI’s permissions option is labeled `Ask for approval`, not the frozen starting condition’s abstract name `Permissive`. This is a naming difference, not evidence of a level change; the same option remained marked current before and after cancellation.
- The product generated normal runtime/profile files under its product home after launch. The case did not require checking them, and they were not inspected beyond the file listing.

### Expected-result outcomes

1. Confirmation appears — **passed**: `PERMS-CHOSE-FULL` showed the full-access confirmation prompt.
2. Cancelling shows that nothing changed — **passed**: after confirming the visibly highlighted `Cancel`, `AFTER-CANCEL` showed “Full access was not enabled; permissions are unchanged.”
3. The level is unchanged — **passed**: `PERMS-REOPENED` again showed option 1 as `Ask for approval (current)`; no other current option was shown.

## Final verdicts

- AGG-31: passed.

VERDICT AGG-31: passed
