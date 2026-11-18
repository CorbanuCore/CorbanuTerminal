## Providers and models

- Provider rate-limit cooldowns now block locally for at most 60 minutes, including saved cooldowns from older versions. A request after expiry probes the provider: success clears the cooldown and backoff; another 429 starts a new bounded wait. TUI and exec errors show the remaining local wait and any longer provider reset hint. Use `/model clear-cooldown` or `corbanu -m MODEL -c 'model_provider="PROVIDER"' model clear-cooldown` to clear only the selected provider/model/credential's cooldown. CLI recovery honors `--account` for named credentials (#2567, D698).
