# RETURN — owner-broker-114

Allocation digest: `d7363b13097763ecd678e0aaa3438b98b0b3432fc69b28b09cc21ba1fbae869c`.
Claim: `90c409bd-3ac7-4f2d-851d-fd439c041a15`. Runtime: gpt-6-astra / high.
Base verified: `8b0a83173e931525b36a024930559ec1db11142b`.
Brief read first after `shasum -a 256` matched
`92c5c4317478fc15d2c9b63d331b5acb11ec1350fdb00f6d4bcdafae0f2db879`.

Routine correction of the future qualification recipe and its evidence record.
Product context: **Internal delivery control — TO BUILD**, “durable event
dispatch, acknowledgments and watchdog”; “authorizes the bootstrap and bounded
live qualification, not premature product sprint resumption.” No runtime, gate,
authorization implementation or plan/sprint change. The manager supplied both
guest accounts as facts; this worker did not contact the VM, inspect live state,
run live preflight or perform qualification. No auth/token read or push.

## One recommended mediation route

**Use the administrative guest account to install the supported system config
`/etc/codex/config.toml`, pointing `openai_base_url` at the guest loopback end
of a restricted SSH reverse tunnel to an external Responses broker.** Keep
`cli_auth_credentials_store = "file"` in that same system file and supply only
a newly authored synthetic API-key marker at the transport's required auth path.
The real upstream credential remains broker-side. This uses the pinned transport
unchanged; sudo belongs to the separate administrative account, never the worker.
It needs no hosts, packet-filter redirect, local CA or provider substitution.

Source inspected directly at the base above, rather than relying on the review:

- [Loader](../../../codex-rs/config/src/loader/mod.rs), lines 54–79:
  Unix system path is `/etc/codex/config.toml`; the denylist is explicitly
  **project-local**. Lines 226–252 load/push the system layer; subsequent user
  and project layers merge on top. `system_config_toml_file_with_overrides`
  at line 663 uses the platform default unless explicitly overridden.
- [Merge](../../../codex-rs/config/src/merge.rs), lines 94–122: recursive table
  merge replaces only supplied keys; unrelated generated user settings do not
  erase the system endpoint.
- [Pinned adapter](../../../scripts/initiative_control/owner_tmux.py),
  `TmuxAdapter.prepare` at line 339: fresh HOME, auth symlink and user config
  with update/UI/analytics/trust settings only. `Worker.__init__` at line 374
  passes only PATH/TERM/LANG and four HOME/profile aliases. `launch` at line
  401 pins model, provider, effort and policy, with no URL, system-config
  suppression or user-supplied environment/config hook.
- [Core config](../../../codex-rs/core/src/config/mod.rs), lines 3945–3960:
  the merged nonempty `cfg.openai_base_url` is passed to
  `built_in_model_providers`. Explicit provider selection is retained below.
- [Provider](../../../codex-rs/model-provider-info/src/lib.rs),
  `built_in_model_providers` at line 1793 passes that URL to
  `create_openai_provider` at line 1124. That constructor retains the URL,
  uses Responses, and sets `supports_websockets = base_url.is_none()`, so this
  custom endpoint uses HTTP/SSE. `to_api_provider` at lines 1004–1034 uses the
  explicit URL before any public default. [Client transport selection](../../../codex-rs/core/src/client.rs),
  line 2109, rejects websocket use when the provider does not support it.
- [API URL construction](../../../codex-rs/codex-api/src/provider.rs), line 53,
  appends the endpoint to that base; [Responses](../../../codex-rs/codex-api/src/endpoint/responses.rs),
  line 117, uses `responses`. The resulting request is to
  `http://127.0.0.1:18443/v1/responses`.
- [Auth storage](../../../codex-rs/login/src/auth/storage.rs), lines 514–528:
  `File` selects file storage. This is not permission to reuse real auth.
- [Denied-project regression](../../../codex-rs/core/src/config/config_loader_tests.rs),
  `project_layer_ignores_unsupported_config_keys` at line 3311, confirms the
  old project-local URL/provider overrides are dropped with a startup warning.

This proves the inspected source has a working configuration path through the
unchanged launcher. The exact binary hash/build must be matched to this source
and demonstrate the route in the control run; no binary was launched here and
no artifact hash was supplied in this assignment. Source reasoning is not a
claim that a guest run or the packaged candidate already passed.

## Evaluated alternatives, proof and cost

Each row evaluates a route with the pinned adapter unchanged. All usable routes
still need the broker identity/response joins and external egress controls below.

| Route | Would the inspected pinned implementation honor it? | Additional proof required | Cost/tradeoff |
| --- | --- | --- | --- |
| **System config plus fixed reverse tunnel (recommended)** | Yes: the system-layer URL survives generated user config and CLI; explicit OpenAI selection uses it. The manager's administrative account can provision it. | Exact file hash/ownership, artifact build match, real adapter-launched SSE/tool round trip, joined upstream response IDs and broker-down failure with public egress denied. | One nonsecret root-owned file, existing SSH fixed forward, an approved broker and the existing guest fence. No adapter re-pin or CA lifecycle; consumes authorized inference budget. |
| Project `.codex/config.toml` with URL or nested provider override | No: denylist strips URL, providers, provider selection and profile keys even in a trusted worktree. | Startup warning/config inspection would prove rejection, not mediation; a TUI answer alone cannot rehabilitate it. | Cheap to write but unusable; removed from the recipe. |
| Environment set on owner/SSH launcher (`OPENAI_BASE_URL`, proxy or CA variables) | No supported pass-through: `Worker.env` reconstructs the entire environment. The OpenAI URL path inspected above is config-driven. | Actual launch environment and upstream joins would be required for any contrary claim; shell startup injection is not part of this pinned contract. | Requires an adapter/launch change to be reliable. No shell-wrapper, TMUX environment race or inherited-variable workaround recommended. |
| User-layer config that `prepare()` authors | The binary would honor a URL there, but `prepare()` never writes one and immediately returns into lifecycle dispatch. There is no approved config-input/pause hook. | A new reviewed adapter would have to bind the URL and prove its generated bytes and route; post-prepare file races are not qualification. | Pre-staging the ordinary account profile is ineffective because each worker gets a new HOME. Editing generated files mid-launch undermines the pinned lifecycle; a supported authoring change requires allocation/review/re-pin. |
| Administrator hosts entry plus local TLS listener | Conditional yes at the network layer: the API-key default URL remains `https://api.openai.com/v1`, so DNS/hosts routing still requires TLS for that hostname on 443. Hosts plus a plaintext listener alone cannot work. The HTTP client includes native-root TLS support. | Actual candidate TLS handshake against the guest's trusted test CA, hostname/SNI verification, HTTP broker joins, DNS/socket capture and proof of no alternative IPv4/IPv6/public route. The default provider may also attempt websockets: verify/provision that path. | Root hosts change, port-443 service, test CA/trust installation, certificate/key lifecycle, websocket compatibility and rollback. Greater provisioning and cleanup cost than an explicit system URL. |
| Administrator packet-filter redirect | Conditional network interception only. Packet redirection does not change HTTPS/SNI or authorize the dummy key upstream; a TLS-terminating broker with trusted hostname certificate is still necessary. A raw TCP relay to OpenAI would send an invalid dummy key and would not mediate credentials. | Actual installed rule/anchor and IPv4/IPv6 interface coverage, packet counters plus TLS/HTTP broker joins, new-connection controls and no direct bypass. No rule has been tested on this guest. | Root rules, OS/interface/routing-specific validation, TLS infrastructure and rollback; more fragile than system config. A PF rule alone is insufficient. |
| Explicit system HTTPS proxy plus trusted local CA | Conditional yes with the supported system config enabling `features.respect_system_proxy = true` and administrative macOS proxy/trust provisioning. That feature is default **false**, so merely setting system proxy preferences is not a proven route. A CONNECT tunnel alone is not a credential broker; interception must terminate TLS, strip dummy authorization and inject the upstream credential outside the guest. | Candidate-specific proxy selection, TLS trust and any websocket routing, CONNECT destination restriction, upstream joins and no direct/PAC fallback. | Adds system proxy feature/config, CA and interception service plus fallback/cleanup validation; strictly more moving pieces for this lane than a direct system URL. |

Network-route source basis:
[HTTP dependency](../../../codex-rs/http-client/Cargo.toml) enables
`rustls-tls-native-roots`; [custom CA](../../../codex-rs/http-client/src/custom_ca.rs)
lines 242–270 loads native roots for the explicit rustls path.
[Feature default](../../../codex-rs/features/src/lib.rs), lines 1087–1090,
disables `RespectSystemProxy` by default;
[core factory](../../../codex-rs/core/src/config/mod.rs), lines 1838–1845,
selects the proxy policy; [macOS resolution](../../../codex-rs/http-client/src/outbound_proxy/macos.rs),
lines 91–122, reads SystemConfiguration/CFNetwork settings.
These support conditional feasibility, not a claim that uninstalled hosts/PF/CA
controls have been demonstrated. Managed configuration is another higher layer,
but requires profile provisioning and adds no benefit over the existing Unix
system file. A binary shim, injected CLI option or modified adapter is a transport
change, not a hidden no-change alternative.

## Evidence that inference actually used the broker

Retain the system-config hash and guest identity, binary SHA-256/build provenance,
five-module package digest, fixed-forward identity and external fence policy.
Launch via the unchanged `TmuxAdapter`, not a manual binary command. In a
separate disposable pre-case control, send a unique nonsecret challenge, complete
a real SSE response and tool-call round trip, and join the broker's inbound
request digest/run identity to its authenticated upstream provider/model/effort,
upstream request/response IDs and the worker rollout's session/turn/response IDs.
Strip authorization from evidence. Preserve retries and failure attempts.

Collect continuous external egress observation for IPv4/IPv6 and alternate
routes, demonstrating that guest inference cannot reach the public endpoint.
In a bounded negative control close/reject the fixed broker lane and require
inference failure with no public connection or successful completion; restore
it and obtain a fresh correlated successful response. This distinguishes a
configured URL from actual broker traversal. Challenge text alone, a socket,
an echoed ACK, or a successful model answer without broker joins is insufficient.

For acceptance, bind those receipts to action/claim/allocation, exact runtime,
ACK and START/RETURN turns, including owner intents and observations as defined
in recipe step 9. A broker that only returns canned answers or silently changes
provider/model/effort cannot qualify. The server must support SSE and tool round
trips while rejecting generic CONNECT, arbitrary URL/source retrieval and
credential export. Only an already authorized entitlement and spend lane is
usable; no price estimate or new permission is invented.

## Existing product authority and the four booleans

**Authority:** the frozen round-114 brief relays Travis's existing written ruling:
“the sandbox exists only to stop the functional tester reading the codebase; if
it cannot read the codebase and can perform functional tests, the arrangement
is successful; take the simplest possible path to qualification and, if the
described conditions are met, treat it as qualified.” This records an existing
decision, not a newly requested or fabricated one. Its provenance is the supplied
brief hash above; this worker did not independently retrieve an earlier message.
The manager should link that existing ruling in the qualification evidence.
No fresh Travis decision or limited-branch acceptance is required for this
covered narrowing.

Under **that same Travis ruling**, the manager may attest each gate field only
after its evidence exists, with these meanings on the record:

| Field | What True asserts under the existing ruling | What it does not assert |
| --- | --- | --- |
| `isolated_transport` | The actual tester and children operate across a disposable guest/tool boundary that prevents access to the tested Corbanu codebase/history/prior findings and still supports the frozen functional workflow. Host administration is outside their capabilities. | Worker/controller UID separation, denial of readable staged Python transport helpers, denial of same-UID guest process/IPC access, or universal hostile-worker confinement. |
| `isolated_profile` | The exact run uses fresh HOME and all three profile aliases with synthetic fixture data; no imported operator profile, real auth clone, personal Keychain or upstream secret is exposed. | That the auth symlink or its dummy marker is unreadable, that every same-UID file is inaccessible, or that native personal-credential workflows passed. |
| `negative_access_probes` | The actual executor and children produced recorded negative source/history/prior-findings and boundary-canary probes, with known-existing targets and positive packet/binary/PTY controls. The results support the source-blindness claim and usable functional lane. | That a missing guessed path proves isolation, that staged helper reads were denied, that all IPC/children are blocked, or that probes prove an unbounded security property. |
| `mediated_inference` | Real authorized worker inference traversed the external broker with upstream secrets outside actor reach, joined response provenance and public-endpoint bypass denied. | An unreadable dummy auth file, a transport-native credential API, a mere URL-setting claim, or universal provider compatibility. |

**Still unproven:** actual guest provisioning, source-denial/control outcomes,
broker traversal and fail-closed behavior, exact-package ACK/START/RETURN cases,
recurrence/recovery evidence and independent review. Stronger malicious
worker-versus-controller isolation is outside this accepted narrowing and remains
unproven. The four fields remain unset for this worker's return; no qualification
is claimed. The gate checks attestations, not the truth of these conditions.

**Restored disclosure — session-less guest:** a session-less guest may qualify
the narrow terminal/PTY/TMUX and user-domain launchd cases; it cannot qualify
GUI-session, interactive personal Keychain or desktop permission workflows by
implication. This is the round-112 limitation restored in the current return.

**Restored disclosure — round-112 incidental nonzero:** historical round-112
top-level verification nonzeros were **0**; its single incidental inspection
nonzero was **1**, exit **2**, from `rg` against the nonexistent guessed
`qa/initiative-control/management-bootstrap/test_owner_promotion_94_preflight.py`.
That was a file lookup failure, not a test failure. This is historical accounting,
not a claim about every child exit or this round's inspection commands.

## Updated recipe and consistency check

The [complete ordered recipe](owner-limited-113b-return.md#executable-qualified-path-recipe-future-work-only)
now uses the administrative system config. Steps 1–4 freeze authority/cases,
separate standard and administrative capabilities, stage only allowlisted assets
and verify pins. Step 5 provisions the external broker and fixed reverse tunnel.
Step 6 installs the supported system layer before preparation, authors only a
synthetic auth marker, and requires positive and broker-down controls. Step 7
probes the actual boundary; step 8 runs the unchanged disposable owner lifecycle;
step 9 correlates real ACK/START/RETURN; step 10 applies the authority-backed
boolean meanings and independent evidence review; step 11 tears down the
recorded test resources and system config.

The fixture initializer, gate predicate order and audit-only quiescence procedure
are retained. The old project-local endpoint snippet and the claim that project
config supplies the URL have been removed. No step depends on an inherited
variable, post-prepare config edit, nested provider override, added launch flag
or worker sudo. The existing item-5 procedure continues to leave dispatch enabled,
owner armed fixture-only and service installed/present. It was not executed here.

## Verification

Read `docs/development/test-isolation.md` before any test. Built a disposable
venv under `env -i` from `scripts/initiative_control/requirements.txt`.
[Setup](owner-broker-114-venv.txt) exited **0** with markdown-it-py 3.0.0,
mdurl 0.1.2 and slack-sdk 3.44.1. [Root record](owner-broker-114-test-root.txt)
identifies the private test root. HOME/CODEX_HOME/CORBANU_HOME/PFTERMINAL_HOME
were all pinned to its fresh `home` directory for discovery and separate
`focused-home` for the focused replay; TMPDIR was respectively `tmp` and
`focused-tmp`. Both runs used a clean PATH, `PYTHONPATH=scripts/initiative_control`,
`CORBANU_TEST_NO_NATIVE_KEYRING=1` and `PYTHONDONTWRITEBYTECODE=1`.
The runs overlapped in time with separate disposable state and the same venv.

From repository root, that venv's Python ran:

```text
python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -B -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_control test_decision_manager test_slack_transport
```

- [Full discovery](owner-broker-114-suite.txt): **880 passed**, **520.485s**,
  exit **0**; failures **0**, errors **0**, skips **0**. Failure names: **none**.
- [Focused replay](owner-broker-114-focused.txt): **483 passed**, **398.507s**,
  exit **0**; failures **0**, errors **0**, skips **0**. Failure names: **none**.
  Breakdown: owner 133, owner TMUX 52, feed 39, attention 24, control 39,
  decision manager 113, Slack transport 83. These overlap discovery:
  **880 unique tests**, not 1,363.
- Logs retain expected negative-fixture refusals and ResourceWarnings. No native
  credential prompt or live-profile access was observed. No Rust test, formatter
  or fix command ran.
- Syntax-only checks passed for **2 Python**, **5 shell** and **1 TOML** examples;
  all local Markdown links resolve. The initial syntax-check helper exited **1**
  with `IndentationError: unexpected indent` because it retained Markdown list
  indentation. A corrected `textwrap.dedent` extraction passed without changing
  or executing any example. This was a helper error, not a suite failure.
  Incidental source lookups also returned nonzero for guessed missing filenames;
  no claim of zero inspection-command nonzeros is made.

This document records no qualification run. The evidence-only correction adds no
user-facing behavior; independent functional design/TUI/live-repository/benchmark
execution is not applicable to this edit, and the later real-worker functional
gate remains required under the recorded authority. Applicability remains for
the integrator to accept; no independent review or human acceptance is invented.

## Changed files

Only six files changed, all in `qa/initiative-control/management-bootstrap/`.
Executable/source files changed: **0**. Final `git diff --check` and scope
inspection passed. No live journal/schedule/transport/coordinator, VM, promotion,
qualification, commit or push was touched. Earlier raw test evidence is unchanged.

| File | Final lines | Added / deleted |
| --- | ---: | ---: |
| `owner-limited-113b-return.md` | 677 | +86 / -34 |
| `owner-broker-114-return.md` | 247 | +247 / -0 |
| `owner-broker-114-suite.txt` | 939 | +939 / -0 |
| `owner-broker-114-focused.txt` | 525 | +525 / -0 |
| `owner-broker-114-venv.txt` | 15 | +15 / -0 |
| `owner-broker-114-test-root.txt` | 1 | +1 / -0 |
