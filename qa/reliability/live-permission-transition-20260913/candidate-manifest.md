# PF83 frozen candidate source manifest

Base: `005cc644f59b1e762e5497b329e106c67925d4ed`. Branch: `fix/live-permission-transition-20260913`. No commit or stage operation.

Patch SHA256: `e36396dcd62ad8daf59832302ea69f10e24770ddab3d44ce1b99122ed69cf96b`.

Final8: 350/350 passed; nextest `778753ff-3655-43dc-a1e3-11f626d68064`. Generated after completion.

Tracked diff plus every untracked `codex-rs` file; excludes parent planning and `.codex-work`. Deleted snapshots remain recoverable from base and patch.

| Path | Status | Current bytes | SHA256 |
| --- | --- | ---: | --- |
| `codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json` | modified | 614688 | `d3da6af9a26270a43f61bff31f67c56f9d75b68bafc9740d01c667c1d8f7f647` |
| `codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.v2.schemas.json` | modified | 521244 | `8d69da934937b952b2a27ceb1662edf966b56b6b303045f3c14d29f1a8c57efe` |
| `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst` | modified | 135701 | `e208b950ecf381512016e4bc8fd54d8bd46e020565cd7beacd9e241d5c71ce2c` |
| `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-stable.json.zst` | modified | 133006 | `b348764d7c34c6e5adc4760f073a00a99a63621dddf65206ac126b50b714a3a2` |
| `codex-rs/app-server-protocol/schema/typescript/v2/ThreadSettingsUpdateOutcome.ts` | new | 220 | `9b2e5a04bcacbbf7511cc2ebcf20c2e25aeae20e2a1c6a26c173aba28acb9a1e` |
| `codex-rs/app-server-protocol/schema/typescript/v2/index.ts` | modified | 39931 | `de4d07da1d7839f188617cf5e5d96f71c2b0508c7f106614a85887476f88bff6` |
| `codex-rs/app-server-protocol/src/protocol/v2/thread.rs` | modified | 69504 | `0942f01f3b5987ef7c4021d8517613be8f41367ab63135b7ccaaceb0aee9eb53` |
| `codex-rs/app-server/README.md` | modified | 161280 | `30739b13e5ceb6906f241f179b5d9d466ee97c60fbae5d02ca6d41973e6ea2e1` |
| `codex-rs/app-server/src/bespoke_event_handling.rs` | modified | 161431 | `1e4cd026370957358b8a0d2fed73128a9cf6d0aa622cd31bb12d303ea9675f6b` |
| `codex-rs/app-server/src/lib.rs` | modified | 56814 | `9993e42831e728b06449f83da0d3315c3a8622de7c7d2e1dc5e9b978977d746d` |
| `codex-rs/app-server/src/request_processors/turn_processor.rs` | modified | 66006 | `b13d6dfd206e0580c3f88aa385830caaf0874b9a76303f819290fa8288c5cd41` |
| `codex-rs/app-server/src/settings_confirmation.rs` | new | 2111 | `d5e9fdec936ffc4b8f146cb441200a50c7d6907953f04518b4b8969fa16879c1` |
| `codex-rs/app-server/src/settings_confirmation_tests.rs` | new | 5544 | `7de93a6483ca1adbbf5b02e25125ecc80d2de5106973a31e7c15301e7adeefa9` |
| `codex-rs/app-server/src/thread_state.rs` | modified | 23546 | `d2be1fea01470835a1fd69a1c88b4aacbe5a9d09cdadde20b824b70852f12d96` |
| `codex-rs/app-server/tests/suite/v2/thread_settings_update.rs` | modified | 18157 | `203751868a907f67e6e25e102f61da40fcbf8574658f56b433d7bfcabf4fd9f8` |
| `codex-rs/tui/src/app.rs` | modified | 97132 | `57eb391e9e8cce669d8df3ac8227ca4f434d6aa63a54dc7bf50942e8375ed95f` |
| `codex-rs/tui/src/app/config_persistence.rs` | modified | 74010 | `27189b296bd0490005a7520945f659f3d4bd89956617687c7b6b9303e03e0dc2` |
| `codex-rs/tui/src/app/event_dispatch.rs` | modified | 298164 | `85dc710ce3cc159c5cff8afda4a605ca1ffbdbdabcb01c22a3282cb2de1aa0f1` |
| `codex-rs/tui/src/app/permission_confirmation.rs` | new | 3821 | `b0f74722fbf504d6a7760f110e330b59440fe7b32a42dc51606678d58e2e4fef` |
| `codex-rs/tui/src/app/permission_confirmation_tests.rs` | new | 6364 | `87a99a211b6bee62e9f48082d7a03866cddfe3008ce2766203b8bb6e90cda4d3` |
| `codex-rs/tui/src/app/snapshots/codex_tui__app__permission_confirmation__tests__permission_confirmation_outcomes_are_explicit.snap` | new | 946 | `dfe517ad4732c5b55c269c32f73fbfa993b2714e8a2cdcdf7add1bb0275a5ebe` |
| `codex-rs/tui/src/app/test_support.rs` | modified | 6344 | `88c7b03470689e8c20e1d6e63803ead9acd9abfcf82d8bbcf1cfb2ac918f137b` |
| `codex-rs/tui/src/app/tests.rs` | modified | 557697 | `85a64ef0f0de59add89d4028e1cc0af25559755f731b2a0e3c62f353af9f857a` |
| `codex-rs/tui/src/app/thread_routing.rs` | modified | 97385 | `6b545e6d7a026d2d6b7d69387411df0085043fa8a3aeef879c7b1f8839ad7922` |
| `codex-rs/tui/src/app_event.rs` | modified | 75251 | `230ccdcd35d93e20c29079752787c0f08787011bacc4e0e14752d7bddf2a9bd8` |
| `codex-rs/tui/src/app_server_session.rs` | modified | 141160 | `f18592f3ac18124223139a3328f04f6d37a69135e968865e09c3045bdfad8e89` |
| `codex-rs/tui/src/chatwidget/permission_popups.rs` | modified | 19008 | `939a4f0ec21feb4af756211f38ec3264892cce4a7143fe6e79a41597bbfa8e7e` |
| `codex-rs/tui/src/chatwidget/settings.rs` | modified | 32677 | `16d2b8ec609c17cadd5ad4de2a761dac728c6d07286a11bfcd9eda36e9ac46e2` |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_after_mode_switch.snap` | deleted | — | base version removed |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_full_access_to_default.snap` | deleted | — | base version removed |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_full_access_to_default@windows.snap` | deleted | — | base version removed |
| `codex-rs/tui/src/chatwidget/tests/permissions.rs` | modified | 47598 | `e0ca90bf12f67308019f4a62af05d2a3a4b10a454033ee3b48b050dd316b629c` |

Total candidate paths: 32. All literal paths audited against sprint scope.
