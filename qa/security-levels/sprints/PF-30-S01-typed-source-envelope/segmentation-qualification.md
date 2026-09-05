# Bounded complete-input segmentation checkpoint

Allocation merged without rebase: `db141e9cb` → local merge `72f5de657`.
Source checkpoint: `078342d85`; synchronized remote formatting and audited ledger:
`2a4fb5857`. No changes to transferred `core/src/client.rs` or
`core/src/session/mod.rs`, shared exports, manifests or locks.

## Implemented boundary

The existing private native screening candidate exposes deterministic chunks of
at most 512 bytes, their count and the unchanged complete normalized input/source
binding. Normalization precedes byte segmentation. A Unicode escape may cross a
chunk boundary; no individual chunk can authorize admission. The existing
content-security reassembler validates source identity, indices/count, complete
content digest and a verdict for the whole target. Core additionally requires
the expected segment count before atomically consuming the pending candidate.

Whole-input raw/normalized limit remains 2,048 bytes; projection limit remains
8,192 bytes plus the fixed context wrapper. Oversized or empty inputs reject,
never clip. This is complete segmentation **within that existing bounded input
contract**, not support for arbitrary-length inputs. Provider projections and
Permissive formatting have not changed. The projection can still exceed 1,000
tokens and requires explicit manual review under Core policy; the limit was not
increased by this continuation.

Five new native tests cover:

- Unicode escape split across chunks, complete reassembly and stable replay.
- Missing/duplicate/cross-source/wrong-count chunks and altered content order.
- Complete screened bytes from a different segmentation contract rejecting and
  consuming the pending candidate without restoring raw input.
- Exact pre-segmentation context JSON shape and empty/oversized rejection.
- One admitted source never covering an absent or newly introduced native kind.

Out-of-order arrival with authentic indices is valid and reassembles in original
index order. Swapping the payloads at those indices changes the complete digest
and fails. Existing real Responses/Chat/Anthropic tests now exercise segmented
input and compare repeated serialized requests byte-for-byte. The actual
Permissive network test retains a >2,048-byte message verbatim, proving protected
input bounds were not accidentally applied to Permissive.

## Final RTX evidence

All commands ran on the allocated RTX host under the shared build lock and a
fresh `TMPDIR` below `/home/travis/security-round5/provenance-tmp`, never in the
polluted shared temporary directory. No local compilation.

```sh
just fix -p codex-core
just fmt
just test -p codex-core -E 'test(pf_30_s01)' --retries 0 --test-threads 4
just test -p codex-content-security --retries 0 --test-threads 4
cargo build --locked -p codex-cli --bin codex
just fmt-check
```

Results: Core provenance **27/27**, full content-security **22/22**, locked CLI
build and formatter check passed. The immutable binary was copied under the
lock to `candidate-2a4fb5857/codex` before releasing the shared target directory.

With `CARGO_BIN_EXE_codex` pointing to that immutable binary,
`CORBANU_TMUX_REQUIRED=1`, and the same isolated environment, the real-key test
`tmux_smoke_single_enter_dispatches_slash_command_and_exits_cleanly` passed 1/1.
It enters `/status`, presses Enter, observes its response, then enters `/exit`
and presses Enter. Passing pane screenshots are not retained by this harness;
the executable test result is recorded. This is supporting TUI evidence, not
human acceptance or all-feature release qualification.

Plan/sprint checks passed: two active plans, 58 current and 114 archived sprints.
Full Core and protocol historical evidence remains in `qualification.md`; the
known five baseline Core failures were not recast as a clean full-suite result.

Artifacts under `/home/travis/security-round5/evidence/provenance`:

| Artifact | SHA-256 |
| --- | --- |
| `segmentation-proof-1.log` | `bb14fb70c1c6e8a05aa863b72d8e2ab3fdf742c4321078303ee9bfe48d0f9df3` |
| `segmentation-final-tmux.log` | `9a4f9ea6c5e08356d25ce883c73c88035f3fd71c73e6f4d5996170b737f60e5d` |
| `candidate-2a4fb5857/codex` | `f131a4b8c8f0d1e63e9e233f7ee132c16e223c3c730c3d7b96c67bbf94bd5bc3` |

## Remaining and review state

PF-30-S01 remains `in_progress`. Production screening capability delivery is
absent: `codex-rs/content-security/src/` currently supplies a contract and
reassembler, not a production classifier. Positive fixtures do not supply one.
Finer source identity and hosted/opaque ingress coverage still require observed
producer integrations; generic tool/transcript origins remain conservative.
No classifier Allow is fabricated and no PF-35 readiness is inferred.

The memory-dispatch follow-up is separately owned by PF-30-S04. Earlier Fable
findings and disposition remain visible. The review ledger is still three of
five used; review 4 has been requested for this consolidated delta, not started
without coordinator authorization. No overall clean review or release claim.
