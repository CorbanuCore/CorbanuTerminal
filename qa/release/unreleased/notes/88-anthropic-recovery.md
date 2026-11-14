## Providers

- Adapted @goodalexander's #88 recovery work: Anthropic signed-thinking rejection can be repaired once when it arrives after stream metadata but before any model output. Requests are not replayed after output begins, and plan-entitlement failures do not trigger credential refresh. Selected-account credential-file preservation is a separate follow-up, not included in this stage.
