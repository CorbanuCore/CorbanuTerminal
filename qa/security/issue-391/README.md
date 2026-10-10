# #391: `broker_model_auth` on by default under Aggressive

Travis decided on 2026-10-10, after #389 and #390 passed: Aggressive turns
`broker_model_auth` on by default first. Moderate and Permissive don't change;
they are a separate, later decision.

## Behaviour

| Level | macOS, Linux | Windows | Other systems |
| --- | --- | --- | --- |
| Permissive, Moderate | off | off | off |
| Aggressive | **on** (PF-27-S05) | **on** (PF-27-S09) | off: the broker isn't supported there |

- **Which levels count as Aggressive:** Core's level (`[security] level`, a
  project layer, or the confirmed `security_state.json`) and the level stored
  by `/security` (`security_level.toml`). An unreadable store counts as
  Aggressive, as it already does elsewhere.
- **An explicit setting still wins:** `[features] broker_model_auth` in your
  own config, a `-c` override, `--enable` or `--disable`, and managed
  requirements. A project's `.codex/config.toml` doesn't count, so a repository
  can't turn off the Aggressive default, even when your own config turns the
  broker on.
- **When a change applies:** a level from config layers (`[security] level`, a
  project layer, `-c`) applies on every load. A new session then starts the
  broker. The level `/security` saves on disk (its level file and Core's
  confirmed record) counts as this process first saw it for that home.
  Choosing Aggressive mid-session therefore takes effect at the next start, and
  a config rebuilt mid-session never makes a running session brokered without
  a broker.
- **Managed policy:** a requirement that pins the feature still wins. When it
  pins the broker on, the fail-closed message says so instead of suggesting a
  config change.
- **What the broker holds:** provider API keys from the environment or the
  vault. Sign-in tokens are still read by Core and then handed to the broker.
  Command, AWS and header sign-ins aren't brokered. Realtime and websockets are
  off (PF-27-S05 limits).
- **Fail closed:** if the broker can't start, every model request is refused and
  nothing is sent directly. When the level turned the broker on, the message
  names `broker_model_auth`, says to choose Permissive in `/security`, and gives
  the config line that turns the broker off. When config turned it on, the
  message gives only the config line. The same applies to a provider URL the
  broker can't bind. A broker that dies later still reports the existing
  "unavailable; restart" error.
- **`/security` view:** the Aggressive review adds a "Model keys" row, showing
  the current value next to the Aggressive value. The inspector shows a broker
  that didn't start as degraded. A broker that config turned off under
  Aggressive makes the level partial, and the launch path shows a startup
  warning.

## Tests

- `codex-core` `model_broker_auth::tests::sec_391_*`:
  - the default matrix: level x OS x explicit setting;
  - config loading: Core level, stored `/security` level, config off and
    `-c` off;
  - a project's config can't turn the default off;
  - the fail-closed message for each origin (level, config, policy) and cause;
  - a managed requirement wins both ways, and a pin that keeps the broker on
    is named;
  - a real broker that can't start on Unix and Windows: refused, and env keys
    scrubbed.
- `codex-core` test binary `sec_391_level_cache` (a normal build): a level
  saved mid-process waits for the restart, while a config-layer level applies
  at once.
- `codex-exec` `suite::aggressive_broker_default` (the real binary): under
  Aggressive the provider key is handed to the broker by default (or every
  request is refused if the broker can't start), and
  `-c features.broker_model_auth=false` wins.
- `codex-tui` `security::inspector::tests::sec_391_*` and
  `security::current::tests::model_keys_row_reports_the_broker_setting`, plus
  the updated `/security` snapshots.
- **Test isolation:** a config that turns the broker on marks the whole process
  brokered. Core's unit tests therefore don't mark it for the level default,
  and the in-process tests that load an Aggressive config for other reasons now
  set `broker_model_auth = false` (`pf_23_s03`, the TUI launch tests and the
  tmux memory canary).

## Live runs (GLM 5.3 Flash, disposable homes)

Evidence is added to the PR and to the #391 closing comment.
