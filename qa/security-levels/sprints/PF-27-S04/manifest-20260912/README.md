# PF27 fixed manifest and image inspection — construction qualified

Product initiative PF27-S04. Exact accepted contract, owner coordinates, frozen
negative cases and limits: [manifest allocation](../manifest-next-20260912.md).
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution boundary.”

Review baseline `919b1652d`; qualified source `099fa6f6dacd7783afbd98f1493b491015bc84af`; Rust tree
`5df6837f11abe2b176c18e1e69a53ef27b595194`. Nine source/build files,642 changed
lines:640 additions/2 deletions, including312 separate test lines. Production
modules130/114/72 lines. Above500 target to preserve negative cases, below800
hard ceiling. No Core/PF20/Vault/normal-startup changes. Existing optional
serde/serde_json/sha2 and rustix fs only; Cargo lock gains three service edges,
MODULE.bazel.lock unchanged after parity. Keep source/evidence commits separate.

Linux synthetic-only public `SyntheticManifestInspection::inspect_system()`
first requires all root UID/GID triplets. It opens only the fixed manifest and
derived pinned image via held directory handles with safe openat2; checks
ownership, modes, types, links, sizes, strict schema/role table and streaming hash
plus ELF64 LE x86-64 identity. Read baseline metadata is the same snapshot whose
permissions were checked, compared after reads. Result has no command factory,
public descriptor/path getters or Clone. No execution, native-ready bit or
normal startup wiring. Public production constructor has no owner/path override.
Private same-user fixture seam cannot be used by library consumers.

Review the actual parser/duplicate-field rejection, role-table reuse, bounded
stream reads, file/ancestor ownership checks, sanitized failures and retained
descriptor limitations. A held handle does not make image bytes immutable;
future exec binding and loader/shared-library trust remain unimplemented gates.
Byte limits are not storage-stall deadlines. Root/kernel are trusted; no malicious
administrator or whole-machine rollback claim. Do not perform root-positive
tests, touch fixed root directories or request extra/nested reviewers.

## Test status and honest limitations

RTX worktree `/home/travis/worktrees/security-broker-manifest-20260912`, same
source copied after formatter. Evidence root
`/home/travis/security-round5/evidence/pf27-manifest-20260912/`.
Initial run: default3 passed; fixture32/35 passed,3 failed. Actual host umask0002
made temporary fixture root group-writable; positive control correctly denied.
The socket fixture exceeded SUN_LEN. Repairs set only the disposable fixture
root0700 and bind a short socket then rename it into the test position. No
production permission relaxation. Original logs retained under initial/.
Before reviews, also bound the read snapshot to the same checked metadata.

Post-fix/format default3 and synthetic35 pass, scoped strict Clippy default and
synthetic pass, Cargo/Bazel parity passes. Full affected356 passed with2 existing
PF20 subprocess-helper skips; build and actual-key TMUX35/denial/inspection pass.
These were pending at first Astra dispatch and completed before Fable closeout.
No stale first-run pass claim. Published logs strip trailing whitespace only;
raw logs remain on RTX. Ignored logs are explicitly tracked for review.
Candidate probe SHA256:
`564eaeebd58bfbb598facb320d1288bf03ebca5f9f9ad38d21657115abe0eb10`;
all binary digests in rtx/binaries.sha256. Local/RTX Rust tree matches exactly.

Construction tests include strict nested required/unknown/duplicate fields,
invalid version/purpose/commit/hash/IDs and role collisions; valid temporary
tree, corruption/length/hardlink/oversize mutations; symlink/mode/missing/type
at every tree component; both leaf FIFO/socket cases; foreign-owner/GID and
device checks; all stamp fields and actual mutation; bounded short/invalid
ELF/header/body read failures; actual non-root fixed-factory denial. Fixtures
are not root-installed containment proof. Permission mutations touch owned
temporary files only; no accounts, install, chmod/chown of system paths or exec.

Full inherited transitive telemetry lint remains manager-owned; scoped lint is
not its resolution. Supporting actual-key TMUX exercises normal deny78, probe
inspection/native deny/preparation nonroot deny and the full synthetic suite.
No new product TUI or live-repository benchmark applies to this uncalled
internal stage; later integration still requires them. Native/protected
eligibility remains false and PF27 remains in_progress.

## Review and integration

Manager authorized two new material-stage passes (Astra High and Fable5.1High
via Corbanu/private TMUX), separately from the retained old contingency.
Reviews16/17 were reserved before dispatch. Both completed with exit0/findings[]:
[Astra16](astra-sixteen.json), [Fable17](fable-seventeen.json). Original JSON,
text and exit records are committed; no source changed after either review.
Two new passes consumed, old contingency retained. Scripts/runtime logs remain
under `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf27-manifest-review-20260912/`.
Do not repeat clean reviews. Main window must be requested after branch handoff.
