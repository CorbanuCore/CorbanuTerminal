# PF27 combined receiving proof — c51d2d6aa

PASS for the allocated private preflight receiving gate only. No native or
whole-PF27 completion claim. Manager assigned this exact combined-tree proof;
no source, dependency, canonical-manager or old-evidence changes were made.

## Exact candidate and invocation

- Receiving commit: `c51d2d6aa76da780868bcf83c2738a6c0149a73c`.
- Rust tree: `dc3ce10ff45c20c371db67b7b75e2b2317b31165`.
- Cargo.lock blob: `ded15fd917dc517b933c0a637821d28f9e3300ba`.
- MODULE.bazel.lock blob: `5ecff077dbbbf7887a61c03a8e9aa354f002e1ed`.
- Detached clean RTX checkout: `/home/travis/worktrees/security-broker-preflight-receiving-c51d2d6aa`.
- Remote evidence: `/home/travis/security-round5/evidence/pf27-preflight-receiving-c51d2d6aa`.
- TMUX socket/session/pane: `pf27preflightreceivingc51d2d6aa:proof:0.0`.

Actual command text sent using `tmux send-keys -l`, then a separate Enter:

```sh
bash /home/travis/security-round5/evidence/pf27-preflight-receiving-c51d2d6aa/run-pf27-preflight-receiving-c51d2d6aa.sh
```

The [verbatim driver](rtx/run-pf27-preflight-receiving-c51d2d6aa.sh) freezes
candidate/inputs and executes the unchanged tracked
`qa/security-levels/sprints/PF-27-S04/system-preflight-20260913/qualify-rtx.sh`,
which retains all29 predecessor commands and adds the explicit preflight filter.
Start **2026-09-13 02:34:24UTC**, end **02:36:45UTC**, elapsed **141 seconds**.
Actual non-root uid1001, GNU2.43/Rust1.95.0, ext-family TMPDIR. Existing exclusive
RTX build lock and receiving target were reused; the Mac target was untouched.

## Literal results from this run

All **30 command exit files are0**, plus [suite.exit](rtx/suite.exit) is0.
Each original command log/exit is in [final-tmux/](rtx/final-tmux/), with the
complete [suite log](rtx/suite.log). No failed test, retry or source repair in this
run. Prior unit failures and reviews remain preserved in their original receipts.

| Commands / cases | Actual result |
| --- | --- |
| adapter / adapter-os | 4 /10 pass |
| default / service | 3 /64 pass; service24 exclusions covered by explicit retained filters |
| pair / real-pair | 5 /1 pass |
| owner / real-owner | 8 /3 pass |
| admission | 8 pass |
| protected-default / protected-feature | 18 /18 pass |
| root-compat / root-dispatch / root-session | 4 /6 /7 pass |
| system-preflight | 5 pass,0.019s |
| profile-hold / profile-inspect / connector-profile / relay-profile / dispatch-relay-profile | Each1 pass |
| fixture-build / relay-build / dispatch-relay-build | Each exit0 |
| strict-clippy / compatibility-clippy | Exit0,15.12s /14.39s |
| bazel-parity | Exit0; existing resolved-version warnings retained |
| unchanged-locks / unchanged-source / compatibility-locks / compatibility-source | Each exit0 |

Focused filters overlap full-suite counts, not extra unique tests. Both markers
`PF27_SYSTEM_PREFLIGHT_TMUX_COMPLETE` and
`PF27_PREFLIGHT_RECEIVING_C51D2D6AA_COMPLETE` are present in the
[actual pane capture](rtx/final-tmux-capture.txt).

## Invariance and limits

[Before](rtx/provenance-before.txt) and [after](rtx/provenance-after.txt) retain
the exact commit/Rust/lock blobs and identical five fixture hashes. Module hash
files compare equal for Cargo.toml, Cargo.lock, MODULE.bazel and MODULE.bazel.lock.
The newer accounting Cargo.lock SHA256
`143c2663872d8857995408a192fcd1a733ae355789cf6fc9132d202485a99f29` is preserved;
MODULE lock SHA256 is
`c8d7e3f8c8bec8f8e71cc3d1d39fcb952eec0f07a41cdac48401a6f64a60d979`.
The checkout remained clean. The first local raw-SHA bundle command refused an
empty bundle before transfer; bundling the verified named branch succeeded.
This packaging correction did not run or repeat any qualification command.

Product initiative PF-27-S04 remains in_progress under p0-security-levels.
Citation: **Non-negotiable controls** — credentials are referenced only by label
and resolved solely inside the trusted execution boundary. This is the previously
accepted policy1.7 internal-only N/A: no new human-facing workflow or activation.
No new code or opinion review was requested/performed; Astra39/Fable40 originals
in the owner preflight receipt remain unchanged. Distinct-principal native
bootstrap, protected-user/PF26 isolated functional execution, all-OS and live
repository qualification, human acceptance and release gates remain outstanding.
No live credentials, root-positive execution, installation, successor code or
main/push actions were performed. The separate private next-step proposal is not
implementation authority and is not part of this evidence commit.
