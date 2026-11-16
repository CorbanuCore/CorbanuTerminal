# #349: 13 network-proxy tests fail without network

Issue: https://github.com/CorbanuCore/CorbanuTerminal/issues/349
Follow-up to #1892 and #1899. The #1899 review listed these 13.

## Environment

- OS: Pop!_OS 24.04 LTS (kernel 6.18.7-76061807-generic)
- rustc 1.99.1 (58cf8f9c3 2026-10-13)
- cargo 1.99.1 (5f94df478 2026-08-27)
- Checkout: main d365f7af80 (2026-11-15)

## Reproduction

Network cut with a user + net + mount namespace (loopback up, nsswitch set to plain dns). Each test run on its own:

```
printf 'hosts: files dns\n' > /tmp/nss-plain
unshare -rnm sh -c 'ip link set lo up && mount --bind /tmp/nss-plain /etc/nsswitch.conf && cargo test --offline -p codex-network-proxy -- --exact <test>'
```

I ran all 331 tests in the crate this way, once with network and once without. With network: 0 fail. Without network these 13 fail:

- `http_proxy::tests::http_connect_accept_blocks_in_limited_mode`
- `mitm::tests::mitm_policy_allows_matching_hooked_write_in_full_mode`
- `mitm::tests::mitm_policy_blocks_disallowed_method_and_records_telemetry`
- `mitm::tests::mitm_policy_blocks_hook_miss_for_hooked_host_and_records_telemetry_in_full_mode`
- `network_policy::tests::evaluate_host_policy_emits_domain_event_for_decider_allow_override`
- `network_policy::tests::evaluate_host_policy_emits_domain_event_for_decider_ask`
- `network_policy::tests::evaluate_host_policy_emits_execution_id_for_baseline_allow`
- `runtime::tests::add_allowed_domain_removes_matching_deny_entry`
- `runtime::tests::host_blocked_global_wildcard_allowlist_allows_public_hosts_except_denylist`
- `runtime::tests::host_blocked_requires_allowlist_match`
- `socks5::tests::handle_socks5_tcp_blocks_limited_mode_without_mitm_state`
- `socks5::tests::handle_socks5_tcp_detects_tls_for_brokered_nonstandard_port_in_full_mode`
- `socks5::tests::handle_socks5_tcp_uses_mitm_in_limited_mode`

## Expected vs actual

Expected: these tests cover allowlist and denylist matching, decider events, MITM method and hook policy, and SOCKS5 and CONNECT routing. None of that needs the internet, so they should pass offline.

Actual: all 13 fail offline. Full unedited output is in `before.txt` next to this file. The failure lines:

```
thread 'http_proxy::tests::http_connect_accept_blocks_in_limited_mode' (30154) panicked at network-proxy/src/http_proxy.rs:1218:9:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'mitm::tests::mitm_policy_allows_matching_hooked_write_in_full_mode' (30169) panicked at network-proxy/src/mitm_tests.rs:273:5:
matching hook should bypass method clamp
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


thread 'mitm::tests::mitm_policy_blocks_disallowed_method_and_records_telemetry' (30184) panicked at network-proxy/src/mitm_tests.rs:165:5:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'mitm::tests::mitm_policy_blocks_hook_miss_for_hooked_host_and_records_telemetry_in_full_mode' (30199) panicked at network-proxy/src/mitm_tests.rs:363:5:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'network_policy::tests::evaluate_host_policy_emits_domain_event_for_decider_allow_override' (30218) panicked at network-proxy/src/network_policy.rs:659:9:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'network_policy::tests::evaluate_host_policy_emits_domain_event_for_decider_ask' (30233) panicked at network-proxy/src/network_policy.rs:811:9:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'network_policy::tests::evaluate_host_policy_emits_execution_id_for_baseline_allow' (30248) panicked at network-proxy/src/network_policy.rs:724:9:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'runtime::tests::add_allowed_domain_removes_matching_deny_entry' (30263) panicked at network-proxy/src/runtime.rs:1326:9:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'runtime::tests::host_blocked_global_wildcard_allowlist_allows_public_hosts_except_denylist' (30278) panicked at network-proxy/src/runtime.rs:1591:9:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'runtime::tests::host_blocked_requires_allowlist_match' (30293) panicked at network-proxy/src/runtime.rs:1302:9:
assertion failed: `(left == right)`

[1mDiff[0m [31m< left[0m / [32mright >[0m :

thread 'socks5::tests::handle_socks5_tcp_blocks_limited_mode_without_mitm_state' (30308) panicked at network-proxy/src/socks5.rs:1186:9:
unexpected error: Custom { kind: PermissionDenied, error: "Sandbox policy blocks local/private network addresses." }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


thread 'socks5::tests::handle_socks5_tcp_detects_tls_for_brokered_nonstandard_port_in_full_mode' (30323) panicked at network-proxy/src/socks5.rs:1159:10:
brokered TLS should defer MITM until protocol detection: Custom { kind: PermissionDenied, error: "Sandbox policy blocks local/private network addresses." }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


thread 'socks5::tests::handle_socks5_tcp_uses_mitm_in_limited_mode' (30350) panicked at network-proxy/src/socks5.rs:1082:10:
limited-mode HTTPS should use MITM: Custom { kind: PermissionDenied, error: "Sandbox policy blocks local/private network addresses." }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Root cause

With `allow_local_binding` false (the default), `host_blocked` resolves the hostname to make sure it isn't a local or private address. Offline the lookup fails, `host_resolves_to_non_public_ip` fails closed, and the host comes back `not_allowed_local` before the check each test is actually about.

## Fix

Test-only. Each of the 13 sets `allow_local_binding: true`, same as #1892, #1899 and #1703. That skips the local-address lookup in `host_blocked`. It also allows private targets at dial time, which none of these tests reach. Tests that cover the local check itself already set it to false and aren't touched.

## Results

Before: 13 of 13 fail offline. After: 13 of 13 pass offline (`after.txt`).

Whole crate after the fix, `cargo test --offline -p codex-network-proxy`:

```
offline:
test result: ok. 315 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
online:
test result: ok. 315 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.13s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt -p codex-network-proxy -- --check` is clean.
