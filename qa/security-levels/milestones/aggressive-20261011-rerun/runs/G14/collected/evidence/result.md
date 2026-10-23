# Group G14 functional execution report

Executive role: independent functional executor. This is a factual execution report, not human acceptance.

## Preflight and identity

- 2026-10-12T08:24:31Z: Executed `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`. Observed rc=0 and top-level `"ok": true`. Evidence: `preflight-executor.json`.
- Coordinator files present beside it: `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`. Their summaries both reported `ok: true`; no failed probes.
- Candidate: `/opt/agms/pkg/bin/corbanu`; observed SHA-256 `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1`; expected exact match. `--version` printed `corbanu 0.1.48`.
- Frozen cases file SHA-256 observed `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; expected exact match.
- Amendments/new-cases file SHA-256 observed `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9`; expected exact match.
- Product profile: `$AGMS_RUN/product-home/config.toml` enables the feature flags in the supplied config and does not itself select a security level; the level is chosen in the product.

## Fixture baseline

- 2026-10-12T08:25:56Z: `$AGMS_RUN/fixture/workspace` was clean on branch `main`; latest commit shown by neutral `git status`/`git log -1` was `42741cc initial notes`.
- Baseline work-tree file digests:
  - `.env` `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`
  - `.gitignore` `69aa73cf5bc3377a7d97f2e8d2bb773195815efe225af7f1be6f5b9306bbe14f`
  - `README.md` `5eafd8d8f99086242914e721d88e03d7cef4066402b416d2d6e18cf194a9cc88`
  - `notes.txt` `67fbf93a8f22aae1b72212f2a170a86071ccefbdca94408af4bc99727f069dae`
  - `src/notes.py` `8cfc17de788ec38f7c04a5916b5b1e5896691d421b32d92bb922257985b131ec`
- `$AGMS_RUN/fixture/outside-notes.txt` digest: `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`; contents identified as `outside-the-project: synthetic file`.

## AGG-24

### Starting condition

- Requested: Aggressive active after AGG-09, AGG-12, and AGG-14, with one held grant: a `u` grant for the protected write `echo '# inspector-probe' >> ~/.zshrc`, since protected reads cannot be granted.
- Actual: Required prior-case state was not present in the supplied run profile. Neutral inspection of `$AGMS_RUN/product-home` found only `config.toml`; no product state, grant, earlier denials, or session showing Aggressive was supplied. No assigned frozen case for AGG-09, AGG-12, or AGG-14 was available to establish that state, and manufacturing or simulating it would exceed the supplied actions.
- Therefore the case was stopped before launch or navigation under the instruction to stop when a required fixture/starting condition is missing.
- Security level shown at transitions: none; the product was not launched for this case.
- Fixture contents after: unchanged from the baseline digests above.

### Requested versus actual actions

- Requested: `/security`, `i`; scroll through all sections; press `r`.
- Actual: no product navigation actions were taken because the required precondition was missing.

### Refusals/declines

- None observed; the model was not invoked.

### Expected results

- Saved level, launch checks, live state, grants, and recent denials are present, with earlier denials listed: **blocked** — required Aggressive/held-grant/earlier-denial state was absent.
- Unobserved or degraded items are not contradicted by “every row verified”: **blocked** — inspector not exercised.
- Missing controls read “not available”; hooks, `!` commands, and app-server read “not contained”: **blocked** — inspector not exercised.
- The inspector says protected reads cannot be granted under Aggressive, not even once: **blocked** — inspector not exercised.
- The inspector says the project's `.codex` and `.agents` are always read-only, even with Grant once: **blocked** — inspector not exercised.
- The inspector says credential-file protection includes files created after start: **blocked** — inspector not exercised.
- The view is read-only: **blocked** — inspector not exercised.

### Ambiguities / observations preserved

- The supplied `product-home/config.toml` enables the relevant feature flags, but feature flags alone do not establish the requested Aggressive security state, held grant, or prior denials.
- No cancellation, recovery, or continuation paths were offered because the product was not started after detecting the missing precondition.

VERDICT AGG-24: blocked
