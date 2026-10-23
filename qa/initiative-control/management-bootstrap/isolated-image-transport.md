# PF-80 bounded inline PNG transport prerequisite — September 13, 2026

Parent-reviewed internal implementation; no independent functional execution or
product acceptance claim. Exactly three owned files: this record,
`scripts/initiative_control/isolated_model_transport.py`, and its adjacent
`test_isolated_model_transport.py`. No catalog or other source edits.

Authority: parent `management-workstreams-20260911`, canonical commit
`d87400753711d262c64e5339452690f8644e6981`, research allocation
`docs/research/tasknode-integration/coordinator-bootstrap-20260913.md`, heading
**Isolated browser prerequisite allocation — September13 receiving8665ff1c9**,
and matching `docs/plans/active/initiative-delivery-control.md` coordinates.
The worker read the parent updates before commitment and verified the committed
coordinates afterward; that commit was not merged into the frozen worker base.

Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/isolated-image-transport-20260913`.
Branch: `bootstrap/isolated-image-transport-20260913`.
Base/unchanged HEAD: `8665ff1c962ff2a6aa6a3157eb2251942d438c3f`.
Classification: bounded internal bootstrap allocation within existing active
initiative PF-80 and `PF-80-S01` (`in_progress`), not new product work.
Product heading **Internal delivery control — TO BUILD**: “Use sequential sprints
per initiative”; September 13 excerpt: “authorizes the bootstrap and bounded live
qualification, not premature product sprint resumption”. Full development skill
and root policy were read; no nested implementation policy exists for these paths.
Parent plan/sprint checkers passed: active 3/3; current 115, archived 126.

## Contract and native observation boundary

`execute(text, *, image_png=None)` accepts one exact immutable `bytes` object.
OFF returns before inspecting either input, paths, credentials or processes.
Enabled input validation precedes attempt creation and any credential access.
Limits: 4 MiB encoded bytes, 1–4096 pixels per side, 4,194,304 pixels total,
and at most 1024 chunks. Supported envelope: PNG signature, one 13-byte IHDR,
8-bit RGB/RGBA, compression/filter method 0, no interlace, one or more consecutive
IDAT chunks with nonempty aggregate payload, then empty terminal IEND. Check all
chunk bounds, CRCs and ordering; reject trailing bytes, all ancillary chunks,
palette/grayscale formats, APNG and other unsupported formats. No image paths,
URLs, mutable buffers, image editing, fetching or guest configuration are accepted.

After review01, the parent added bounded IDAT inflation: at most the declared
scanline bytes plus one, exact length and terminal zlib stream, no trailing or
concatenated compressed stream, and filter bytes0..4. For the accepted8-bit RGB/RGBA
profile this rejects invalid compressed pixels before auth or native execution.
Maximum output is16MiB plus row-filter bytes plus one; no unbounded flush occurs.
It does not transform images or certify every native decoder/OS behavior. No new
dependency was added; struct, zlib and CRC32 are standard-library operations.

Validated bytes are saved once as `attempt-*/observation.png` using existing
O_EXCL creation at 0600 inside a fresh private 0700 attempt. No overwrite or alias
is accepted. The native argv suffix is exactly
`--image <absolute-attempt-path>/observation.png -- -`; the last argument preserves
stdin text. Commas in the owner run root are rejected for image attempts because
the supported CLI splits image values on commas. Text-only behavior is unchanged.
The only added Seatbelt rule is a literal read allowance for that exact image file;
no directory or image-write allowance is added. All model/tool/network/auth flags,
Astra High, catalog pin, zero retries, 90-second wall and 256-KiB output cap remain.

The image's SHA256 and existing device/inode/mode/uid/link-count identity are pinned
after writing, checked immediately before launch, and checked after process return.
Replacement, symlinks, hardlinks, permission/content changes and deletion fail the
attempt. This is trusted-host code: before/after snapshots do not prevent a hostile
owner process from changing and restoring a file between checks. It is not hostile
Python isolation. `image.json` retains the original identity, digest, dimensions
and byte count; the safe receipt includes image metadata but no image bytes/path.
Failures retain existing receipt/failure/raw diagnostics and available image files.
Invalid input raises before an attempt exists, matching text validation behavior.
The new `image_observation=supplied_not_attested` field is deliberate: image
metadata identifies bytes supplied, not proof that the model received or understood
them. The pinned CLI can replace native read/decode failures with text placeholders
and continue a successful text turn. Only task-specific observed execution can
establish visual acceptance; status=ok is not an image-delivery acknowledgment.

## Original worker evidence (before review correction)

Python 3.14.4, in the exact worker worktree:

- `python3 -B scripts/initiative_control/test_isolated_model_transport.py`:
  25 tests, 0 failures/errors, 3.122s.
- `python3 -B -O scripts/initiative_control/test_isolated_model_transport.py`:
  25 tests, 0 failures/errors, 2.691s.
- Normal retained root:
  `/private/var/folders/3p/j2_d1_f57vz5d0tj3ycvjg9w0000gn/T/isolated-transport-tests-mxb1ccmw`.
- Optimized retained root:
  `/private/var/folders/3p/j2_d1_f57vz5d0tj3ycvjg9w0000gn/T/isolated-transport-tests-7kfubuoa`.

Both roots contain `suite.txt`, `summary.json`, and private per-test attempts,
including deliberately failed mutation, malformed-event, timeout and collision
cases. No test run failed. Six new test methods cover image wiring/bounds/refusal;
the existing 19 remain. Fixtures traverse public execute and actual bounded child
process I/O. They record real-builder argv/policy in `synthetic-native-launch.json`
and have the synthetic child read the image path from that argv into
`observed-image.raw`. The fixture replaces the native process and does not enforce
Seatbelt; exact policy-text comparison is not OS enforcement proof.

Verified native binary SHA256:
`4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`, at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-final9-20260910/bin/corbanu`.
Only native invocation: `exec --help` from `/private/tmp`, via `env -i` with
PATH `/usr/bin:/bin` and HOME/CORBANU_HOME/CODEX_HOME `/nonexistent`. Exit 0;
help says `-i, --image <FILE>...` and confirms `-` reads stdin. Retained diagnostic:
PATH aliases could not be created because CORBANU_HOME `/nonexistent` does not
exist. Shared CLI source confirms the comma delimiter and variadic image option.
No actual native image request, real auth read or model call was performed.

Worker diff: +325/-15, 340 changed lines including 196 non-test; below target400/200
and hard650/350. `git diff --check` passed; the new evidence file is untracked.
Module SHA256: `c073454679d3da3c0d5c43d81e24188bec4f3d96015268f7a48aa26ee5ccc177`.
Tests SHA256: `6376e964b3eab50223e920c8fc90f7341c11a3be1893aafab2c34f8f268b63f1`.

## Parent review, recovery and correction

Fable High review01 session `01a09a99-ae21-74e0-a04b-6cedf069365f` resumed after
the external-volume stall and completed with one P2: native image failures can
silently become text. Parent verified `protocol/src/models.rs` placeholder paths,
accepted the issue, added the supplied-not-attested field and bounded pixel checks.
Original three files remain under `.codex-work/image-transport-review-6prpAT/original/`;
review01 JSON/text and all old test/native artifacts are retained. This is the
same in-scope correction, not a new decoder framework or reset review budget.

The first parent replay `isolated-transport-tests-lx4j5nis` was interrupted with
SIGTERM143 during the volume stall; it is not a pass. After recovery, original
candidate replay `isolated-transport-tests-p1tptv27` passed25 tests/2.959s.
Corrected candidate:26 normal tests/2.919s at `isolated-transport-tests-j4byjxy1`;
26 optimized/2.994s at `isolated-transport-tests-kujccich`, same parent temp prefix
as the worker roots above. New negatives cover bad/truncated/concatenated DEFLATE,
wrong scanline size, expansion beyond declared dimensions, invalid filter bytes,
and no auth/files/process access; valid split IDAT and filters0..4 pass.
Module SHA256 `c0610c187b3aa5e8096ddc8090126ee66d2de694bc935da536acc7db5191d929`;
tests SHA256 `39a61c9d7bc98f6b598c81890629a56aa5d250040c0dfcf4c014b18c1f3962a8`.
Parent accepts the small target overrun within the unchanged650/350 hard scope.

Original candidate native proof: `.codex-work/image-native-YGnIIE/positive.json`,
session `01a09bf2-e1f7-71e0-aea0-0a0f9accf6f7`, Astra High correctly read the
synthetic browser screenshot's input without that phrase in the prompt. Actual
14.328s, exit0, owned group gone, inputs unchanged. Image SHA256
`47f81084ad221b27fca309d9b22579e74881c167972b591c9b321f32aaf30f1c`.
The separate invalid-pixel attempt `attempt-z8x_heut` failed after71.712s due to a
provider stream/request failure, not verified decoder refusal; no fallback/retry.
It neither proves nor disproves the placeholder behavior. Corrected code now
rejects those exact malformed bytes before inference; native read-denial outcome
and final-candidate visual proof remain separate owner qualification below.

## Final parent qualification and disposition

Fable corrective review02, session `01a09bf7-72f7-74c3-a485-737acb060a1d`, completed
exit0 with no findings. No unchanged-code repeat review requested. Corrected native
positive `attempt-jvhznsix` in the same image-native root returned the actual input
text in9.705s, session `01a09bf8-0fb5-73f1-9afd-7fcf5598e5a1`, exit0/group gone and
inputs unchanged. The prompt did not contain that text.

Actual negative `attempt-gd0lvdl_` omitted only the image-file read grant from the
otherwise same guard. The model returned `NO_IMAGE`, session
`01a09bf9-9458-7430-b37a-b194e263d0ed`,10.333s, status=ok/exit0, group gone and inputs
unchanged. This confirms the native silent-placeholder risk and why the new
supplied-not-attested marker must not be treated as visual acceptance. The altered
negative policy is recorded in `read-denied.json`; it is not the production guard.
The two native modes use the final module hash above, with no changes to real auth.

Parent accepts this internal transport increment and its reasoned functional N/A:
it exposes no user workflow or operational agent tools. Actual code-blind browser
actions, child/network/source-denial probes, candidate cases and independent
evidence review remain required at the combined execution gate. Receiving tests
and exact merge receipt are recorded separately by the single integration writer.

## Remaining combined gates

Parent owns receiving integration/tests and the combined isolated action loop,
full boundary qualification and independent visual acceptance. Existing reviews stay
retained. No descendant workers, Docker, services, policy/state/scheduler writes or push.
True-TUI, TensorCash/Isometric Game, benchmarks and named-human acceptance were not
run: this changes an internal library only. Internal-only functional N/A is offered
for integrator acceptance; the later isolated action loop and combined visual
workflow still require independent code-blind execution/evidence. No product,
browser, release, recurring-runtime or PF-80-S01 completion is established here.
