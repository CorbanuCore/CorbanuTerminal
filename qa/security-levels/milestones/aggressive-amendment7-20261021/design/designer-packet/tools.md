# Test machine and product facts for amendment 7

## The test machine (as in the second local-model re-run)

- The tester's environment is described in `tester-prompt-R04.md` (the prompt the AGG-20 tester of the second
  local-model re-run received; Environment section) and `navigation.md`. `amendments-L2.md` gives the local-model
  launch (`agms-tui start --route qwen`), the rules L-1..L-5 and the AGG-20 parent baseline.
- The product's home is `$AGMS_RUN/product-home`; its `config.toml` is the product's configuration, which the tester
  may edit from their own shell before a launch (the product reads it at start). Agent commands cannot write the
  product's home under Aggressive.
- `screens-v6/` holds screens of the current candidate on the local model, including `/spawn` (09, 11-13), the model's
  description of its agent tools (10), panes (14, 16) and a child's `/security` and inspector (18, 19). Screen 10 shows
  the model's spawn tool taking an `agent_type` parameter whose allowed values are the roles with their descriptions.
- In R02 the product's permission instructions to the model were read by the integrator from the product's session
  file after the run (`R02-permission-text.txt`); the tester cannot see them.

## Custom agent roles (from the product's published configuration schema)

- A person defines a role in `config.toml`:

  ```toml
  [agents.NAME]
  description = "Human-facing role documentation used in spawn tool guidance."
  config_file = "./agents/NAME.toml"   # relative to the config.toml that defines it
  nickname_candidates = ["A", "B"]     # optional
  ```

- `config_file` is a role-specific configuration layer: a TOML file with the same keys as `config.toml`, applied to a
  child agent started with that role (the spawn tool's `agent_type` = `NAME`). Keys that give a child more access
  than Aggressive include `sandbox_mode = "danger-full-access"`, `approval_policy = "never"` and
  `[sandbox_workspace_write]` with `network_access = true`. A role file may also set only non-access keys, for example
  `model_reasoning_effort = "low"`.
- Released behaviour notes for Aggressive (from the change records of the sprints that shipped it): "Custom agent
  roles that would change a child's settings are refused at start rather than overridden for the child", and
  "Aggressive now also refuses to start with custom roles that set `sandbox_workspace_write` or `profile`". Where and
  how the refusal is shown (at the product's start, at the restart into Aggressive, or when the child is started) has
  not been captured on screen in any run.
- With the local model, a request naming the role (for example "Start a sub-agent with agent_type NAME ...") is
  expected to reach the spawn tool with that `agent_type` (in R04 the model passed the `agent_type` it was asked for).
