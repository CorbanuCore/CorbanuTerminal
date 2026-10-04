# RETURN — pf83-native-f05-54

Action `pf83-native-f05-54`; allocation digest `e783b50206753e96a460504b1c540451aa1456a18ff93cc8c16c70bab8fa446a`; claim `9c8a754a-a232-44c8-9104-2fee93039247`; worker `gpt-6-astra`, effort `high`.
Verified the brief first with `shasum -a 256`: `40ea77229e6098b4b6cc8855dba79b8aad1f50c1dbe710932a457b7906e5d841`.
Base/HEAD `f2d1a6478da22a9b334dfcd57c01e25dccb9f07b`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`; initially clean. No commit or push performed.

Class: routine test/evidence work, zero product changes. Product heading **Permission selection confirmation — TO BUILD**, excerpt: “Existing active-turn approval/sandbox snapshots and pending approvals are not retroactively changed.” Also: “Show a clear finish/stop-current-turn and continue path without automatic interruption, command approval or replay.” Supports PF-83-S01; no plan/sprint scope or case text amended. Independent functional acceptance and human-test readiness are not claimed.

## Per-case verdicts and limits

All verdicts below refer to the specified native observable claims. They are not substitutes for the full frozen human cases, packaged real-key execution, independent execution/review, both live repositories, or other platforms. Native inference is scripted; the real product authorizes and executes real Python commands against disposable files. No production behavior is stubbed or changed.

| Original case | Native verdict | Constructed claim and exact remaining gap |
| --- | --- | --- |
| F05 | Built and passing | Two fresh runs leave the original approval withheld through Applied and a later backend round trip, then decline or accept once. Selection alone writes zero; decline writes zero; accept writes once; distinct next turn writes once with zero approvals. Request count and append counts exclude reissue/replay. Native RPC proves the request remains resolvable; the picker/approval overlay and understandable visual disposition still require exact-package real keys. |
| F06 | Built and passing | Both directions begin with an actual started receipt and no finished receipt. Restricted start is explicitly approved. Opposite selection cannot interrupt/replay this command: after barrier release started/finished are exactly one each. Old continuation uses its captured authority; separate next turn uses the selected authority. UI explanation/partial-effect presentation is outside these native fixtures. |
| F07 | Built and passing (native protocol plus TUI fixture) | Both three-selection orders are submitted during real in-flight work before any response is read; all three receive distinct Applied outcomes. Newest response is consumed first, then older buffered replies; final fresh probe matches final accepted authority. Separate embedded TUI fixture tests visible refusal of conflicting requests and obsolete completion delivery during/after a later accepted request. This is composition of native protocol effects and TUI handler assertions, not a single real-key workflow or transport-injected stale-notification test. |
| F09 | Built and passing | Shared F06/F09 fixtures cover both directions: old restricted continuation is declined/zero, new full probe writes once; old full continuation writes once, new restricted probe is declined/zero. Started/completed effects remain exactly once. Visible retained context, stop/new-session navigation, deferred-user-input presentation and resistance to permission claims in conversation wording are not qualified by this fixture. |
| F10 | Three native restart branches built and passing; full case not expressible here | Real app-server process shutdown, new process, same synthetic profile and `thread/resume` cover effective R, effective F, and unresolved command approval. No inferred acceptance or automatic model replay; old effect counts survive and new probes agree with reported authority. Settled runs permit a reported reset instead of assuming persistence. Cannot construct the original (c) pending-change states in both directions at a known pre-application barrier through the available test APIs: holding a reply is not holding Core application. An exact-package isolated guest/account with a demonstrated external fault/barrier seam and ordinary restart/recovery navigation can cover those states and the UI explanation. No abrupt-interruption branch is claimed. |
| F11 | Not expressible with these native fixtures; not built | The step “repeat F03/F04/F05 on each exposed supported model route” requires actual route identity and inference, plus a supported pending route switch if exposed. Two mock provider configurations would be substitutes expressly excluded by the allocation. Needs an isolated exact-package environment with mediated access to at least two actually exposed supported routes, route receipts, real command effects, and real keys. No live credentials were accessed. |

## Fixture map and out-of-band observations

Native test prefix: `suite::v2::thread_settings_update::` in `codex-rs/app-server/tests/suite/v2/thread_settings_update.rs`.

| Fixture suffix | Effects and non-vacuity |
| --- | --- |
| `thread_settings_f05_pending_decline_is_not_approval_or_replay` | One actual `old` approval is observed before selection; a post-Applied `thread/read` completes while decision is withheld and inference requests remain one. After decline and turn completion old append count is zero, extra approvals zero, inference count two. Distinct new probe writes once without approval; total inference requests four. |
| `thread_settings_f05_pending_accept_runs_once_and_next_turn_is_full` | Same pre-decision zero control; explicit wire `accept` executes old append once, never twice; no second approval. Distinct new write once with zero approvals; old still once; four inference requests. |
| `thread_settings_f06_f09_inflight_restricted_to_full_keeps_old_boundary` | Explicitly approve `in-flight`, observe started=1/finished=0, select Full, release, then finished=1. Old continuation approval is observed and declined, zero effect; new-turn probe writes once without approval. Five inference requests prove both command continuations reached the provider and completed. |
| `thread_settings_f06_f09_inflight_full_to_restricted_keeps_old_boundary` | Started=1/finished=0 precedes restriction; release produces finished=1. Old continuation writes once without approval; new-turn approval is observed/declined and writes zero. Started/finished remain one and five inference requests exclude replay. |
| `thread_settings_f07_rapid_conflicts_end_full` | Real in-flight barrier plus three request-specific Applied responses; final full probe writes once with zero approvals, started/finished each one, four inference requests. |
| `thread_settings_f07_rapid_conflicts_end_restricted` | Initial barrier explicitly approved, then same conflicting-selection sequence ending restricted; final probe approval observed and declined, zero effect, started/finished each one, four inference requests. |
| `thread_settings_f10_restart_effective_restricted_reports_probe_authority` | Before restart a real approval is declined and the old marker is zero; two inference requests complete. New process resumes without overrides and sends zero inference requests automatically. Fresh probe approval/effect matches returned policy/sandbox; old count remains zero; exactly two new inference requests. |
| `thread_settings_f10_restart_effective_full_reports_probe_authority` | Before restart old append is one without approval; resume sends zero model requests. Fresh probe follows returned authority; original append remains one. Two requests before and two after exclude replay/duplication. |
| `thread_settings_f10_restart_does_not_accept_pending_approval` | Original wire approval is observed and deliberately unanswered; zero old effect before/after process exit. Resume in a new process sends zero model requests and does not report Never approval. New probe has one declined approval and zero effect; old remains zero; new inference count two. |

TUI fixture: `app::permission_confirmation::tests::permission_confirmation_f07_conflicts_are_refused_and_old_completion_cannot_win` in `codex-rs/tui/src/app/permission_confirmation_tests.rs`. Both orders use real embedded confirmation requests. Refusal text is observed and no requested-success text for the conflicting label is permitted. Old completion IDs are delivered while a later request is pending and after it settles: pending identity, effective/fresh-session config, history-event channel and command channel prove no overwrite, extra success, approval, interruption or replay. Real file effects are supplied by the separately listed native F07 fixtures; this TUI test alone is not shell execution proof.

All command effects use append-mode `executed\n` records and direct filesystem reads, never “Ran” rendering. Missing-file is accepted as zero only together with an observed originating command approval, an executed barrier, completed turn/model-request counts, or a paired executing direction/control. Unexpected filesystem errors fail. Started is separate from finished; the barrier is bounded and expiration fails rather than producing a false completion. Approval requests buffered during other RPC reads are later counted; extra/reissued requests fail the expected totals. Temporary fixtures are deleted by their owners after assertions.

## Validation

Read `docs/development/test-isolation.md` before any tests. All Rust tests use this checkout's guarded `just test`, `INSTA_UPDATE=no`, `--locked --offline --retries 0 --test-threads 1`; no native prompt observed. Host Darwin arm64, rustc 1.95.0. One shared dedicated target: `/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-rebind-31`. From `codex-rs`, prerequisite command `cargo build --locked --offline -p codex-cli --bin codex -p codex-rmcp-client --bins -p codex-code-mode-host` exited 0 before tests. Builds are not raw Cargo tests.

Only owned files were formatted using `rustfmt --edition 2024 --config skip_children=true`; no `just fmt`, `just fix`, or workspace formatting. Scoped status/diff checks passed afterward. Final affected lanes follow the last edit/format of the test files they cover.

The first permission lane failed exactly `app::permission_confirmation::tests::permission_confirmation_f07_conflicts_are_refused_and_old_completion_cannot_win` (93 passed / 1 failed, exit 100): its command-channel assertion included pre-selection initialization. The preserved focused replay `pf83-native-f05-54-f07-startup-control.log` reports exactly `[ListSkills { cwds: ["/tmp/project"], force_reload: true }]` before any selection in both fresh starts and then passes all F07 assertions (1 passed, exit 0). The final fixture explicitly asserts that pre-selection commands are only ListSkills; after that boundary, the no-command assertion remains intact. This is a measured fixture setup correction, not a product regression or a relaxed permission assertion. No original log was overwritten. The final filesystem assertion was also strengthened: zero effects requires the marker to be absent, not merely empty.

Final affected lanes (all from `codex-rs`; common guarded flags above):

| Command suffix after `just test` | Run / passed / failed / skipped | Exit | Raw log |
| --- | --- | --- | --- |
| `-p codex-app-server settings` | 34 / 34 / 0 / 1074 | 0 | `pf83-native-f05-54-settings-final2.log` |
| `-p codex-app-server turn_steer` | 5 / 5 / 0 / 1103 | 0 | `pf83-native-f05-54-turn-steer-final.log` |
| `-p codex-tui permission` | 94 / 94 / 0 / 4078 | 0 | `pf83-native-f05-54-permission-final.log` |
| `-p codex-core --lib 'session::'` | 345 / 345 / 0 / 2142 | 0 | `pf83-native-f05-54-core-session.log` |

**Final required lanes: 478 passed, zero failures; all ten added tests pass.** Historical nonzero lane: initial TUI permission run only, one failure, exit 100, exact name and measured startup correction above. Earlier settings attempts: 32/32 pass (`settings.log`), then 34/34 pass after adding settled restart cases (`settings-final.log`); both exit 0. Earlier steering: 5/5 pass, 1101 skipped, exit 0 (`turn-steer.log`). Focused startup diagnostic: 1/1 pass, 4171 skipped, exit 0. No compilation failure, ignored new test, automatic retry, or suppressed failure. Settings was rerun after the stronger absent-marker assertion; permission was rerun after the explicit startup baseline. Original logs remain intact.

Raw logs are retained locally under this return's directory; SHA-256 values follow. They contain 3828 generated lines / 207635 bytes in total, separately counted from authored source/evidence lines.

| Log | SHA-256 |
| --- | --- |
| `pf83-native-f05-54-core-session.log` | `e5ac09ca9139ac568acf6028a48d7a5296a48c84d8bc4d144c9e1880b26bada3` |
| `pf83-native-f05-54-f07-startup-control.log` | `8e07aa9e794bf8b8c0fab3aeafa3a5ce0e880f3914fdcb8ce4837c0fdbe09bdb` |
| `pf83-native-f05-54-permission-final.log` | `09a4266edff7f7b0364f3e8bc024ef26416d000dcb3c4ee19d55fec22a0273d7` |
| `pf83-native-f05-54-permission.log` | `5c893e3371d574018489fd895a8d3f97796ba8fc71fdb282d2dc4aaa148b498e` |
| `pf83-native-f05-54-prerequisites.log` | `e3b08d38bf49a8f0c673593f8410377c792d30d733f6c4b83cc8863161c2b8bf` |
| `pf83-native-f05-54-settings-final.log` | `4605d91d6e5a279fdecd45c7d5523df151fcc6ed9286ec3c3779d2aff8cfd19a` |
| `pf83-native-f05-54-settings-final2.log` | `05cc2a7d72ed00ced9090866bba18932f51395893d7353650cac0fd25a13ec25` |
| `pf83-native-f05-54-settings.log` | `a84a43a73d1a5b927701973fed7d6049de5beeacfca71cf2afbdb3f861be4c24` |
| `pf83-native-f05-54-turn-steer-final.log` | `6aa4fa2125148e6d305e33e9346713478f8d53b5a8f0c3b0567db548ade8787d` |
| `pf83-native-f05-54-turn-steer.log` | `e099106b7e54b460df6aa4f15b42379aaef37b9440f54f5315a0fc70174e09be` |

Final Rust source SHA-256: `thread_settings_update.rs` = `21424d50d1c4fde0b1624834007ef12fbf203796caf9c692af006450fa34c090`; `permission_confirmation_tests.rs` = `095b6eac8e6aad64e7cf851f522b3ef8b39958640e5de9656bdd323cb7a9bb52`. Git diff is additions only: 458 app-server test lines + 104 TUI test lines = **562 inside tests; zero non-test Rust lines**, zero deletions. Outside tests: this QA return record adds 88 authored Markdown lines; no other authored non-test file changed. All generated logs are under the allowed management-bootstrap directory and ignored by Git; they are not source changes. No path outside the writable source/evidence scope changed. `python3 -B docs/sprints/check.py` passed: 115 current / 127 archived.

## Corrections to the brief's readings

- The allocation document points to the frozen originals; it is not itself their independent source. Original file `/Volumes/CorbanuDrive/Corbanu/.codex-work/permission-transition.eocbhB/frozen-functional-cases.md` SHA-256 remains `c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726`. Allocation file SHA-256 remains `e53d9e0d0df49a39b824e5739ec406d31139d7cfce5563060022bd15e98f3c15`.
- F05 allows a clear, safe invalidation/replacement/deferral as well as explicit decision. These fixtures test this product's implemented retained-request path, not a universal requirement that the original request must survive every implementation.
- F06 itself permits interruption or continuation when truthfully disclosed; it does not mandate automatic continuation. The stronger no-automatic-interruption/replay assertion comes from the product specification quoted above.
- F07 assumes no ordering policy beyond truthful safe treatment. The backend currently accepts these ordered RPC requests; the TUI serializes conflicting requests with explicit refusal. Reverse response consumption and synthetic old completion delivery are distinct tests, not evidence that an external transport reordered a real completion.
- F09 is bidirectional and also covers retained completed work/context and understandable continuation. “Old denied/new one” describes only restricted-to-full. Both native directions were implemented; UI/context/wording gaps remain explicit.
- F10 is more specific than “recovery” and does not prescribe a persistence default. A genuine process restart is expressible natively and was built; known pending-application state plus UI recovery cannot be honestly manufactured by withholding a response.
- F11 original covers each exposed supported route, at least two if available, and supported switching. Its synthetic-substitute prohibition is explicit in the allocation. No actual route is qualified here.

No original case text or existing assertion was changed, no assertion weakened, no test ignored, no non-test Rust line changed, and no product finding suppressed. This source-informed native work does not establish why any other executor refused or whether its isolation was adequate.
