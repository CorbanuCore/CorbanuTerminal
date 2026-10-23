PF-27-S02 is merged as PR #191 (merge commit `699bd4a82f`). It sits behind a new flag, `secretless_agent_launch`, which is off by default, so Permissive behaves as before. The sprint stays `in_progress` because four items are still open (listed at the end), and two of those need a decision from Travis.

**Bookkeeping you asked for (same PR):**
- **PF-27-S04 is completed and archived.** Its per-sprint gate was already met in PR #180. The archived record has a table showing where each open item went:
  - Core's own model-client auth (and vault-label credentials) goes to the new **PF-27-S05**, placed after PF-27-S02 in the broker lane, depending on PF-27-S02.
  - The revocation trigger is now an explicit Remaining item in **PF-23-S03**.
  - The containment items are covered by PF-27-S02 below.
  - The Windows broker goes to a new **PF-27-S06**, together with Windows for this sprint. I created PF-27-S06 myself because you said to record Windows as a follow-up; Travis may want it moved to the P1 plan.
  - The flag-on limits (a dead broker isn't restarted, method/path limits, private-IP GitHub Enterprise, upload timeouts) are kept as documented behaviour and not scheduled anywhere.
- The plan's front matter, broker-lane row and archive list, the sprint index, `docs/sprints/index.md` and `mkdocs.yml` are updated. Both checkers pass.

**What PF-27-S02 ships (macOS and Linux; Windows refuses every protected launch with a reason):**
- **No raw secrets in agent commands.** Their environment is reduced to an allowlist: provider keys, tokens, passwords, credential helpers, start-up hooks and passwords inside URLs are removed. Login shells and shell snapshots are off. Raw secret values in a command line or in typed input are refused. This applies to shell and unified-exec commands, escalated commands, the `!` shell, hooks and MCP servers.
- **Every launch must run in the OS sandbox.** Commands cannot read Corbanu's credential, log and history files or common CLI credential files, and nothing under `CODEX_HOME` is writable. Unsandboxed, full-access, escalated and remote-environment launches are refused with a stated reason. The patch, edit and image tools get the same restrictions, or no file access at all.
- **Corbanu protects its own memory** from other processes (non-dumpable on Linux, no debugger attach on macOS).
- **The design conflict is settled.** Secret-looking variables are stripped first; the broker then takes the real tokens from Corbanu's own environment and gives agents dummies. It only does this for tokens the user's environment settings would have passed anyway.
- **The four broker items handed over from S04:**
  - **Broker sandbox:** the broker confines itself (Seatbelt on macOS; seccomp, plus Landlock where the kernel has it, on Linux). It cannot run programs, trace other processes or write outside its own directory, and with the flag on a broker that can't confine itself is refused.
  - **macOS setup-pipe window:** closed. Secrets now go over a socket the broker creates, and each side checks the other's process id.
  - **Socket directory:** moved out of the agent-writable temp directory into `CODEX_HOME/run`, or a per-user directory the agent can't write when that path is too long for a socket.
  - **Strip vs dummy:** settled as above.
- **One behaviour change for `isolated_credential_broker` used alone:** the new socket handshake and the broker's self-confinement now apply there too. It's recorded in the sprint and the evidence README.

**Gate evidence:**
- **Tests (`just test`, after `just fix` and `just fmt`):**
  - `codex-core pf_27_s02`: 12 passed.
  - The protocol, process-hardening, network-proxy, hooks and features suites: 722 passed.
  - A core subset (sandboxing, unified exec, config, network proxy and related): 853 passed.
  - `codex-rmcp-client`: 203 of 204. The failure is a keyring test that can't reach the native keyring in the isolated test setup; the changed code only runs with the flag on.
  - Linux was tested in Docker: the `pf_27` tests pass and the broker confines itself with seccomp (that kernel has no Landlock). PR CI has no Linux Rust test job, so Docker is the only Linux test coverage.
  - All PR checks were green at merge.
- **GLM 5.2 runs, flag off vs on:**
  - Credential-looking variables the agent sees: four with the flag off (`ZAI_API_KEY`, the canary key, `SSH_AUTH_SOCK`, the GitHub token); only a dummy GitHub token with it on. The test server still reports `authorized: true`.
  - Reading `auth.json`, the vault store, `.env`, `config.toml` and shell snapshots: allowed with the flag off, denied with it on.
  - Corbanu's own environment can't be read from agent commands.
  - Full access is refused with the reason.
- **Review:** one Opus 5.5 High review. The first pass asked for changes: several credential files were still readable, the in-process file tools bypassed the contract, plus five lesser issues. All were fixed or recorded; the re-check approved with minor notes, which are also fixed or recorded. Both reports are in `qa/security-levels/sprints/PF-27-S02/`.
- **Videos** (links in `qa/demos/index/PF-27-S02.md`):
  - [agent sees only a dummy, broker still works](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-secretless-env-03c9dfd9716d-2026-10-06.mp4)
  - [protected paths and Corbanu's environment are out of reach](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-protected-paths-03c9dfd9716d-2026-10-06.mp4)
  - [command outside the sandbox is refused](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-unsandboxed-refused-03c9dfd9716d-2026-10-06.mp4)
  - [broker confined in a private directory](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-broker-contained-03c9dfd9716d-2026-10-06.mp4)
  - [control run with the flag off](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-baseline-flag-off-03c9dfd9716d-2026-10-06.mp4)

**Decisions for Travis:**
1. **Unsandboxed routes.** MCP servers, hooks, the `!` shell and app-server `command/exec` get the clean environment but still run outside the OS sandbox. The options are to refuse them in protected levels, or to accept them as user-configured or user-started and show them as "not contained" in the PF-41 inspector. I recommend the second.
2. **Claude panes** (and other external provider tools) aren't covered at all. I recommend handing this to the TUI lane to decide between blocking them and bringing them under the contract.

**Still open in PF-27-S02:**
- The two decisions above.
- A Linux end-to-end agent run on a host with Landlock. Docker Desktop's VM ran out of disk building the full binary.
- Windows, now PF-27-S06.

**Known limits (in the evidence README):**
- Shell profile files and credential files outside the deny list (such as `~/.ssh`) stay readable, and a nested `bash -l` still loads profiles. PF-29's migration is the fix.
- The command-line and input check only knows Corbanu's secret-named variables at start-up, not vault values or keys resolved later.
- On macOS, unsandboxed same-user processes can still read Corbanu's launch environment. PF-27-S05 moves Core's own keys out.

**Process notes:**
- My sprint record doesn't reserve three files other lanes hold: `core/src/config/mod.rs` (reserved by PF-60-S03; I added three small hunks), the feature-flag registry and config schema (one line each), and five new demo specs under `qa/demos/specs/` (reserved by PF-30-S02). The integration gate says the integration owner should coordinate these.
- The sprint's CI checkbox is still unticked even though CI passed. Adding a docs commit would have restarted the roughly 1.5-hour CI run, so I left it for the next PF-27-S02 update.