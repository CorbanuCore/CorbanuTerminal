## Linux sandbox

- Accept the AppArmor launcher for system bubblewrap only after its namespace probe exits successfully. A timeout, startup error, or unrelated failure remains inconclusive rather than being treated as confirmed namespace access. If a host cannot complete the probe, sandbox startup may fail; check bubblewrap and the AppArmor `rootlesskit` profile, then retry after resolving the failure or host load.
