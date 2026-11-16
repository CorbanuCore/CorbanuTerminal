use super::*;
use crate::chatwidget::tests::assert_app_input_handoff_cancels_burst;

#[tokio::test]
async fn app_input_handoffs_cancel_delayed_enter_before_focus_returns() {
    assert_app_input_handoff_cancels_burst(
        |app| {
            app.overlay = Some(Overlay::new_static_with_lines(
                Vec::new(),
                "Other input target".into(),
                app.keymap.pager.clone(),
            ));
        },
        |app| app.overlay = None,
    )
    .await;
    assert_app_input_handoff_cancels_burst(
        |app| app.reconnect.offline = true,
        |app| app.reconnect.offline = false,
    )
    .await;
}
