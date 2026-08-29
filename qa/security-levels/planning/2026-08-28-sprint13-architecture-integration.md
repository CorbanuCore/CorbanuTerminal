# Sprint 13 architecture integration evidence

Date: 2026-08-28. Integrator: Codex. Branch:
`codex/security-architecture-review-20260828`.

## Candidate identity

- Integration commit: `dfbd4108d` (`Merge Sprint 13 security implementation`).
- First parent: `f81c45d9917e7d871ab27ca2085a97e0914fdee3` (security architecture review).
- Second parent: `c25e2825a2fe3fe63c8e1d58cc1e3aa82b0d1d04` (published Sprint 13 implementation).
- The merge retains PF-13-S05 as the in-progress component-qualification sprint and adds PF-13-S07 as the later composed credential-boundary integration gate. Completed PF-13-S02 through S04, PF-26-S01 and PF-27-S01 records are archived. The browser-isolation implementation is treated as input to the canonical PF-31/PF-33 graph, not as a duplicate completed sprint.

## Integration repair

The first structured Codex autoreview found one P1: image preparation trusted a
predictable local recipe tag plus forgeable metadata as a browser-worker cache
hit. The finding was accepted. Image preparation now always rebuilds from the
checked-in Dockerfile/worker and pinned Scrapling base with `--no-cache`, takes
the immutable image ID returned by that exact build, inspects that ID, and
requires the inspected ID, recipe label, user and entrypoint to match before
use. A preseeded tag is not consulted as trust evidence. Fixture coverage
includes Docker and Podman ID forms, malformed build output, recipe mismatch,
and a build-ID/inspect-ID mismatch.

Final autoreview command:

```text
python3 /Users/travisgood/.codex/skills/autoreview/scripts/autoreview \
  --mode local --stream-engine-output --prompt '<focused integration re-review>'
```

Result: **clean; no accepted/actionable findings**. The reviewer specifically
confirmed that the prior image-cache P1 was closed. Review output is advisory;
the finding and repair were independently checked against `image.rs`, the
engine command boundary, container ownership verification and fixture tests.

## Final automated results

Rust commands used the repository's Rust 1.95 toolchain. The complete Core and
TUI packages used the documented macOS large-binary recipe:
`ld64.lld`, `CARGO_INCREMENTAL=0`, and debug symbols disabled. This changes no
test selection or assertion. Core companion binaries (`codex`, `corbanu`,
`codex-code-mode-host`, `test_stdio_server`) were built first.

| Check | Result |
| --- | --- |
| Plan checker and plan-checker unit tests | pass; one active plan, 4 tests |
| Sprint checker and sprint-checker unit tests | pass; 69 current / 86 archived, 19 tests |
| Strict MkDocs build | pass |
| Portable skill mirror | pass; 25 files |
| Security harness Python tests | pass; 50 tests |
| `just test -p codex-security-policy` | pass; 39/39 |
| `just test -p codex-vault --test-threads 1` | pass; 33/33. An earlier parallel run caused nine macOS keychain timeouts and two flaky results; serialized rerun was clean. |
| `just test -p codex-network-proxy` | pass; 223/223 |
| `just test -p codex-protocol` | pass; 281/281 |
| `just test -p codex-browser-isolation` | pass; 20/20 after the cache-trust repair |
| Browser worker Python tests | pass; 6/6 |
| Complete `just test -p codex-core --test-threads 4` | **gate remains failed**: 3,407 executed, 3,389 passed, 18 failed, 19 skipped |
| Complete `just test -p codex-tui` | **gate remains failed**: 3,840 executed, 3,799 passed, 41 failed, 7 skipped |
| `just bazel-lock-check` | pass; existing direct-dependency version warnings only |
| Final `just fix -p codex-browser-isolation`, `just fmt`, scoped whitespace check | pass |

The Core result improves the prior 19-failure record by one: the previously
intermittent prompt-cache transition passed. The remaining 18 names are the
same already-triaged Bash 3.2 compatibility, agent lifecycle/authority,
MCP-authority refresh, stale denial wording, tool registration/approval,
prompt/serialization and shell-parallelism groups. No credential- or
browser-boundary test failed.

The TUI failures are 37 stale product/version/status snapshots and four macOS
`/private/var` versus `/var` path-normalization assertions. The test runner's
37 generated `.snap.new` files were removed after recording the result; no
snapshot was accepted or relabeled. These failures remain follow-up work and
are not represented as a clean package gate.

## Live-engine and release limits

The final Docker smoke failed closed with `ResourceLimit`. Direct `docker info`
and pinned-base `image inspect` also timed out, showing that the shared Docker
Desktop daemon was stalled before image preparation could be qualified. The
daemon was not restarted during integration because it hosts the existing
Ambient containers. This is blocked live-engine evidence, not a browser pass.

This record does not complete PF-13-S05 or PF-13-S07 and does not replace the
required Windows qualification, true-TUI/live-repository proof, Travis human
acceptance, or release evidence. It establishes that the two branches are
actually integrated, structurally consistent, reviewed, and retested with the
remaining gates stated explicitly.
