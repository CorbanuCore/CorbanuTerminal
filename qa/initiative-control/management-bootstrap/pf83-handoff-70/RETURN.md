# RETURN — pf83-handoff-70

Action `pf83-handoff-70`; allocation digest
`a8b0986ead59c8be299f3aecf603cf9ebd584ca3a2d761d957fbd63f4f6559c9`;
claim `d519679a-a9d5-4a2b-b6e8-797e4820d74c`; runtime `gpt-6-astra high`.
Brief SHA-256 verified first:
`0b354b09f321e8581d7b52ae12627728d4cd90ebe131d6615c54c5a80fcc255d`.
Clean initial checkout/base `29ea186d4d8177d451d988625f42cc32250366d3`.

**Relocation, selected packet rebinding and transport preparation are complete.
Unconditional functional dispatch is not ready:** the existing boundary refuses
case operations, and the required admitted executor/mediator and guest identity
inputs are absent. No guest staging or case was executed.

- [Relocatable attestation](build-attestation.json): SHA-256
  `9d33fcc21788a84c61011fe64a3607fc7569b390a260e5c19a7b9d3e6582ce27`.
  Package, manifest and binary paths are relative to its declared parent root.
  Original build provenance remains byte-identical and hash-bound.
  [Move proof](move-proof.json) records a real rename and identical before/after
  checks of all four binaries, unchanged attestation digest and absent old root.
  [Verifier](verify_bundle.py) checks independent digest, source identities,
  exact file inventory, hashes/modes and rejects symlinks/unsafe paths.
  Eight tampering cases were rejected; [results](verifier-checks.json).
- Package remains commit `ae5981d2d7d762dfd6d54e0ac0967729bf88f082`,
  tree `834f57388066d33c27fdee6e3b234dec7fa0b0e6`.
  Manifest SHA-256:
  `32529d0a894503c021ff756ebc27a75555ab05ed2ec63dc47dc0d9d7bb878024`.
  The four binaries include the code-mode host; **34/34** current production
  digests still match the previous freeze. No rebuild or stronger source-origin
  claim is implied by this packaging correction.
- All **288 selected coordinator packets** have additive current bindings:
  F05 **8**, F06 **8**, F07 **16**, F08 **40**, F09 **216**.
  [Per-packet old/new hashes](packet-bindings.tsv), [receipt](packet-rebinding.json).
  Original packets and every non-candidate field are unchanged.
  The runbook mandates `rebind_packets.py` into the future receiving directory
  and exact comparison of its two inventories before dispatch.
- Cache exclusion is explicitly limited in the attestation and runbook:
  build-time cache content was not hashed. It cannot exclude a cached binary
  from a different source pin or prove a clean/reproducible build; hashing today's
  cache would not establish its historical state.
- [Execution runbook](runbook.md) provides exact host handoff, packet regeneration,
  strict SSH transport, guest UUID/account/platform and complete inventory checks;
  exact TSV row order; per-case capture/sealing list; and immediate stop conditions.
  The archive includes only original/neutral actor packet assets, plus the trusted
  preflight runtime and provenance. Normalized packets remain coordinator-only.
  [Local extraction proof](stage-receipt.json) pins archive SHA-256
  `3b1afa1c84666f5b3dbc1a514d89bfd138fb15bae9a3813c7b6f5f7d66298ff0`.
  Live harness files were not modified, and the incompatible historical adapter
  is not the staging route.

Required gate, from this checkout's codex-rs, prerequisites built first; shared
dedicated round-67 test target under the allowed QA scope,
`NEXTEST_TEST_THREADS=4`, `INSTA_UPDATE=no`, allowlisted environment,
guarded disposable profiles/native-keyring denial:

| Exact command | Run | Passed | Failed | Skipped | Exit |
| --- | ---: | ---: | ---: | ---: | ---: |
| `just test -p codex-app-server thread_settings --locked --offline --retries 0` | 26 | 26 | 0 | 1082 | 0 |
| `just test -p codex-tui permission_confirmation --locked --offline --retries 0` | 12 | 12 | 0 | 4161 | 0 |

**38 passed; exact failure names: none.** Prerequisite build exit 0.
[Checks and raw-log hashes](checks.json); individual command JSON files record
argv/cwd/environment/exit. One TUI test was slow and passed:
`app::permission_confirmation::tests::permission_confirmation_f10_request_does_not_optimistically_apply_or_persist`
(42.554s). No retry, native prompt or live-profile access observed.
Python and runbook shell syntax checks passed; guest commands were not run.
[Attempts](attempts.md) preserves the initial local copy-mode failure and fresh
successful preparation.

Changed lines: **+1624/-0**: 26 new files (+1622, including 467 Python lines and generated inventories/evidence), plus 2 lines linking the historical runbook. All changes are inside
`qa/initiative-control/management-bootstrap/`; **zero Rust lines**.
No workspace formatter/fix, subagent, guest contact, packaged launch, case,
commit or push. Existing round-67 records and failed local attempt are preserved.

Brief discrepancies/blockers: **380 total normalized packets**, of which 288
belong to the selected inventory. A stale packet pin conflicts with the new
candidate; by itself it does not launch any package, and the old adapter would
reject the new package rather than automatically run the old one.
The requested “without further preparation” dispatch cannot be truthfully
delivered with a preflight-only boundary, absent admitted launcher/mediator,
and missing trusted guest UUID/host-key inputs. Those requirements are named
and stopped on explicitly. Implementing a new authorization boundary would
exceed this routine packaging assignment; no approval is invented. The
independent functional/evidence gate, human acceptance and release qualification
remain open. The existing product heading also explicitly keeps the isolated
broker paused. No claim that this return closes the full functional handoff.
