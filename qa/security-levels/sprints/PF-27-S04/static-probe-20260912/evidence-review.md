# Single authorized build/linkage evidence check

Review only this accepted build-only allocation over main140e094ad. Do not
repeat Rust code review or start nested reviewers, tests, builds or target images.
Rust subtree0aa65bd04f5e30f3a21e1309aac7aa6b83336859 is unchanged; the new work is
QA tooling, isolated prerequisites and exact artifact qualification evidence.

Read README.md and the accepted static-probe-next-20260912.md. Inspect the six
small QA scripts, final linkage-retry source/lock/build/linker/fingerprint/readelf
and linkage.json, and linkage-control keys/capture/result. Check that exact keys
invoke build/inspection tools rather than either probe. Verify preserved initial
101, OpenSSL2 and rejected interpreter-bearing build, and signed prerequisite
metadata/hash records. Large compiler logs are mechanical; inspect targeted
diagnostics rather than treating the raw diff as new product logic.

Determine whether the final evidence proves only the claimed bounded build/linkage
properties: ELF64/LE/x86-64, static PIE, no interpreter or external DT_NEEDED,
bounded size, fresh digest/provenance, correct negative GNU control and unchanged
source/locks. Check no target execution, global/root changes, old artifact reuse
or suppressed failure is inferred from a successful build. Runtime security,
libc advisories, privileged installation and release suitability remain open.

Report concrete mistakes or unsupported claims within this evidence boundary.
Do not demand a Rust redesign or extra runtime qualification for a build-only
feasibility record. No new design pass or additional review is authorized.
