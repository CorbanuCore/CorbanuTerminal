# PF27 private descriptor-pair root dispatch

Final exact-source RTX proof passes; allocated independent reviews pending.
Allocation23d8e54a1 imported at launch00b07dad10b215e756696f8c5477055f1e1722d5;
timestamp correction imported1cfdce29d. Source/size base12e0bc2d8 unchanged.
Fifteen exact allocated paths, target740/hard800 including receipt/tests/runner.

Private dispatch binds the admitted role/generation and same pair-control identity
to its fixed root. The matched OwnedChild mints retained identity; no raw PID/fd
authority or mock identity enters PF20. At most two handlers; the existing worker
alone owns termination/reaping. Receipt drop fences both channels on every return
or panic; start/open failure cancels. Completion requires handler joins and owner
cleanup. Typed PF20 errors remain distinct, including Ambiguous.

The Linux/GNU synthetic-only bridge creates fresh private temporary roots with
fixed synthetic bindings. It accepts no existing path/root/key/UID and calls the
actual storage, transport and client; stale sequence injection reuses actual HMAC
framing. Existing fixed-system factories, public root.rs/Child and default builds
are unchanged. Caller retains fixture lifetime; returned roots are fixture data,
not native authority. No installation, protected activation, credentials or UI.

## Evidence and original attempts

Remote originals: `/home/travis/security-round5/evidence/pf27-root-dispatch-20260912`.
Initial compile101: test referred to a nonexistent IntegrityRootError::Ambiguous;
PF41 maps that outcome to Timeout. Lost-reply test now exercises policy's actual
RootError::Ambiguous, without changing runtime mappings. Repaired compilation0.
First actual six-case run:5pass/1fail; namespace assertion expected client Invalid,
but existing protocol closes without an error frame. Corrected client expectation
to Unavailable and asserts exact server Invalid; added reverse namespace case.
Distinct repaired run:all6pass. Original logs/exits remain; no hidden retries.
Quoted remote scp brace failed before transfer; explicit source arguments worked.

Cases: reversed role arrival, both namespace load/CAS/reload, duplicate socket
receives no handshake, both wrong namespaces, malformed and authenticated replay,
committed withheld policy reply remains Ambiguous and consumes client, actual
death/cancel/deadline/drop closes both, wrong parent peer and old receipt against
same-number replacement generation reject without a key, open/start/panic failure
cleanup. Real OS/protocol happy paths; only construction/panic faults use seams.

## Final proof and review

Frozen source `c7d48e4822293ec0899311be63fbd08848516ba0`, Rust tree
`bd56b597e348d2924da21706c594c20a868bace4`. Exact clean RTX checkout
`/home/travis/worktrees/security-broker-root-dispatch-20260912` ran once through
private TMUX `pf27rootdispatch20260912:proof:0.0`, September12 23:01:54–23:03:38UTC
(104s). Command text and Enter were sent separately. All28 command exits and
suite exit0; original `rtx/final-c7.sh`, logs/exits under `rtx/final-tmux/`,
`rtx/suite.log`, before/after provenance and `rtx/final-tmux-capture.txt` retained.
Both `PF27_ROOT_DISPATCH_TMUX_COMPLETE` and wrapper completion appear in capture.

| Final checks | Actual result |
| --- | --- |
| Adapter default / actual supported OS | 4 / 10 pass |
| Service default / pair / actual pair | 3 / 5 / 1 pass |
| Owner / actual owner / admission | 8 / 3 / 8 pass; admission covers ten OS scenarios |
| Full service feature suite | 59 pass; 17 ignored cases separately exercised |
| PF20 default / feature / actual compatibility | 18 / 18 / 4 pass |
| New actual dispatch cases | 6 pass, including lifecycle and protocol failures |
| Hold / inspect / connector / both relay profiles | Five distinct artifacts pass sealed ELF profile |
| Strict service+adapter / PF20+adapter Clippy | Exit0, 6.40s / 14.21s |
| Derived Bazel update / final read-only parity | Exit0; no lock drift |

Filtered suites overlap; counts are not summed as distinct tests. Service17
ignored cases are the prior11 plus new6; PF20 helper-only exclusions remain.
GNU2.43 non-root uid1001, Rust1.95.0, ext-family TMPDIR, shared build lock/jobs4.
`just fix`, `just fmt`, and actual `just bazel-lock-update` all completed before
the final tests. Source and Cargo/MODULE files matched before/after; no new lock
delta (tempfile was already a dependency edge). Cargo.lock SHA256
`5101ffe0dea88d84e213fcd31025f1d2ca0f9a8cbd069b8ac5ea787d392b1459`;
MODULE.bazel.lock `c8d7e3f8c8bec8f8e71cc3d1d39fcb952eec0f07a41cdac48401a6f64a60d979`.
Relay source `5cbe518e919f36766347df29cd958c9fef71846b614fc781d48557b5745cd08c`,
artifact `36f1fa188f3e95faa5643ec20e6640504ce44a2cfb3093e7aa0f205ec3a32867`.

Initial WIPa1e0ff954 omitted remote Rustfmt's export ordering. Full staged-source
comparison and wrapper guard stopped before tests; original failed preflight
capture retained. Formatting-only c7 copied back, checkpointed and matched the
entire remote staged tree before switching clean. No failed attempt overwritten.
The committed runner retains all25 predecessor commands and adds relay build/
profile plus six dispatch cases. Reviews35 Astra High/36 Fable5.1 High reserved
under allocation23d8e54a1; preserve1–34. No source changes while reviews run.
No negative fixture or ignored case may be relabeled passing without execution.
Manager-accepted policy1.7 internal-only N/A applies only to this increment;
later protected-user/PF26 isolated functional/native/live-repository qualification,
human acceptance, benchmark and whole-sprint/release gates remain open.
