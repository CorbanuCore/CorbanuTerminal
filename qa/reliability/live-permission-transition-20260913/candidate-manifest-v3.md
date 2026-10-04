# PF83 frozen v3 candidate

Base `005cc644f59b1e762e5497b329e106c67925d4ed`; branch `fix/live-permission-transition-20260913`.
Patch SHA256 `b5031c64617512dcab488c8ad6c96690a6d5ad9d47b9957a2744d1bc13408142`. V1 patch and manifest hashes verified unchanged.
Final1 support: 353/353 passed; log SHA256 `64b9fecb83373ba0717937c9a3423f56422469d8c00abf823c6239c3e44cad4e`.
Source-only candidate: 34 paths; all within literal PF-83-S01 scope.
No staging, commit, or source changes performed by this generator.

Changed lines: 639 non-test + 1043 test = 1682 hand-authored Rust.
Mechanical schema text: 42 changed lines plus two compressed fixtures.
Snapshot text: 32 changed lines. API README counted separately.
Counts include additions/deletions/new leaves; test registration and existing
native-client test hunks are separated from non-test source.

| Path | Status | Bytes | SHA256 |
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
| `codex-rs/tui/src/app.rs` | modified | 97121 | `b333f71ecc2d6c2a1f64f02c526685cb1687f9e92a174932674489d0007d9273` |
| `codex-rs/tui/src/app/config_persistence.rs` | modified | 74010 | `27189b296bd0490005a7520945f659f3d4bd89956617687c7b6b9303e03e0dc2` |
| `codex-rs/tui/src/app/event_dispatch.rs` | modified | 298164 | `85dc710ce3cc159c5cff8afda4a605ca1ffbdbdabcb01c22a3282cb2de1aa0f1` |
| `codex-rs/tui/src/app/permission_confirmation.rs` | new | 7912 | `7ca74c1e2c52a71babef805d34c9a17cf29e85b9fdd63b454a62ec3ce6d6fdbe` |
| `codex-rs/tui/src/app/permission_confirmation_tests.rs` | new | 18514 | `9ace97c844aa52554279610a310429346389314d8f0af3868b196998c7911ff4` |
| `codex-rs/tui/src/app/snapshots/codex_tui__app__permission_confirmation__tests__permission_confirmation_outcomes_are_explicit.snap` | new | 946 | `dfe517ad4732c5b55c269c32f73fbfa993b2714e8a2cdcdf7add1bb0275a5ebe` |
| `codex-rs/tui/src/app/snapshots/codex_tui__app__permission_confirmation__tests__permission_confirmation_superseded.snap` | new | 264 | `3b97e1aab6ce65fe6a24841664109a31f263112ee9806e742e354efe01cb3736` |
| `codex-rs/tui/src/app/test_support.rs` | modified | 6753 | `8008a2d03d0c1df52830454428cc74524b6d9beb4ddf87e2dbe3f32f5f8e0633` |
| `codex-rs/tui/src/app/tests.rs` | modified | 557664 | `98346d17ee885b91034337888b9ed7a75525bc31d97b22e98eec95189340c9eb` |
| `codex-rs/tui/src/app/thread_routing.rs` | modified | 97661 | `3ca8cd25a056de3ce5363c7ecc986eb2365ecc67f54af81068b7ebb24d2f1451` |
| `codex-rs/tui/src/app_event.rs` | modified | 75251 | `230ccdcd35d93e20c29079752787c0f08787011bacc4e0e14752d7bddf2a9bd8` |
| `codex-rs/tui/src/app_server_session.rs` | modified | 142563 | `b2c4877e3dcaf216de0e8dfec684c1e9243f4b5943c3d376c559b9830157cfcf` |
| `codex-rs/tui/src/chatwidget/input_restore.rs` | modified | 21416 | `c4665f354a1d6afb1190c1299753cae8203a2eba19cbbebb1c1ac55d31752d76` |
| `codex-rs/tui/src/chatwidget/permission_popups.rs` | modified | 19008 | `939a4f0ec21feb4af756211f38ec3264892cce4a7143fe6e79a41597bbfa8e7e` |
| `codex-rs/tui/src/chatwidget/settings.rs` | modified | 32677 | `16d2b8ec609c17cadd5ad4de2a761dac728c6d07286a11bfcd9eda36e9ac46e2` |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_after_mode_switch.snap` | deleted | — | `base version removed` |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_full_access_to_default.snap` | deleted | — | `base version removed` |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__permissions_selection_history_full_access_to_default@windows.snap` | deleted | — | `base version removed` |
| `codex-rs/tui/src/chatwidget/tests/permissions.rs` | modified | 48790 | `81e3b05cefe717ff939189b96a4368f685417cedd986dc3d5e2e7ab4d83340d3` |

Cycle 2 changes relative to v2 (all other candidate file hashes unchanged):

- `codex-rs/tui/src/app_server_session.rs`
