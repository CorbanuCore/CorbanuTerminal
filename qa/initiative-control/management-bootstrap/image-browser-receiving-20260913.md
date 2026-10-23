# Image transport receiving and cached browser readiness

PF80 bootstrap, September13. Internal prerequisites accepted, not full isolated
functional execution, Slack qualification, a completed sprint or recurring launch.
All three product streams and old automations remain paused.

## Exact image integration

Single-writer `Integrator.merge` received `ea661d9c6cb9d507a1290df4375085b2df3c0547`
at `1061417275ad04d803e77dfd451a9185727bb4c8`, from receiving base
`b39d6298c39adb48e9d35078ee511c2c3eebd565`. Exactly the three allocated files;
no main/release, catalog, credential or active service changes. The424-line
diff (263 non-test) is accepted within the650/350 hard allocation.

Private receipt `.codex-work/image-native-YGnIIE/receiving/image-transport-ea661d9c6.json`,
SHA256 `768cb245eb24d36919a254c115c1df2155d4357ded4c320cea470515a77596dd`,
status verified. Six commands exited0, none timed out:26 normal/2.929s,
26 optimized/2.925s,43 coordinator/1.222s,13 integration/3.454s, plan3/3 and
sprint115 current/126 archived. Total108 actual test executions, not108 unique
cases. Receiving image test roots end `57jknirh` and `xa9pi801` under the parent
`/private/var/folders/3p/j2_d1_f57vz5d0tj3ycvjg9w0000gn/T/isolated-transport-tests-` prefix.

[Transport record](isolated-image-transport.md) preserves original25-test worker
results, the interrupted parent replay, both reviews, malformed-image network
failure and final native positive/read-denial results. Review01's P2 was accepted;
review02 completed clean. No repeated clean review was requested. Byte size,
dimensions, PNG framing/CRC and bounded scanlines validate before auth/process.

The exact native positive correctly read a phrase present only in the screenshot.
The read-denied negative returned NO_IMAGE with a successful text turn, confirming
why `image_observation=supplied_not_attested` is not a visual pass. Later executor
cases require actual actions, observations and independent review, not transport
metadata. Default OFF and all existing tool/credential restrictions remain.

## Cached browser engineering

Private source `.codex-work/docker-browser-CcdJFy/`: three authored files252 lines.
Final real replay `attempt-ngsqlsrf` passed browser render, keyboard expansion,
separate typed text/Enter submission, mouse click and screenshot. Python3.12.13,
Playwright1.59.0, Chromium147.0.7727.0/build1217, pinned cached ARM64 image.
Its root-only public browser cache was extracted from a never-started owned
container, not by granting the executor root or reading a host browser profile.
All468 runtime file hashes were checked; no symlinks/special files. The two binds
are only readonly synthetic guest and readonly public browser runtime.

Outer networknone, private PID/IPC, nonroot UID65534, caps0, NNP1, explicit corrected
seccomp, readonly root and noexec temporary storage remain enforced. All12 observed
Python/Node/Chrome/crashpad/renderer processes had matching restrictions. Inner
Chromium sandbox is explicitly disabled inside that outer boundary; no claim of
renderer isolation or full executor/negative-probe qualification. Final container
exited0 and was removed; fresh label inventory returned no owned browser containers.

| Artifact | SHA256 |
| --- | --- |
| probe.py | 694719f748f379535468745fc8d9e7860cfd3001afad5e66bb27f7f5c4e546f6 |
| guest.py | a766d29e0e1f9630a107de62b235bfabda32d3fde61c3a875695f8c96e9a290b |
| README.md | 0a52c541332aff2bfd2dcac05839e21c8c61e897566b8cfbba26aaf9d024d115 |
| final receipt | 80b5251fb9e1ce32360b1a9111ece4c7f8611ab68b938efdc9998eca22e1a27b |

Fable browser review01 session `01a09a98-689f-7e21-a10e-5ac5ae2e7a50` verified the
actual evidence and found one non-blocking P3: pre-create validation failure also
leaves a conservative owner-reconciliation hold. Parent accepts that limitation
for this retired sequential diagnostic; preserve it for the actual launcher
admission design. Do not apply the suggested negative-inspect shortcut: absence
immediately after a timed-out create cannot prove there is no delayed object.
The helper exited1 with the P3; this is an explicit integrator disposition, not a
claim of a zero-findings review. No repeat of unchanged diagnostic code is needed.

## Next action and remaining launch gates

Fresh Astra High Huygens `01a09bf9-9763-7810-a213-a8f47158750b` is assigned the
three-file fixed confined browser guest on `bootstrap/isolated-browser-guest-20260913`,
base `72429ddaec4ab0f2dd19f4e59da8d095c6bc39a8`. Parent owns actual Docker lifetime,
bounded image/text mediation, frozen-case packet and independent execution/review.
This is bootstrap work in PF80, not a fourth initiative or resumed product sprint.

The volume stall cleared before receiving tests; root cause was not established.
Original reviews resumed rather than being duplicated. Manual Slack poll
`actual-ga5sik5y` observed only the already-recorded reply and kept coordinator
revision63 unchanged. It is not periodic reception. Direct RPC SSH succeeded;
`tailscale serve status --json` still returned an empty object, so private off-Mac
HTTPS is not configured. Normal Slack decision-to-native-ACK recovery, supervised
event/integration wiring and final three-stream rehearsal remain outstanding.
No final completion Slack ping or recurring enablement is authorized by this record.
