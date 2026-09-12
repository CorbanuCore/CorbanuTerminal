# Campaign Tracker recovery and provider-tab repair

Bounded fixes. Product-spec **Campaign Tracker — LOCAL PILOT CANDIDATE**: “preserve attributable user prompts and compact GLM 5.3 Flash summaries in a bounded database”; **Shipping MVP — LIVE**, **Multi-provider inference**: existing provider catalog and selection.

Reviewed `/home/pfrpc/repos/campaign-tracker-error-report-2026-09-10.md`. Its limits are correct: the original failing action is unknown. Confirmed defects are generic validation messages, lost recovery route/input and missing supported Corbanu credential aliases. No claim that an empty credential caused the original report.

Provider refresh copies the current model into every configured runtime provider using a permissive provider-default resolver. OpenAI falls through to the current Claude model, so the picker receives an explicit incorrect OpenAI entry. Reuse the existing known-provider compatibility check before including runtime models; preserve intentional custom-provider catalog entries.

Worktree: `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix`; branch `fix/0.1.41-wallet-daemon-launch`. Starts after local wallet fix `9677aef4e2` on released 0.1.41. This record and the code are committed together; the candidate remains unpublished. The binary SHA-256 is recorded in [candidate.json](tracker-and-provider-tabs-2026-09-10/candidate.json).

## Changes and validation

- Campaign Tracker resolves the three supported Corbanu environment credential aliases in their established order, then uses the saved credential fallback.
- Failed reads retain their exact route and filters. Failed campaign, mapping and review forms offer correction with the draft retained; cancellation does not resubmit a write. Idempotent recording changes can retry with freshly resolved credentials. Recovery state excludes the API credential.
- Error views show the operation and wrap the server's actionable message. The corresponding Task Node validation changes are deployed as release 736, completed 2026-09-10 at 17:27:39 UTC. Deployed-module checks confirmed field/byte-limit guidance and a distinct missing-credential response.
- Runtime catalog refresh rejects known incompatible provider/model pairs using the existing shared compatibility function. Tests cover several Claude models, Z.AI, repeated refresh and preservation of intentional custom providers.

`just fmt` passed before final validation. `just test -p codex-tui -E 'test(campaign_tracker) | test(model_catalog) | test(model_picker_runtime_refresh)'` passed all 25 selected tests. Three new snapshots were reviewed and accepted, including wrapped read/write errors and the OpenAI tab after a Claude refresh. `cargo build -p codex-cli --bin corbanu -j 16` passed. No runtime code changed after these checks.

## Actual PTY workflows

The final Linux debug binary was exercised through tmux with actual keys. Text entry and Enter were separate sends. Inputs and assertions are preserved in [the tracker script](tracker-and-provider-tabs-2026-09-10/tracker_pty.py) and [the model-picker script](tracker-and-provider-tabs-2026-09-10/model_tabs_pty.py).

1. Tracker: open `/tasknode` → Campaign Tracker → Enable recording. With no credential, confirm actionable Providers guidance. Restart with each supported alias and confirm recording can be enabled and paused.
2. Create a campaign with a 201-byte title and a retained objective; submit with Ctrl-D. Confirm the 200-byte rejection, choose Edit draft, inspect the retained objective and cancel with Escape. Assert no second write. Reopen correction, submit a valid draft and confirm exactly one corrected submission.
3. Open activity against a one-shot failing local service; select Retry and confirm the second request has exactly the same route.
4. Seed synthetic OpenAI and explicitly selected Claude environment credentials in a fresh isolated home. With Claude Fable current, open `/providers`, cancel, open `/model`, and navigate provider tabs using Left/Right. Confirm OpenAI contains its five GPT models and no Claude model; confirm Claude Plan retains Fable. Cancel with Escape.

Both PTY scripts passed. [Tracker results](tracker-and-provider-tabs-2026-09-10/tracker-result.json), [model results](tracker-and-provider-tabs-2026-09-10/model-tabs-result.json), screens and focused-test results are saved alongside this record. The temporary vault example used only to seed synthetic Claude selection metadata is preserved as `provider_qa.rs.txt`; it is not part of the production crate.

The tracker PTY uses a localhost HTTP fixture; model selection uses the bundled catalog and synthetic credentials. Neither exercise submits inference or production writes. These checks establish local interaction and recovery behavior, not live provider account eligibility. The original reported failing operation remains unidentified. Full release benchmarks, both default-repository release workflows, cross-platform execution and named-human acceptance are not supplied by this bounded-fix record.
