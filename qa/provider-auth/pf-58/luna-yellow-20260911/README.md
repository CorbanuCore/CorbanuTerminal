# Sequential Luna Max functional-testing pilot

User-authorized on 11 September 2026. This is a routine QA experiment and evidence
collection task, not adoption of a new canonical process, a product modification,
a new code review, or release approval. Main has not been rebased or merged for
this experiment; the user asked to try the method first.

## Frozen inputs and candidate

- Twelve yellow human checks: **1, 2, 3, 5, 6, 7, 11, 12, 15, 20, 24, 26**.
- Source: `humanTest.html`, SHA-256
  `610f81e16f52ebc2cfc95d0901685593c00f2d21be2d1db691dacfd21662e452`.
- Original headings, actions, expected outcomes and failure criteria are preserved
  in each case's `human-instructions.md`. Historical prerequisite notes/results
  were omitted from agent input to avoid telling agents what to find. The full
  original HTML remains frozen in the coordinator's external packet.
- Installed Mac release package: `macos-candidate-final9-20260910`, version 0.1.41.
- CLI SHA-256:
  `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`.
- Source branch: `feat/provider-reauth-health`, committed head `eb7af932bc8cc859122b5b2dbef223fbd45ea25c`.
- Each executor uses that same CLI, `gpt-5.6-luna`, and
  `model_reasoning_effort="max"`. Each is a new standalone CLI session, not a
  full-context in-app subagent. Runs are sequential, with separate profiles,
  sockets, working directories and report directories.

## Isolation and preflight

macOS `sandbox-exec` supplies the external enforcement boundary. File reads are
denied by default except required system/runtime locations and the individual
run directory. Writes are restricted to run state and required terminal devices;
the copied executable package and supplied instructions are read-only. Apple
Events are denied. Other run directories, the repository, prior reports and
the user's ordinary home content are not readable. File metadata remains readable
for runtime compatibility. No source content is supplied to the executor.

Before each case, coordinator probes verify:

1. Reading this repository's `AGENTS.md` is denied with `Operation not permitted`.
2. Listing the main checkout is denied.
3. Reading the previous dispositions report is denied.
4. Following a scratch-directory symlink to repository content is denied.
5. The copied candidate and TMUX version commands still work.

An additional child-TMUX probe confirmed repository reads remain denied from a
shell created by the sandboxed TMUX server. The dedicated preflight Luna agent
successfully created a private TMUX session, captured `PILOT_TMUX_OK`, and cleaned
up that server. Earlier infrastructure attempts failed because of an initial
coordinator filename bug and missing system-loader/PTY allow rules; these were
fixed before acceptance-case dispatch and are not product failures.

This is **filesystem-enforced local-source isolation**, not a claim of VM-grade
isolation or adversarial sandbox certification. Network access remains available
for model inference; source downloading, external services, GUI automation and
other TMUX servers are explicitly prohibited to executors. A future canonical
method should decide whether network mediation and stronger IPC isolation are
required. Do not describe this pilot as an air gap.

## Execution boundaries

Agents see only their human case, candidate identity, TMUX operating instructions,
and necessary safety/environment constraints. They send real TUI keys, with prompt
text and Enter separately. They may make brief harmless live model requests,
inspect menus, cancel flows, resize and restart only their own target.

The provisioned test profile is a private clone of the earlier isolated live
OpenAI/Claude test profile. It is **not** the user's ordinary launcher profile.
It uses file-backed account storage and the native-keyring test fallback. Its
startup update check is disabled. These limitations are disclosed in every agent
prompt, so no claim about native Keychain, full launcher parity or version-notice
behavior may be inferred from a quiet TMUX launch. Credentials are not placed in
prompts or reports; agents must not inspect credential files. No global trace
logging is enabled with this live profile.

No payments, key creation/revocation, credential replacement/deletion, real policy
changes, browser sign-in completion, or GUI automation are delegated. Missing
credentials/services and unavailable native interactions remain blocked. Partial
TUI coverage must not replace the original native criteria. The source guide's
mixed historical build references remain an explicit documentation limitation.

The user separately reported successful live messages/provider switching without
Keychain prompts, then reported a second launcher instance on the wrong desktop.
Those reports were **not given to the agents** and do not become agent evidence.

### Fixture correction during the pilot

Cases 1–3 used fixture version 1. The encrypted vault was copied without relocating
its home-derived fallback-key filename, so Claude's vault metadata was unavailable
in the copy. This is a coordinator fixture defect, not evidence of a new product
regression. The provider-catalog executor exposed it through the UI. Its original
observations and verdict are retained, not rewritten.

The coordinator inspected the relocation mechanism (not shared with executors),
copied the existing private fallback key to the identity expected for the new
private home, and verified only fixture setup: OpenAI and Claude both display
`Enabled · configured`, with Claude current. No plaintext credential was decrypted
or logged during this copy. Later cases use fixture version 2. Ordinary user
profiles and native Keychain permissions remain unchanged. This demonstrates why
a canonical implementation needs profile-provisioning checks before agent dispatch.

### Literal-instruction correction

The initial extraction retained instruction words but dropped HTML inline-code
formatting. Case 5's executor submitted only `FABLE_51_OK`, not the prescribed
`Reply with exactly: FABLE_51_OK`, then reported the different reply as a failure.
The action receipt disproves faithful execution of that criterion. The raw agent
verdict is preserved, but the coordinator classifies that attempt as invalid, not
as a product failure. Extraction now retains literal input boundaries with Markdown
backticks. Each case's packet is taken from the actual prompt it received; earlier
inputs are not silently replaced with the corrected formatting. A fresh replay
will use the unchanged original expectation, without receiving prior results.

## Results and review accounting

[Browse the report](report.html). Individual `report.md` and `results.json`
preserve executor verdicts. `actions.json` records the actual completed shell/TMUX
commands and their output, separate from each agent's prose. Coordinator copying
redacts recognizable credentials/login codes; captures must be inspected before
any publication. Private profiles and raw runtime logs stay outside Git.

The original nine approved code/design/evidence reviews remain exhausted. This
new user request authorizes sequential **test execution** by Luna Max, not another
code review or an unrequested redesign. No human checkboxes are marked and no
full acceptance/merge readiness is implied. Canonical process adoption and the
newer plans on main are follow-up decisions after evaluating this experiment.
