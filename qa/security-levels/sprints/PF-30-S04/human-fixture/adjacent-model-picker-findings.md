# Adjacent model-picker findings — unpatched

The human fixture is routine test-only work. These existing product behaviors
are not changed, and this fixture must not be reported as `/model` UX parity.
Product runtime inspected/tested: `6a6bb029d`, immutable RTX codex0.1.38.

1. **Single-effort selection loses explicit custom provider identity.**
   `tui/src/chatwidget/model_popups.rs` resolves the selected preset's provider
   in `open_reasoning_popup_for_purpose`, but its single-effort Session arm
   delegates to `apply_model_and_effort(model, effort)`. That re-resolves the
   provider from the model/current catalog instead of preserving the selected
   preset's provider. The synthetic `fixture-model` journey selected another
   custom row yet displayed Memory Fixture A and sent both foreground requests
   to loopback A, not B. Evidence: RTX
   `/home/travis/security-round5/evidence/human-memory/rehearsal-sixth/ProviderSwitch/`.
2. **Custom-provider rows are ambiguous for shared model slugs.**
   `model_picker_item` marks current using the model string alone. Its label
   does not include the runtime provider. The same capture shows five identical
   `fixture-model (current)` rows. A numbered row is not an adequate human
   provider-identity instruction or proof.
3. **Fresh custom runtime presets are populated by `/providers`.**
   `open_provider_manager` calls `sync_runtime_models` for configured runtimes.
   Before that action, the fresh custom fixture's `/model` picker exposed only
   the current runtime preset. This is a source-supported limitation, not a
   claim that all configured providers are broken.

Additional observation, not fully diagnosed: in the tenth rehearsal a Bedrock
model remained selectable even with `provider:amazon-bedrock` present in the
fresh inactive eligibility file. Selecting it failed closed with incompatible
model/provider configuration and made no request. This fixture does not repair
or claim full inactive-provider parity. Evidence: `rehearsal-tenth/ProviderSwitch`.

Two unsuitable fixture approaches were discarded: overriding built-in provider
endpoints (only transport retry/timeout overrides are accepted), and inventing
a `gpu-` custom route (rental catalog refresh removes non-rental entries). No
real rental or external provider was used and no product safeguards disabled.

Coordinator-approved human journey instead uses `/providers` to deactivate
fixture A and explicitly replace it with fixture B, then `/model` to select
effort. The fresh disposable eligibility store makes unrelated built-in routes
inactive. Because existing model labels are not sufficient identity proof, the
final fixture also uses the existing `model_catalog_json` option to provide
distinct synthetic A/B slugs, labels and explicit provider metadata. It chooses
the uniquely named B model after replacement. This has no effect on personal
provider settings. Actual B routing and source-specific memory persistence
outcomes remain mandatory; final qualification is pending.
