# PF27 sealed-byte format inspection — qualification in progress

Accepted contract and original cases: [allocation](../runtime-elf-next-20260912.md).
PF-27-S04 remains in progress, same sole owner/worktree/original d870 base;
incremental baseline97d. This is internal Linux synthetic construction, not a
human-test candidate or runtime execution authority. Product heading
**Non-negotiable controls**: “Permit agents to reference credentials only by
label; resolve them solely inside the trusted execution boundary.”

The consuming method returns a non-Clone opaque profile-inspected result holding
the same sealed memfd and recipe. Public callers cannot construct/extract it.
Checks use bounded seeks/reads, at most128 program headers/4096 dynamic entries,
checked file/virtual ranges and unambiguous dynamic backing. Frozen vocabulary
and zero-only DT_NULL padding policy are in the allocation; unknowns deny.
Sections/relocations/symbol targets are not runtime-qualified by this parser.
Neither successful parsing nor build feasibility proves safe execution, runtime
file/dlopen behavior, dependency CVE suitability or native containment.

Tests preserve identity/header/table/dynamic mutation cases, safe I/O errors,
seal denial, held-inode/drop cleanup and pathname replacement after sealing.
Three frozen artifacts are opened as data only via the explicitly invoked ignored
artifact test; ordinary suite skips are not represented as that test passing.
Default/synthetic/affected suites and the explicit artifact test run on RTX after
fix/format; private TMUX records separate text/Enter and completion evidence.
No probe, ldd, privileged installation, launcher/default or graph changes.

Initial remote setup used a fetch that did not create an origin tracking ref;
worktree creation failed128 before builds. Retrying with fetched exacte2725639a
created the intended detached worktree. Initial fix removed one unused re-export;
formatting only was synchronized back. Preserve initial outputs; the subsequent
pathname-replacement test update requires final-tree rerun before closeout.
Reviews are limited to the two manager-authorized new-parser passes. No UX,
code-blind designer or live-repository benchmark applies to this uncalled API.
