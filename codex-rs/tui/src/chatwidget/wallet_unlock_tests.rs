use super::*;
use pretty_assertions::assert_eq;

#[test]
fn custom_unlock_accepts_only_whole_minutes_in_the_supported_range() {
    assert_eq!(
        parse_unlock_minutes(" 30 "),
        Ok(UnlockPolicy::Timed {
            duration_seconds: 1_800,
        })
    );
    assert!(parse_unlock_minutes("0").is_err());
    assert!(parse_unlock_minutes("481").is_err());
    assert!(parse_unlock_minutes("1.5").is_err());
}

#[test]
fn unlock_confirmation_distinguishes_single_action_from_timed_access() {
    assert_eq!(
        unlock_confirmation(UnlockPolicy::OneAction, 300),
        "Wallet unlocked for one signing action; it locks after that attempt or in 5 minute(s) if unused."
    );
    assert_eq!(
        unlock_confirmation(
            UnlockPolicy::Timed {
                duration_seconds: 900,
            },
            900,
        ),
        "Wallet unlocked for 15 minute(s)."
    );
}

#[tokio::test]
async fn custom_unlock_guidance_is_complete_in_the_narrow_qualification_terminal() {
    let (mut chat, _tx, _event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.open_wallet_custom_unlock(None);

    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 69);
    assert!(rendered.contains("Custom wallet unlock"));
    assert!(rendered.contains("Whole minutes from 1 to 480"));
    assert!(rendered.contains("Signing access stays in this TUI and expires on schedule."));
}
