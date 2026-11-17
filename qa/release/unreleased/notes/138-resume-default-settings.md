## Resuming conversations

An interactive resume can remember the resumed conversation's model, provider,
reasoning and service tier as defaults. An absent service tier can be saved
explicitly as `service_tier = "default"`. Explicit invocation overrides remain
scoped to that invocation, and an unknown remote provider is not saved as a
local default (#138).
