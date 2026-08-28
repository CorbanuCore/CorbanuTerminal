# PF-30-S01 platform qualification attempt

Date: 2026-08-27 Arizona time (2026-08-28 UTC). Historical initial phase, before
Travis authorized Linux sudo authentication and Mac shared-engine recovery.
All three qualifications were initially blocked. The subsequent
[qualification findings](platform-qualification-2026-08-27.md) supersede the
prerequisite status here: engines recovered, Mac/Linux smoke failed, Windows held.

Change class: existing product initiative, active `p0-security-levels` plan,
PF-30-S01 `in_progress`. Product authority: **Moderate/Aggressive isolation and
content provenance** — “Support Windows, Linux, and macOS with containerized
Scrapling”; “Reuse an installed Podman or Docker runtime without replacing it
or changing its global configuration.”

## Candidate identity

- Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01`.
- Branch: `codex/pf-30-isolated-runtime`.
- Checkout: `ddfb27b4daed6d022403ae0c4e2d869f1910f02a`.
- Code candidate: `322c81b88cbe8e90ba02b16937a4c7b4a792bb1c`; the later commit
  changes evidence only. All 28 [source fingerprints](candidate-files.sha256)
  verified before this run; no production code changed in this attempt.
- Mac smoke binary: `codex-rs/target/ci-test/examples/qualify` in the shared
  build cache at `/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01`.
- Binary SHA-256:
  `c2288dafd9a7b5b605618620fa3f16802befac71178c7dbdf7bc45fffc527b9b`.

## Observed platform results

| Platform | Observed state | Qualification result / unblock |
| --- | --- | --- |
| Mac arm64, macOS 15.6.1 / 24G90 | Docker Desktop 4.39.0, CLI 28.0.1, existing `desktop-linux` context. Daemon unavailable; about 16 GiB free. | Real smoke exits 1 with `RuntimeUnavailable`; needs shared Docker engine recovery before any container proof. |
| Linux `travis@100.99.88.49`, Ubuntu 26.04 LTS x86_64, kernel 7.0.0-30-generic | SSH succeeds with existing host-key verification. No Podman/Docker, Cargo/Rust or rootless UID helpers on PATH; Python/tmux available; about 169 GiB free. | Prerequisite-blocked; no Linux candidate build or runtime probe yet. User must perform privileged package installation, then rootless capabilities and build prerequisites must be checked. |
| Windows `postfiat@postfiat1`, supplied IP `100.111.98.12` | Mac tailnet is `ambientcrypto.ai`; neither `postfiat1` nor `productionrpc` appears among relevant peers. Name resolution fails on Mac/Linux. TCP 22 times out on Mac and reports network unreachable on Linux. | Network-blocked; no SSH login, candidate transfer/build or Windows runtime test. Leave on hold as Travis anticipated. |

### Mac: actual backend probe

From the candidate's `codex-rs` directory:

```text
CARGO_TARGET_DIR=/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01/codex-rs/target cargo run -p codex-browser-isolation --example qualify --profile ci-test --locked
   Compiling codex-network-proxy v0.1.35
   Compiling codex-browser-isolation v0.1.35
    Finished `ci-test` profile [unoptimized] target(s) in 5.29s
     Running `.../target/ci-test/examples/qualify`
Error: RuntimeUnavailable
```

Above compiler paths are abbreviated; exit status is **1**. This command runs
the existing example, not `cargo test`. The example stops during preparation,
before health, Chromium, public fetch or its four private/file URL checks.
No success assertion ran. The earlier 270-test checkpoint is unchanged and is
not relabeled as a new full-suite run or platform proof.

`docker version --format '{{json .Server}}'` returned `null` and “Cannot connect
to the Docker daemon at unix:///Users/travisgood/.docker/run/docker.sock.”
`curl --max-time 5 --unix-socket /Users/travisgood/.docker/run/docker.sock
http://localhost/_ping` also failed to connect (exit 7). Both configured Docker
contexts resolve to that socket, directly or through `/var/run/docker.sock`.

Computer Use inspection of Docker Desktop displayed “Engine running” but no
container metrics and three unrelated Ambient containers. The UI indicator is
not accepted as backend health. Requested the user's go-ahead or manual action
for shared-engine recovery; did not restart it, stop containers, change contexts,
install a replacement engine or delete data. No PF-30 image/container was created.

### Linux: installation and Windows-route preflight

Read-only SSH checks included `uname -srm`, `/etc/os-release`, `id`, `command -v`,
`df -h .`, UID-helper file checks and `apt-cache policy`. `sudo -n true` reported
“interactive authentication is required.” Package candidates from the configured
Ubuntu repositories, not installed versions:

| Package | Candidate |
| --- | --- |
| podman | `5.7.0+ds2-3build1` |
| uidmap | `1:4.17.4-2ubuntu3` |
| passt | `0.0~git20260120.386b5f5-1` |
| slirp4netns | `1.3.3-1` |

Asked Travis to run `sudo apt-get install podman uidmap passt slirp4netns` and
enter the elevation password himself. SSH authentication was used only for SSH;
no credential was placed in files/evidence or reused for sudo. No install or
remote configuration change was performed. SSH sessions exited normally.

`getent passwd pfrpc` and `getent hosts productionrpc postfiat1` produced no
entries on this Linux host. It is not the supplied `productionrpc` access path.
`tailscale ping --c 1 --timeout 5s postfiat1` failed name resolution on both
hosts. Mac `nc -vz -G 5 100.111.98.12 22` timed out; Linux
`nc -vz -w 5 100.111.98.12 22` reported network unreachable.

Windows host fingerprint supplied by Travis, **not verified by a connection**:
`SHA256:AL/bFpl1DX/lsk94alfnPXX20Fgx+PgIXyzsF9FATZ0`. No private key was copied,
no tailnet/SSH authorization was changed and no attempt was made to treat the
existing installed Windows binary as this candidate.

## Remaining gates

After infrastructure recovery, build/pull the pinned derivative and run actual
containment, egress/redirect, cancellation/timeout, crash/restart, ownership,
quarantine and platform probes against recorded candidates. Rootless Linux and
Windows VM/WSL behavior still need direct evidence. Shared native allowlist/DNS
ordering and coverage gaps in [review fixes](review-fixes.md) remain unresolved;
working infrastructure alone will not complete S01.

S01 is an internal, unactivated backend: no public TUI path is applicable here;
S02/S03 own the actual-key integration/setup proof. No live-repository, human
acceptance, benchmark or release pass is claimed. No dependent sprint starts.
