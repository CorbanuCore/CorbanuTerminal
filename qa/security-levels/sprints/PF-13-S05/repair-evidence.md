# PF-13 accepted-review repair evidence

Status: implementation and qualification in progress; not a release approval.

## Authority and identity

- Travis requested that all existing work be committed before implementing the accepted fixes. The checkpoint is `f0a160eee5b820bc16fcd116013274d44e7d9be4`.
- Reviewed repair candidate: `a7ae94e4c9c01924c896d9f10b1f588f1727fc67`; all 16 repair source hashes match `repair-source-files.sha256`.
- Classification: product initiative, existing active PF-13 / PF-13-S05. Product authority: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02`; branch: `feat/pf-13-s02-scoped-vault-resolver`.
- Upstream ancestor reverified: `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa` from `https://github.com/openai/codex.git`. No upstream update, dependency, storage/config/wire contract or scheduler change.
- The amended sprint owns exact adapter boundaries. PF-23 native/profile wiring, Permissive behavior and rejected scoped dummy shaping remain outside this repair.

## Accepted fixes

| Item | Repair and proof |
| --- | --- |
| C1 | Scan complete stdout and stderr before retention checks; deny timeout/incomplete capture and overflow. Ten Python tests include an actual overflowing subprocess, both streams, before/at/after the limit, UTF-8 byte accounting and timeout non-disclosure. Capture still uses `subprocess.run(capture_output=True)`; this is not a streaming memory-bound claim. |
| C2 | Vault-owned thread-local, non-Send nested guard plus one permanent filtering hook; TUI logging and terminal-restoration hooks check before formatting/chaining. Tests exercise secret-bearing panic payloads, nesting, concurrent ordinary panics, recovery, and real production hooks in isolated subprocesses. |
| F3 | Copied encrypted vault cannot use the original home's mock-keyring account; canonical aliases preserve identity and case-distinct labels. Native symlink/junction CLI tests cover both home variables and persisted-posture downgrade denial. These tests do not protect against copying an unprotected fallback key or direct filesystem access. |
| F4 | Scoped bearer construction uses a zeroizing temporary and sensitive `HeaderValue`; legacy provider paths unchanged. The required HTTP wire copy still exists; sensitive Debug is not memory zeroization. |
| F6 | Correct the label normalization comment without changing behavior. |

## Automated checks

- `just fix -p codex-vault -p codex-network-proxy -p codex-tui -p codex-cli -p codex-core`, then `just fmt`: passed before the first affected run. A second TUI fix/format pass covers the repair-review additions; existing unrelated warnings remain.
- `python3 -B -m unittest scripts.test_security_credential_canary`: 10 passed, including final overflow byte-count assertions.
- Plan/sprint checkers and `git diff --check`: passed.
- macOS `just test -p codex-vault -p codex-network-proxy -p codex-security-policy`: 295 passed, zero skipped (33 Vault, 223 proxy, 39 policy). Nextest run `c03bb90b-d68e-4aa0-b924-f824595a6b14`; `repair-focused-macos-junit.xml`, SHA-256 `e13b58f601b88e8578894dec981e1b1d9622a019d2a30de4d10de11cd917322a`. Source files for those crates were unchanged by the subsequent TUI/Python hardening.
- First production-hook test failed before hook execution because the new JSON fixture omitted `authorization.schema_version`. Corrected to version 1; this failed attempt is not counted as panic containment proof.
- Corrected final production-hook test: passed, both hook installation orders and recovery. Nextest run `e1febb68-57bc-4148-a8e8-b773ec39ef1f`, one subprocess-parent test in 50.812 seconds; `repair-tui-panic-macos-junit.xml`. The 3,812 unrelated TUI tests were filtered, not represented as run.
- Final candidate true-TUI proof: startup, local provider success, Escape cancellation, subsequent recovery, clean exit, restored history with `resume --last`, and successful request after resume all passed. See `repair-tui-macos-checkpoints.md`. Candidate SHA-256 `cc60e86beccffcbbbcf8fa5239054f5d7041b01619ebc94f63ec8fe699ddf30d`, version 0.1.35. No real API credential or external provider was used.
- Complete Core: 3,407 executed, 3,388 passed, 19 failed; all 13 credential-named tests passed. The runner reported 19 pre-existing skips outside execution. See `repair-core-triage.md` and the complete failing JUnit artifact. The full-Core gate remains failed.
- Linux focused Vault/proxy/policy suite: 289 passed, zero skipped (host-conditional selection differs from Mac); run `de0ef7ad-2b55-4a78-9328-7f4e9cb8aa81`, `repair-focused-linux-junit.xml`, SHA-256 `8e4f534efb68932796fcc2ceceaf33dc46bc4a2bc629b0b6b20133ac2c88842a`.
- First Linux expanded canary: all nine probe groups / 47 tests passed, including all four CLI raw-export tests. Retained as `repair-credential-canary-linux-pre-identity-fix.json` (SHA-256 `06e40f8f6c85d6699a9383862cf5c2f34acea6f2c2e9dce3c41e96427dcc0945`), **not final artifact qualification**, for the identity issue below. Final Mac/Linux canaries pending.

Mac commands use `RUSTFLAGS="-C link-arg=-fuse-ld=lld"`, the installed Rust 1.95 LLVM linker directory on PATH and `CARGO_INCREMENTAL=0`. Selected rebuildable Core/TUI artifacts were initially cleaned with Cargo to recover 6.4 GiB. The companion-binary build subsequently exhausted disk while placing the final executables. After it exited, `cargo clean --target-dir /Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02/codex-rs/target` removed 27.7 GiB of regenerable build artifacts (85,285 files), restoring 26 GiB available. Source, committed evidence and other worktrees were untouched. Rebuild/qualification uses `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` to reduce artifact size, without excluding tests or changing assertions.

## Independent repair review

Reviewer remains Corbanu Terminal `moonshotai/kimi-k3` High, through the existing actual TUI; no fallback or nested reviewer. Reviewer host version 0.1.35, SHA-256 `9201d1a1d3623733f3ce163ba6652e1775293a9936392b009d91379583e2aa17`; session `01a0469a-3d91-7ba1-af03-da0cc86f7c61`.

First repair review ran 2026-08-28 06:00:09–06:04:20 UTC against `/tmp/corbanu-pf13-kimi-fixes.bCD3sx`, with 15 frozen source hashes verified. Raw normal response: `kimi-repair-review-1.txt`; structured output: `kimi-repair-review-1.json`. No reasoning trace or private auth log is retained.

Kimi found no new P1/P2 and two P3 hardening suggestions. Both were accepted within the same boundary: guard terminal restoration directly and include non-secret measured/limit byte counts in overflow errors. The final review used `/tmp/corbanu-pf13-kimi-final.btZYog` with 16 verified source hashes, including the corrected test fixture, from 06:09:20–06:12:07 UTC. Result: **no findings**, both P3 suggestions resolved; review cycle stopped. See `kimi-repair-review-final.txt`, `kimi-repair-review-final.json`, and `repair-source-files.sha256`. This is a repair-delta review, not a full-platform certification.

## Qualification limits

### Final-artifact identity repair discovered during qualification

The first Linux canary recorded pre-probe binary SHA-256 `55c4b0bab2ca1a5efc5bd52056ba0d197d6af168da3ac2948699409db67f70a8`, but a post-run hash was `ce3d70c3a36aff3912253ec2d73aba0a698ac97eb03b4bcdc71361fee02ad46f`. CLI test builds replace the executable with a test-feature-linked artifact. The harness now restores the production build and identifies it **after every probe**, retaining the initial build command additively. The eleventh Python test pins call ordering and denies report publication if the final build fails.

Kimi reviewed this concrete, two-file harness correction from `/tmp/corbanu-pf13-kimi-identity.Ejx0wY` at 06:28:00–06:29:08 UTC: **no findings** (`kimi-artifact-identity-review.txt` / `.json`). No Rust source changed; the prior runtime review stands. Updated complete source manifest: `repair-final-source-files.sha256`. Repaired canary reports must match the binary actually left on disk before final TUI qualification.

The historical complete-Core report remains failing (135 failures) and is not relabeled. A fresh full run must distinguish missing companion binaries from actual remaining defects; unrelated runtime changes require scope review, not weakened assertions.

Linux prerequisites were prepared on the authorized host: isolated Rust 1.95 toolchain, `just` 1.51.0, nextest 0.9.143, and `libcap-dev` using authorized elevation. No password is stored in evidence. A Git bundle transferred the candidate to a fresh, clean detached checkout at `/home/travis/corbanu-pf13-repair.XY7q1n/source`. The prior qualification checkout was not edited. Build uses eight Cargo jobs, no incremental artifacts, and debug symbols disabled. Final commit-bound run pending.

Windows reachability check from this Mac: `ssh-keyscan -T 5 100.111.98.12` returned exit 1 and no host key. No authenticated session or host-fingerprint verification was possible; no production private key was copied. This is blocked final-tree Windows evidence, not a Windows pass or an established cause of the connectivity failure.

Windows final-tree proof, live-repository release flows, Travis's human acceptance and any due benchmark remain release gates. Neither this repair record nor the component review certifies PF-23 native integration or a shippable release.
