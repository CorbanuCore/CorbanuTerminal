PF-33-S01 is merged into main behind a new default-off flag, `url_destination_policy`. PF-33-S02 has not started; the sprint rules block it until S01 is archived (details under "Open").

**What shipped**
- **PR #210** (merged as `8dd531714a`). With the flag on, the managed network proxy applies the existing PF-33-S03 contract to real traffic:
  - **HTTPS on port 443 only.** Plain HTTP, other ports and SOCKS5 UDP are refused. URLs with a username/password or ambiguous syntax are refused.
  - **Every DNS answer must be public.** Answers that are private, loopback, metadata, link-local or mixed are refused, as are failed lookups. `allow_local_binding` and local allowlist entries no longer grant private-network access; only an exact private-service grant does.
  - **Every tunnel is intercepted and checked.** If interception isn't available, the tunnel is refused.
  - **Every redirect is re-checked before it is passed on.** The target must be on the host allowlist before any DNS lookup. Downgrades and private targets are refused. The `Location` header is rewritten to the exact URL that was checked.
  - **Redirect chains stop at 10 hops or 120 seconds.** `Authorization` is removed when a redirect crosses to another origin.
  - **The agent sees the reason**: "blocked by the URL destination policy (<reason>)". Logs and blocked-request records hold only host, port and that reason.
  - **Flag off:** behaviour is unchanged.
- **PR #213** (merged as `39c1f06213`): docs only, records the merge in the sprint record and the sprint index.
- **Sprint allocation.** I took the free TUI lane. The worktree, branch and base are recorded in the sprint file and the plan's front matter. Both checkers pass.
  - The record stays `draft`, because the checker requires PF-27-S02 to be archived first. PF-28-S01 handled this the same way.
  - Shared-file edits are kept to small hunks: the feature registry, the config schema, `core/src/config/mod.rs`, and one line each in `network-proxy` `lib.rs`, `config.rs`, `runtime.rs` and `Cargo.toml`.

**Gate evidence**
- **Tests:** `cargo test -p codex-network-proxy` passes 265 tests plus the 16 contract tests. 23 of them are new `pf_33_s01` tests that use fake DNS answers, so nothing contacts a private address. The new core tests and the schema test also pass.
- **Lint:** `just fix` is clean for network-proxy, features and core, and `just fmt` was run.
- **CI on #210:** all green. Two failures came up and were fixed: a codespell hit in a test string and a stale cargo-shear exception.
- **GLM 5.2 tmux runs:** the agent ran `curl` in its sandbox through the real proxy against httpbin.org, postman-echo.com, localtest.me and nip.io.
- **Independent review (Opus 5.5 High):** three rounds, ending in APPROVE WITH NITS.
  - Round 1 found two serious problems, both fixed. Redirect targets were looked up in DNS before the allowlist check, so data could leak through DNS. Non-TLS tunnels could skip the checks.
  - Round 2 found SOCKS5 UDP unguarded; that is fixed.
  - The reviews are in `qa/security-levels/sprints/PF-33-S01/review-opus-{1,2,3}.md`.
- **Videos:** seven, published to the `demos` prerelease and listed in `qa/demos/index/PF-33-S01.md`. They show redirect re-checks, DNS answers, HTTPS-only, credential stripping, the hop limit, and two flag-off controls.
  - They were recorded on `b40018175d`. The only later changes are the two CI fixes, which don't change product behaviour.
- **Evidence summary:** `qa/security-levels/sprints/PF-33-S01/README.md`.

**Open**
- **Not done in S01: credential path binding.** Legacy brokered credentials are tied to a host only; binding them to method and path as well lives in `credential_broker.rs`, which the broker lane owns. I recommend moving it to PF-28-S02.
- **Known costs, recorded in the evidence summary:**
  - A POST answered with 301/302 is refused, because the contract forbids it.
  - Hosts with more than 16 DNS answers are refused.
  - Redirects to hosts approved only at runtime, not on the allowlist, are refused.
  - There is no in-process end-to-end interception test; the tmux runs cover that path.
- **Decision for Travis: PF-33-S02 can't start.** It needs S01 archived, and S01 can't be archived until PF-27-S02 is archived (it has two open decisions) and the credential-binding item has a home.
  - Option 1: let S02 start now with S01 merged but not archived. This is what PF-28-S01 did.
  - Option 2: wait for PF-27-S02 to be archived.
  - My recommendation is option 1, plus moving credential binding to PF-28-S02.