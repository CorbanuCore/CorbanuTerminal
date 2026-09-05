# Ambient GLM-only model options

Date: 2026-09-04. Bounded catalog correction requested by the user.
Product citation: **Shipping MVP — LIVE**, “Multi-provider inference” includes
Ambient. The requested correction retains GLM 5.2 as Ambient's only offered
model; it does not change credential storage, provider routing, or other
providers' model options.

Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/reconcile-release-0.1.37`.
Branch: `integration/reconcile-release-0.1.37`; base `f38dccd8b`.

- Hide Ambient Kimi K2.7 in the bundled catalog and disable automatic selection.
- Keep it hidden when an older cache/discovery response advertises it.
- Remove its pane-creation option, retaining compatibility metadata for saved
  sessions and explicit legacy diagnostic routes.
- Update Ambient setup documentation. Corbanu Plan's separate backend catalog
  and other providers' Kimi models are outside this request.

- `just fmt` and `git diff --check` passed.
- Models-manager: 63/63 tests passed, including the stale-discovery overlay.
- Final focused UI tests: 14/14 passed with normal snapshot checking. The
  existing 80-column picker snapshot and new 140-column snapshot show GLM only.
- True TMUX: `tmux_ambient_model_picker_offers_only_glm` passed without retries
  in 21.368 seconds, nextest `1f786597-f08b-4882-965a-6dc4048674fb`. It opened,
  cancelled, and reopened Ambient's picker using actual keys, with no real
  credential and no inference request.
- Saved-pane fallback restoration includes retired profiles independently of
  new-pane menu eligibility; its regression test passes.
- Fable 5.1 Max via Corbanu/TMUX identified two snapshot gaps; both are corrected.
  Final review exited 0 with no findings (`ambient-final.txt`).
- Final native release build passed; stable Apps-launcher target reports
  `corbanu 0.1.38` and passes `codesign --verify --verbose`. SHA-256:
  `816d7490ad421d9cd643d23b3a7c1696032956e7b906ce2dfab6de75f4d1c77e`.
  The stable `bin/corbanu` link still resolves to `../target/release/corbanu`.
  Existing open windows retain their old executable; start a new window through
  `/Applications/Corbanu Terminal Launcher.app` for this catalog.

Operational logs: `/Volumes/CorbanuDrive/Corbanu/.codex-work/ambient-glm-*.log`;
review evidence: `.codex-work/provider-ux-review/ambient*.txt` outside the repo.
Successful TMUX captures are preserved outside the repository in
`.codex-work/ambient-glm-success-artifacts/pf54-ambient-glm-only/`.
Human acceptance remains pending; cross-platform and benchmark release gates
are unchanged by this bounded catalog correction.
No new release or merge is authorized by this change.
