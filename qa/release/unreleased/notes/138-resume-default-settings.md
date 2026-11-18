## Resume and recovery

An ordinary interactive resume automatically saves the resumed conversation's
model and provider as defaults in `config.toml`, along with its reasoning and
service tier. If the conversation has no reasoning effort, the saved reasoning
effort is cleared; an absent service tier is saved as `service_tier = "default"`.
Explicit invocation overrides remain scoped to that invocation. Profile or
explicit session service-tier overrides, non-embedded app servers and providers
absent from local configuration do not trigger this default update (#138).
