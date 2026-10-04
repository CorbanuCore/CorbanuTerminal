# PF83 qualification preparation receipt

Prepared 2026-09-14 UTC. Preparation owner only; not acceptance executor or
evidence reviewer. Routine QA/package preparation supporting PF-83-S01
(`in_progress`) in `docs/plans/active/p0-security-levels.md`. Product heading:
**Permission selection confirmation — TO BUILD**, “A submitted selection is not
a confirmed change.” No source or planning edits were made by this preparation.

Private preparation root:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-preparation.vwQlK8`.
The bounded pre-build checkpoint is `CHECKPOINT.md` there. This receipt is the
only preparation-authored file in the frozen worktree.

## Exact candidate and package

Source worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913`.
Base: `005cc644f59b1e762e5497b329e106c67925d4ed`.
Patch SHA256: `e36396dcd62ad8daf59832302ea69f10e24770ddab3d44ce1b99122ed69cf96b`.
All 32 manifest paths, exact changed/new/deleted path set, base and reverse-apply
check matched before and after building. `source-before.json` and
`source-after.json` are byte-identical. No patch was applied by this worker.

Canonical standalone package: `package/` in the private preparation root.
Archive: `corbanu-terminal-package-aarch64-apple-darwin.tar.gz` there.
Version `0.1.42`, `aarch64-apple-darwin`, documented local `dev-small` profile;
unoptimized, stripped local qualification candidate. It is not a DMG, native
Applications installation, Developer ID/notarized release, or the running app.
Read-only `codesign -dvv` reports linker-signed ad-hoc identity `corbanu` with no
TeamIdentifier; no signing or installation step was run by this preparation.

| Artifact | SHA256 |
| --- | --- |
| Archive | `c1e774d068dd3c9be42c8e888f401b7d79566c948c3ebd5a7c88600067edd7a5` |
| `bin/corbanu` | `57b19153926f778f69dc12243703796c07d100e97f47253244b23d321659d9bd` |
| `bin/corbanu-acp` | `8d79871c4da88927a374490c14309755ee0a21f26af9115b080edc52278d06e5` |
| `bin/corbanu-walletd` | `06fd3573e7563d32d0bb4cd81dab3b89bcdf4116872c6ac61cffc8afa51ddb7f` |
| `bin/codex-code-mode-host` | `c2c2f9316a9d24aef7f655f3109f5eee55e7bc17415821e441eba87c38af5cc4` |
| `codex-path/rg` | `a326a1fb48074202e9ad41e4cd1e389eeea372c8c6f7d7e80da81176d5d9430e` |
| `codex-resources/zsh/bin/zsh` | `db6fe1a78eaceaff3b0f0cde25fc25afe466d61b0bf76b4ebe35812e4bc8dd71` |
| `package-manifest.json` | `b1cb4db32ebb63f17dbfa6ee99ead38af1a2ad04fefc79a246c12949df5331a7` |

`corbanu-debug` is the canonical relative symlink to `corbanu`. The manifest
records all 13 packaged file/link entries, matching helper hashes, source receipt,
build script/log and archive hash. Archive entries were streamed and compared
against the entire package manifest without extracting or executing them.

## Actual build command

Executed the private `build.sh` below, with stdout/stderr retained in
`build-attempt-01.log` (exit 0). Rust compilation completed in 2m53s; warnings
remain in the log. Existing workflow: `scripts/codex_package/README.md`,
`scripts/build_codex_package.py`, and the Corbanu variant in the checked-in
package CLI/release workflow. No source wrapper or packaging implementation changed.

```sh
/Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-preparation.vwQlK8/build.sh
```

The exact script/environment is retained, SHA256
`073304d91ccc66cac0bc62875e83abe7cf94b8d50500fa5908fa65464612f197`.
It clears inherited environment with `env -i`, sets private HOME/TMPDIR/CARGO_HOME/
CARGO_TARGET_DIR/XDG_CACHE_HOME, uses eight Cargo jobs and existing Rust1.95.0,
disables Python bytecode writes, then runs from the source root:

```sh
/opt/homebrew/bin/python3 -B scripts/build_codex_package.py \
  --target aarch64-apple-darwin --variant corbanu --cargo-profile dev-small \
  --cargo /Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-preparation.vwQlK8/cargo-locked \
  --package-dir /Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-preparation.vwQlK8/package \
  --archive-output /Volumes/CorbanuDrive/Corbanu/.codex-work/pf83-preparation.vwQlK8/corbanu-terminal-package-aarch64-apple-darwin.tar.gz
```

The private Cargo wrapper invokes installed
`/Users/Neo/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo` with
`--locked --offline`. The actual grouped build was:

```sh
cargo build --target aarch64-apple-darwin --profile dev-small \
  --bin corbanu --bin corbanu-acp --bin corbanu-walletd \
  --bin codex-code-mode-host --locked --offline
```

Only registry/git dependency caches from existing `.codex-work/cargo-home` seeded
the new private Cargo home with APFS copies; no credentials/config/profile copied.
Target cache started empty. Existing packaging utilities downloaded and verified
the pinned V8 pair and rg/zsh artifacts into private TMPDIR. Their retained hashes
are in `closeout.json`. No shared build cache was written.

## Isolation audit and concrete preparation observations

Read root policy, code-blind README/isolated-execution contract, the existing
`.codex-work/functional-execution.ikteKU` README/relevant utility boundaries, and
the six policy-linked private PF58 pilot scripts. At-rest SHA256 pins are retained
in `closeout.json`. No original utility or historical attempt was edited or run.
No raw real credentials or live profiles were opened/cloned.

The only native synthetic preflight reused the literal PF58 `pilot.py` Seatbelt
policy with a new private run-root substitution. It did **not** call `prepare()`;
that function copies live auth and is unsuitable here. One-shot preparation
scripts are confined to this private directory; no generalized runner/product
feature was introduced.

`native-01/receipt.json`, SHA256
`e38a57e18353988aef8853ee96f46b6bcf995a107995adf621df5cf4d6585aad`:

| Boundary/control | Actual observation |
| --- | --- |
| Existing synthetic source/history/auth canary read/list | Each returned exit1 with `Operation not permitted` |
| Child shell source/history/auth reads | Each returned exit1 with `Operation not permitted` |
| Symlink escape to existing synthetic source canary | Exit1, `Operation not permitted` |
| Synthetic packaged-file modification | Exit1, `Operation not permitted` |
| Packet read and TMUX runtime | Exit0 |
| PTY text then Enter, sent separately | `PTY_RECEIVED:PF83_SYNTHETIC_KEY`; child canary read denied |
| Unapproved network | **DENIAL FAILED:** confined process connected to a live, preparation-owned loopback TCP listener; exit0 and peer accepted |
| Cross-run IPC | **DENIAL FAILED:** confined process connected to a live Unix socket in the separate preparation-owned peer directory; exit0 and peer accepted |

The synthetic listeners were closed and the owned socket removed after probing.
No real service/public endpoint or existing session socket was contacted.
No model/inference request was made. Negative probe failures and PTY raw output
are retained verbatim; none is presented as a functional product failure/pass.

`native-package-02/receipt.json`, SHA256
`9155cfb5fa7787ba8e7fee1f0fdba1fce5fb4ac230f59a2b73d0be768049315d`:
fresh exact package copy under the same historical policy (new root only) launched
`bin/corbanu --version` with exit0 and `corbanu 0.1.42`; an attempted append to
that actual executable returned exit1/`Operation not permitted`. Every packaged
file/link hash remained unchanged. This is a launch/write control, not a real-key
product acceptance run or proof of the complete executor boundary.

The existing `.ikteKU` utility has a model-only action mediator plus nonroot,
network-none, read-only, no-new-privileges/seccomp Docker browser execution with
private PID/IPC and synthetic child probes. Its cached image was confirmed
read-only as `sha256:fdbf57e27079258be976cf5d8d309c1c00faadfc24cec5c562caf661b27926bd`,
`linux/arm64`. It is fixed to DEC-001/browser actions and a different candidate;
it cannot run Mach-O macOS or native TUI/Keychain/launcher flows. No Docker object
was created, started, stopped, removed, or modified here.

The associated model transport is a separate privileged broker, not a native
target sandbox: its policy allows an explicit provider-auth read and outbound
443, with model tools disabled and a bounded action interface at the coordinator.
It cannot simply be handed to a target shell. Its native-auth validation/linking
was not invoked. The earlier loopback-only synthetic transport blocks helper
execution, so it also cannot substitute for a PTY/command-executing target.

**Available PF83 native executor: none qualified by this audit.** Narrow missing
prerequisite: a permitted native macOS target/child boundary with exact network
allowlisting, cross-run IPC/process restrictions and a bounded PTY action bridge
to credential-free model-only mediation. Old pilot default network/IPC access is
concretely disproven; process signaling/Mach/other host paths and child network
also lack a complete qualifying probe set. An unrestricted fresh subagent is not
blind. Existing Linux browser enforcement and instruction-only native controls
must not be relabeled as the missing native mechanism.

## Recommended launch inputs and open gates

The detailed coordinator-only handoff is private `LAUNCH-INPUTS.md`. Supply an
actual fresh isolated executor only the exact read-only package, neutral public
navigation, original case/group, and synthetic per-run task data/state. Original
F01–F11 are preserved byte-for-byte in `frozen-input/frozen-functional-cases.md`,
SHA256 `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`.
Keep approved scope clarifications additive and coordinator-owned; do not derive
new expectations from the implementation. Do not include this receipt, source,
patch, review findings or preparation scripts in executor context.

The actual executor and all target children must produce fresh negative/positive
receipts and effective policy/tool inventory before any case. Use separate
private HOME/state/evidence/short PTY sockets; no real profile clone. Model and
target inference require permitted mediation and a trusted non-readable broker
where live routes are needed. Required live-repository variants need separate
disposable task-data snapshots with recorded bases; none were created here.

All original cases, active-command real-key proof, both permission directions,
pending approvals, continuation/restart/recovery, route coverage and independent
evidence review remain open. No functional tests were claimed passed, no case was
waived, and no human acceptance, benchmark qualification or release readiness is
claimed. Parent Fable review continues independently. No commit/stage/push/merge/
install, live app/session/home/launcher/stable-link mutation, portfolio resumption,
admin/persistent-machine provisioning or remote-machine write was performed.
