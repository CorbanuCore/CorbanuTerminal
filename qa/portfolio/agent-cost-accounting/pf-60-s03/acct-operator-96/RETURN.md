# RETURN

Action `acct-operator-96`; allocation digest
`19185ce44dbae415bbb56f7a1472734f65bc2df47747d9ba48ec652e26d796e7`;
claim `9143f414-fc7b-48fe-ae28-02caae76ed37`. Worker: `gpt-6-astra`, high.
The frozen brief SHA-256 matched
`9e471b4b035adc2e6fb5591edf3f580ce410bc082e9b8a1d350cfaf31a353b38`.

The [corrected note](../acct-derive-95/OPERATOR.md) was executed as six unchanged
shell blocks, from clone creation through all three gates. The
[complete transcript](transcript.txt) retains verbatim inputs and merged raw
stdout/stderr, with block boundaries, working directories and exit codes.
[Results](results.json) bind the note, commands and outputs by SHA-256;
`block-01.json` through `block-06.json` retain individual receipts, and the
corresponding `.log.gz` files retain lossless raw output. No attempt was retried
or replaced.

- Executed note SHA-256:
  `632c78b084d7145d5dc406e0d8a96e0d12eea30a4d85a46129526928abd76140`.
- Transcript SHA-256:
  `81399ab1e0ba22c2ed4a339627dbf77ac4ae2ed78187b55956efc27e8552b88c`.
- Candidate and source HEAD: `f565aa0f68b3361b80b1302e5606bac60f5f4ee4`.
- Fresh full-history clone: `acct-operator-96/target/audit-copy` beneath this
  QA scope. Block 1 ran from the supplied root; blocks 2–6 ran from that fresh
  clone's root. Every block used a new `bash --noprofile --norc` process, with
  no variables inherited from another block. `run_block.py` extracted the
  exact block text from [the frozen note](OPERATOR.executed.md); it did not
  rewrite shell inputs. The corrected note was an external frozen input; the
  cloned candidate excluded the uncommitted note/evidence edits, as documented
  and shown in the source-status output.
- One initially empty dedicated target served the prerequisite build and all
  tests, under the clone's `acct-operator-96/target/cargo`. The printed
  `git check-ignore -v` checks passed. `cargo fetch --locked` populated missing
  dependencies using the existing toolchain/dependency cache locations and
  available network; no offline adaptation or copied build target was used.

| Lane | Passed / run | Skipped | Failed | Timed out | Failure names | Exit |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| Core default accounting | 124 / 124 | 3545 | 0 | 0 | `[]` | 0 |
| Core developer-accounting | 127 / 127 | 3545 | 0 | 0 | `[]` | 0 |
| TUI usage | 91 / 91 | 4078 | 0 | 0 | `[]` | 0 |

Prerequisites passed first, exit 0. All gates ran from `codex-rs` through
`just test`, with `NEXTEST_TEST_THREADS=4`; all emitted the required isolation
banner. There were 342 overlapping test executions, not 342 distinct tests.
Compiler warnings are retained. No native credential prompt or live-profile
read was observed. No credentials were copied or printed.

The acceptance verifier produced `BASELINE MATCH` and
`RESULT agreement=20 disagreement=0 unavailable=3 exit=2`; all 13 controls
passed; replay exited 0 with matching stdout and nested exits 2/0, empty stderr
and clean clone status. The [replay streams and receipt](replay-streams/receipt.json)
are retained outside ignored scratch space.

The note now explicitly discloses replay self-reference: the disposable clone's
integration ref selects the candidate, whose verifier/evidence are compared with
expected bytes and their digest in that same candidate. This proves internal
agreement, not independent historical or integration approval. The supplied
repository's integration ref remained
`895fdf06025815b3277e0c1630d037994a72dd93`; only the clone's ref was re-pointed.

The empty-initial-status requirement was dropped. Source status is now printed
before cloning so omitted edits are visible where they actually exist. The
transcript shows the pending note and harness edits; it does not disguise them
as included candidate files.

The three missing prerequisites are supplied inline: the exact isolation banner,
`Test isolation: disposable profile; native keyring disabled (debug lane)`;
`acct-reference-88/expected.stdout.txt` and its adjacent manifest/digest as the
comparison reference; and `cargo fetch --locked`, plus explicit cache-transfer
and build-script-artifact guidance for disconnected hosts.

Changed note lines: **94 added, 21 removed**, with exact hunks in
[changed-lines.patch](changed-lines.patch). Changes cover clone setup/source
status, per-block variables/working directory, self-reference disclosure,
dependency preparation, separate gate blocks, banner and comparison reference.
All other additions are this execution's evidence and its two small runner/
collector scripts. `git diff --check` passed; the audit clone remained clean;
all source changes remain under the assigned QA scope. No workspace formatter,
Rust/product edit, commit or push was performed.

One qualification to the brief: the existing `*/target/` rule already covers
the old *intended* `$PWD/qa/portfolio/agent-cost-accounting/pf-60-s03/acct-derive-95/target`
path. The escape occurs when `$q` is unset in a fresh shell, producing a different
path outside that rule. No ignore-rule amendment was necessary; every block now
defines its own `q`, and the target's ignore match is executed and retained.
No other supplied finding was contradicted by this work.

The opening limits and entire three-binary exclusion paragraph are unchanged.
`codex`, `codex-code-mode-host` and `rmcp_test_server` historical package bytes
remain unavailable from the repository. Independent functional acceptance and
human approval remain unverified. The historical timeout and scope-zero finding
remain open; these passes do not accept S03 or authorize shipment.

Classification: routine QA-note/evidence correction. Product context:
**Product measurement → Measurement targets**, “No commercial performance
numbers have been supplied.” No plan/sprint lifecycle change is claimed.
True-TUI, code-blind functional and live-repository qualification are not part
of this internal documentation revision, which changes no product workflow and
makes no functional handoff; the existing S03 functional gate remains due.
Human acceptance, benchmarks and release readiness are not claimed.
