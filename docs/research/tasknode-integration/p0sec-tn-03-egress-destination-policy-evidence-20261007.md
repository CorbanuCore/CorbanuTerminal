# Task Node evidence record: P0SEC-TN-03

Compile the P0SEC-TN-03 Egress Destination Policy Evidence Record. Task `task_94f31f7ebd6ba20445da0291ca77f945`, request `req_6ab71c71b9e9eaf14e8909d191cdb28ce2a0a6bf7bd2bdad2ea9e09fdc3ef4d6`. Every PR below is merged to `main` in
CorbanuCore/CorbanuTerminal. Each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main`
at `bb609b449a` (2026-10-07). The quoted blocks are copied verbatim from the sprint and gate records (links
re-pointed to this file's location).

Per-sprint gate (Travis, 2026-10-06): focused `just test`, a GLM 5.2 tmux run, one independent Opus 5.5 High
review and SOP demo videos, then merge behind the feature flag. **Not claimed:** the milestone code-blind VM run,
human sign-off, flag removal and the P1 hardening plan. Carried-forward items are recorded as open, not done.

| Sprint | PRs (merge commit on main) | Flag | Record status |
| --- | --- | --- | --- |
| [PF-33-S01](../../sprints/archive/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md) URL, DNS and redirect destination policy | [#210](https://github.com/CorbanuCore/CorbanuTerminal/pull/210) (`8dd531714a`) | `url_destination_policy` | completed (archived) |
| [PF-33-S02](../../sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md) Connection pinning and alternate-egress denial | [#215](https://github.com/CorbanuCore/CorbanuTerminal/pull/215) (`a0d96aea4b`), [#224](https://github.com/CorbanuCore/CorbanuTerminal/pull/224) (`6d55f6ae8f`) | `url_destination_policy` | merged behind flag; record current (milestone items open) |

## PF-33-S01: URL, DNS and redirect destination policy

Summary: tests `cargo test -p codex-network-proxy pf_33_s01` (23) and `-p codex-core pf_33_s01` (1); `cargo test -p codex-network-proxy` 265 + 16 contract tests. TUI run: GLM 5.2 tmux runs through the real proxy. Review: Opus 5.5 High review and re-checks, dispositioned.

Gate record: [qa/security-levels/sprints/PF-33-S01/README.md](../../../qa/security-levels/sprints/PF-33-S01/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md`, Verification:

> - [x] `just fix -p codex-network-proxy`, `-p codex-features`, `-p codex-core`; `just fmt`; final diff inspected.
> - [x] Focused: `cargo test -p codex-network-proxy pf_33_s01` (23 passed); `cargo test -p codex-core pf_33_s01` (1).
> - [x] Integration: `cargo test -p codex-network-proxy` (265 + 16 contract tests); core schema fixture test passes.
> - [x] TUI: GLM 5.2 tmux runs through the real proxy against httpbin.org, recorded as SOP videos.
> - [x] Independent Opus 5.5 High review and re-checks; findings dispositioned in the evidence README.
> - [x] Linux and Bazel CI on the PR: all checks green; merged as PR #210 (`8dd531714a`).
> - [x] PF-26 final-candidate requalification runs at the milestones, not per sprint (decision 5).

Demo videos (7, index [qa/demos/index/PF-33-S01.md](../../../qa/demos/index/PF-33-S01.md)):

- `pf33s01-baseline-flag-off`: Control: with url_destination_policy off, the proxy relays redirects to http:// and private targets: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-baseline-flag-off-b40018175d73-2026-10-06.mp4
- `pf33s01-redirect-reauthorized`: URL destination policy: every redirect is re-authorized before the proxy relays it: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-redirect-reauthorized-b40018175d73-2026-10-06.mp4
- `pf33s01-dns-answers`: URL destination policy: every DNS answer must be public; allow_local_binding is not a private-network grant: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-dns-answers-b40018175d73-2026-10-06.mp4
- `pf33s01-https-only`: URL destination policy: public retrieval is HTTPS on port 443 only: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-https-only-b40018175d73-2026-10-06.mp4
- `pf33s01-credentials-flag-off`: Control: with url_destination_policy off, curl --location-trusted carries Authorization to another origin: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-credentials-flag-off-b40018175d73-2026-10-06.mp4
- `pf33s01-credentials-same-origin`: URL destination policy: Authorization does not follow a redirect to another origin: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-credentials-same-origin-b40018175d73-2026-10-06.mp4
- `pf33s01-hop-limit`: URL destination policy: a redirect chain is cut off after 10 hops: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-hop-limit-b40018175d73-2026-10-06.mp4

Carried forward (open, not claimed): Legacy per-host adapter binding to PF-28-S02; pinning done in PF-33-S02; documented limits.

## PF-33-S02: Connection pinning and alternate-egress denial

Summary: tests network-proxy `pf_33_s02` (15), sandboxing (1), core (1); `just test -p codex-network-proxy` 296 passed; `just test -p codex-sandboxing` 79 passed (2 fail identically on base). TUI run: GLM 5.2 tmux runs. Review: Opus 5.5 High: APPROVE WITH NITS.

Gate record: [qa/security-levels/sprints/PF-33-S02/README.md](../../../qa/security-levels/sprints/PF-33-S02/README.md).

Verbatim, sprint record `docs/sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md`, Verification:

> - [x] `just fix -p codex-network-proxy`, `-p codex-sandboxing`, `-p codex-core`; `just fmt`; final diff inspected.
> - [x] Focused: `cargo test -p codex-network-proxy pf_33_s02` (15), `-p codex-sandboxing pf_33_s02` (1),
>   `-p codex-core --lib pf_33_s02` (1). `codex-secret-broker` is broker scope: no test there.
> - [x] Integration: `just test -p codex-network-proxy` (296 passed); `just test -p codex-sandboxing` (79 passed, 2
>   failing identically on the base commit: temp-dir writable roots, unrelated); core `windows_sandbox`/`network_proxy`.
> - [x] GLM 5.2 tmux runs as five SOP videos ([index](../../../qa/demos/index/PF-33-S02.md)); Opus 5.5 High review
>   APPROVE WITH NITS, dispositions in the [evidence README](../../../qa/security-levels/sprints/PF-33-S02/README.md).
> - [x] PR CI green; merged behind `url_destination_policy` as PR #215 (`a0d96aea4b`).
> - [ ] PF-26 final-candidate requalification (milestone gate).

Demo videos (5, index [qa/demos/index/PF-33-S02.md](../../../qa/demos/index/PF-33-S02.md)):

- `pf33s02-pinned-dial`: Connection pinning: each request connects only to the DNS answers the destination guard checked: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s02-pf33s02-pinned-dial-cde3d0f49988-2026-10-06.mp4
- `pf33s02-sandbox-egress`: Alternate-egress denial: the sandbox reaches only the proxy (no loopback services, raw DNS or Unix sockets): https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s02-pf33s02-sandbox-egress-cde3d0f49988-2026-10-06.mp4
- `pf33s02-sandbox-egress-flag-off`: Control: with url_destination_policy off, loopback services, raw DNS and granted Unix sockets bypass the proxy: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s02-pf33s02-sandbox-egress-flag-off-cde3d0f49988-2026-10-06.mp4
- `pf33s02-upstream-proxy`: Alternate-proxy denial: the guard refuses to send traffic through an inherited upstream proxy: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s02-pf33s02-upstream-proxy-cde3d0f49988-2026-10-06.mp4
- `pf33s02-upstream-proxy-flag-off`: Control: with url_destination_policy off, agent traffic leaves through the inherited upstream proxy: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s02-pf33s02-upstream-proxy-flag-off-cde3d0f49988-2026-10-06.mp4

Carried forward (open, not claimed): SearXNG adapter absent; decider-host revocation on open tunnels (PF-25-S02).
