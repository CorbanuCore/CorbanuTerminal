# OpenClaw reference-review evidence

Planning evidence only. No Corbanu feature or release is qualified by these results.
See [the source review](../../../../docs/plans/openclaw-source-review-2026-08-28.md)
for reviewed ranges, callers, adoption decisions and unexecuted cases.

## Snapshot

- Upstream: `https://github.com/openclaw/openclaw.git`, default branch `main`.
- Pin: `13adff02ca3897768d80d2bca18f5acf08c55d91`.
- Historical comparison: `6ce272c2a662f81b7779507335d91de4d61c589b`.
- Package version: `2026.8.1`; this is a source snapshot, not a release install.
- Download: shallow clone with hooks disabled; the historical commit was fetched
  with `--depth=1 --no-tags`. No upstream install/startup scripts were run.
- Original review checkout: `/Users/travisgood/Documents/ChatGPT/openclaw-review-2026-08-28.xLJPKK/source`
  (historical temporary path; no longer present).
- Durable local reference: `/Volumes/CorbanuDrive/Corbanu/.codex-work/references/openclaw-13adff02`,
  recreated on 2026-08-30 as a clean detached checkout at the same pin with
  hooks disabled. All 42 `source-manifest.json` file digests match. No install,
  startup script or upstream test was run while creating this reference.
- Corbanu planning checkout: `/Users/travisgood/Documents/ChatGPT/corbanu`, `main`,
  base `f173a0bc97c7495d134a67079aadfbe3657d11a7`, with pre-existing reconciliation
  edits preserved. PF-13/PF-29 implementation worktrees were not changed.
- Node: `24.15.0`, darwin-arm64, downloaded separately from nodejs.org;
  archive SHA-256 `372331b969779ab5d15b949884fc6eaf88d5afe87bde8ba881d6400b9100ffc4`,
  matched the official downloaded SHASUMS256.txt. System Node 25.2.1 does not meet
  this upstream pin's engine range and was not used for these tests.

## Outcomes

| Evidence | Result | Meaning |
| --- | --- | --- |
| `upstream-tests.json` / `.log` | 2 files, 87 tests passed | Unmodified external-content tests (85) and turn-state tests (2), minimal harness |
| `probes.json` | 10 passed observations | Includes confirmed limitations, not 10 Corbanu security passes |
| `source-manifest.json` | Source pin and file digests | Referenced source identity; not an assertion that every line in each file was audited |
| `runner-package.json`, `runner-package-lock.json` | Exact isolated runner dependencies | Not OpenClaw's complete workspace lock or official test environment |
| Plan/sprint validators | Pass: 1 active plan, 70 current / 72 archived sprints | Existing counts unchanged; includes seven unrelated Autoreview drafts |
| Plan/sprint checker unit tests | 9 passed | Three plan tests and six sprint tests |
| Adoption-link checks | 46 sprint records passed | Exact pin and all OC anchors resolve |
| `docs-build.log` | Strict MkDocs build passed | Existing excluded-archive links are informational; no new broken-link warning |
| `git diff --check` | Pass | No whitespace errors |

The minimal Vitest config aliases only `@openclaw/normalization-core` to source,
omits OpenClaw's global test setup and runs one worker. No upstream test/source
file was patched. The probes use synthetic strings, no credentials, network
service, Gateway, browser, container or system trust-store modification. Both
runners reject the wrong commit or changed tracked upstream files.

Observed limits include short-value omission, bounded-registry eviction, split
redaction calls with no carry, raw sentinel opt-out and a request prefix emitted
before a later stream refusal. They are explicitly asserted as current behavior;
Corbanu's protected-mode regressions must assert its stronger desired behavior.

Not run: the full upstream suite, proxy network/established-tunnel revocation
tests, persistent-memory suites, migration/runtime suites, provider transports,
browser/CDP, platform containment, Corbanu Rust/TUI/live-repository or human
acceptance. Those remain implementation/qualification work, not silent passes.

## Reproduce

Use a separate disposable directory and a supported Node 24.15.0 runtime. Clone
the origin and check out the exact pin above; keep tracked files unchanged.
Set `OPENCLAW_REVIEW_SOURCE` to that absolute checkout, `REVIEW_EVIDENCE` to this
evidence directory, and `REVIEW_RUNNER` to a separate empty runner directory.
These are run inputs, not fixed repository-policy paths.

Copy `runner-package.json` to the runner as `package.json` and
`runner-package-lock.json` as `package-lock.json`. From the runner directory:

```sh
npm ci --ignore-scripts --no-audit --no-fund
node node_modules/vitest/vitest.mjs run \
  --config "$REVIEW_EVIDENCE/vitest.config.mjs" \
  --reporter=json --outputFile="$REVIEW_RUNNER/upstream-tests.json"
node --import tsx "$REVIEW_EVIDENCE/probes.mjs" > "$REVIEW_RUNNER/probes.json"
```

Run with a minimal environment, a writable temporary directory and Git on PATH.
The recorded run used `env -i` with only PATH and OPENCLAW_REVIEW_SOURCE. Running
these helper tests is not installing OpenClaw or certifying all its dependencies.

Documentation validation used the repository requirements in an isolated uv
environment because system Python had no MkDocs installed:

```sh
python3 docs/plans/check.py
python3 docs/sprints/check.py
python3 -m unittest discover -s docs/plans/tests
python3 -m unittest discover -s docs/sprints/tests
uv run --with-requirements requirements-docs.txt --no-project \
  mkdocs build --strict --site-dir "$REVIEW_RUNNER/docs-site"
git diff --check
```

## Planning follow-through

The active plan and source reconciliation link OC-1–11; 40 new security drafts
and six existing foundation/qualification sprints have explicit adoption checks.
No sprint activation, completion, dependency reorder, runtime change, commit,
push or release is part of this evidence update. Permissive stays compatible.
