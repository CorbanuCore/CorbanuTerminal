PF-33-S02 is merged into main behind the existing default-off flag `url_destination_policy` as PR #215. The independent review approved it with nits, all checks passed, and five demo videos are published. With the flag off, behaviour is unchanged.

**Coordinator decisions, recorded**
- **S02 started with S01 merged but not archived.** This is recorded in both sprint files. The sprint checker blocked it, so I changed it minimally: a dependency now counts as met if its record names the flag it merged behind and points to a gate evidence file that exists in the repo. There is a unit test, and one clause in `docs/sprints/index.md`. PF-33-S01 now uses this.
- **Credential method/path binding moved to PF-28-S02.** It is a Remaining item there and is gone from S01's list.

**What shipped (PR #215, merged as `a0d96aea4b`)**
- **Connection pinning.** Each guarded HTTPS request carries the DNS answers the guard already checked. The proxy connects only to those addresses, for that exact host and port, and never looks the name up again. A connection with no checked answers, or with answers for a different host, is refused. Each address still gets the private-address check, TLS is still verified against the requested name, and every request opens a fresh connection.
- **Other ways around the proxy are refused.** The agent sees "Destination policy denied the request (…)" with the reason in brackets:
  - An upstream proxy inherited from `HTTPS_PROXY`/`ALL_PROXY` (`upstream_proxy`).
  - The proxy's `x-unix-socket` route to local services (`unix_socket`).
  - A host added to the deny list after its tunnel opened, on that open tunnel (`host_denied`).
- **The macOS sandbox now reaches only the proxy's own ports.** Local binding and Unix-socket grants are dropped, so there is no direct access to other loopback services, raw DNS on port 53, or Unix sockets. Windows firewall setup also turns local binding off. The Linux sandbox already isolated these and is unchanged.
- **Scope adjustments:** I reused the existing flag. Pinning lives in `connect_policy.rs` instead of a new file, so `lib.rs` (reserved by PF-27-S02) is untouched. Nothing changed in Cargo files or lockfiles.
- **PR #217 (merged as `4b42012daa`)**, docs only: records the merge and returns S02 to `draft`, which frees the TUI lane.

**Gate evidence**
- **Tests:**
  - network-proxy: 15 new `pf_33_s02` tests, and the full suite passes (296). They use real local TCP/TLS servers reached through names that can't resolve, which proves the pinned address was used.
  - One new macOS sandbox test and one new Windows setup test, both passing.
  - Two other sandbox tests fail on this machine. They fail the same way on the base commit and are unrelated (temp-directory paths).
- **GLM 5.2 tmux runs (five SOP videos)**, listed in `qa/demos/index/PF-33-S02.md`:
  - Pinned connections to two real sites.
  - Sandbox bypass attempts with the flag on, plus a flag-off control.
  - Inherited upstream proxy with the flag on, plus a flag-off control.
- **Independent review (Opus 5.5 High):** APPROVE WITH NITS, no P0 or P1. I fixed two nits: a 30-second total limit on connection attempts per request, and a test for internationalised hostnames. The other items are answered in the evidence README.
- **Evidence:** `qa/security-levels/sprints/PF-33-S02/README.md` holds the per-platform table and the review answers.
- **Linux clippy:** not run on the RTX box, because none of the new code is Linux-only. PR CI was all green.

**Open**
- **Brokered credentials are not pinned.** Requests that go through the credential broker skip pinning; the broker looks the name up again, though its own private-address check still applies. Fixing this means passing the checked answers through the broker, which is broker-lane work and needs a place in that lane.
- **SearXNG:** there is no adapter in the tree yet.
- **Revoking approved hosts:** a host approved at runtime is not cut off on a tunnel that is already open. That belongs to PF-25-S02.
- **Root policy still contradicts the checker.** `AGENTS.md` still says a dependency must be archived first; the plan owner should reconcile it with the new checker rule.
- **Corporate proxies:** with the flag on, a user whose own `HTTPS_PROXY` is set gets no network from the agent. I chose to refuse, because routing around a corporate proxy would bypass its controls; Travis can reverse this if he prefers.