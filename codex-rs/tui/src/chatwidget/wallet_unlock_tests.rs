use super::*;
use crate::onboarding::provider_setup::DeferredProviderSetup;
use crate::onboarding::provider_setup::ProviderSetupAction;
use crate::onboarding::provider_setup::ProviderSetupEffect;
use crate::onboarding::provider_setup::ProviderSetupSession;
use pretty_assertions::assert_eq;

fn deferred_setup() -> DeferredProviderSetup {
    let mut session = ProviderSetupSession::from_statuses(&[]);
    session.dispatch(ProviderSetupAction::QueueCorbanu(true));
    session
        .dispatch(ProviderSetupAction::Done)
        .effects
        .into_iter()
        .find_map(|effect| match effect {
            ProviderSetupEffect::BeginDeferred(deferred) => Some(deferred),
            _ => None,
        })
        .expect("queued Corbanu setup should produce a deferred continuation")
}

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
        unlock_confirmation(UnlockPolicy::OneAction, /*expires_in_seconds*/ 300),
        "Wallet unlocked for one signing action; it locks after that attempt or in 5 minute(s) if unused."
    );
    assert_eq!(
        unlock_confirmation(
            UnlockPolicy::Timed {
                duration_seconds: 900,
            },
            /*expires_in_seconds*/ 900,
        ),
        "Wallet unlocked for 15 minute(s)."
    );
}

#[tokio::test]
async fn custom_unlock_guidance_is_complete_in_the_narrow_qualification_terminal() {
    let (mut chat, _tx, _event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.open_wallet_custom_unlock(
        /*validation_error*/ None,
        WalletUnlockContinuation::WalletMenu,
    );

    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, /*width*/ 69);
    assert!(rendered.contains("Custom wallet unlock"));
    assert!(rendered.contains("Whole minutes from 1 to 480"));
    assert!(rendered.contains("Signing access stays in this TUI and expires on schedule."));
}

#[tokio::test]
async fn custom_unlock_preserves_the_requested_continuation() {
    let (mut chat, _tx, mut event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.open_wallet_custom_unlock(
        /*validation_error*/ None,
        WalletUnlockContinuation::OpenPlans {
            mode: crate::chatwidget::wallet_menu::WalletPlanPurchaseMode::New,
        },
    );

    while event_rx.try_recv().is_ok() {}
    chat.handle_paste("30".to_string());
    chat.handle_key_event(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    ));

    assert!(matches!(
        event_rx.try_recv(),
        Ok(AppEvent::OpenWalletUnlock {
            policy: UnlockPolicy::Timed {
                duration_seconds: 1_800
            },
            continuation: WalletUnlockContinuation::OpenPlans {
                mode: crate::chatwidget::wallet_menu::WalletPlanPurchaseMode::New
            }
        })
    ));
}

#[tokio::test]
async fn invalid_custom_unlock_preserves_the_requested_continuation_for_retry() {
    let (mut chat, _tx, mut event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.open_wallet_custom_unlock(
        /*validation_error*/ None,
        WalletUnlockContinuation::OpenPlans {
            mode: crate::chatwidget::wallet_menu::WalletPlanPurchaseMode::New,
        },
    );

    while event_rx.try_recv().is_ok() {}
    chat.handle_paste("481".to_string());
    chat.handle_key_event(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    ));

    assert!(matches!(
        event_rx.try_recv(),
        Ok(AppEvent::OpenWalletCustomUnlock {
            validation_error: Some(_),
            continuation: WalletUnlockContinuation::OpenPlans {
                mode: crate::chatwidget::wallet_menu::WalletPlanPurchaseMode::New
            }
        })
    ));
}

#[tokio::test]
async fn failed_unlock_reopens_the_passcode_prompt_for_the_same_flow() {
    let (mut chat, _tx, _event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;

    chat.on_wallet_unlock_finished(
        UnlockPolicy::Timed {
            duration_seconds: 300,
        },
        WalletUnlockContinuation::OpenPlans {
            mode: crate::chatwidget::wallet_menu::WalletPlanPurchaseMode::New,
        },
        Err("incorrect passcode".to_string()),
    );

    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, /*width*/ 69);
    assert!(rendered.contains("Unlock wallet"));
    assert!(rendered.contains("Wallet passcode — masked"));
}

#[tokio::test]
async fn unlock_preflight_does_not_request_a_secret_when_no_wallet_exists() {
    let (mut chat, _tx, mut event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;

    chat.on_wallet_unlock_preflight_finished(
        UnlockPolicy::Timed {
            duration_seconds: 900,
        },
        WalletUnlockContinuation::WalletMenu,
        Ok(false),
    );

    let mut rendered_history = String::new();
    while let Ok(event) = event_rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            for line in cell.display_lines(/*width*/ 80) {
                for span in line.spans {
                    rendered_history.push_str(&span.content);
                }
                rendered_history.push('\n');
            }
        }
    }
    assert!(rendered_history.contains("no local wallet exists"));
    let popup = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, /*width*/ 69);
    assert!(!popup.contains("Wallet passcode — masked"));
}

#[tokio::test]
async fn escaping_deferred_unlock_cancels_the_exact_provider_setup() {
    let (mut chat, _tx, mut event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    let deferred = deferred_setup();

    chat.show_wallet_unlock_prompt(
        UnlockPolicy::OneAction,
        WalletUnlockContinuation::OpenCorbanuApi {
            deferred: Some(deferred.clone()),
        },
    );
    while event_rx.try_recv().is_ok() {}
    chat.handle_key_event(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::NONE,
    ));

    assert!(matches!(
        event_rx.try_recv(),
        Ok(AppEvent::DeferredCorbanuPlanCancelled { deferred: observed }) if observed == deferred
    ));
}

#[tokio::test]
async fn failed_deferred_unlock_preflight_cancels_instead_of_opening_an_ordinary_menu() {
    let (mut chat, _tx, mut event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    let deferred = deferred_setup();

    chat.on_wallet_unlock_preflight_finished(
        UnlockPolicy::OneAction,
        WalletUnlockContinuation::OpenCorbanuApi {
            deferred: Some(deferred.clone()),
        },
        Ok(false),
    );

    let events = std::iter::from_fn(|| event_rx.try_recv().ok()).collect::<Vec<_>>();
    assert!(events.iter().any(|event| matches!(
        event,
        AppEvent::DeferredCorbanuPlanCancelled { deferred: observed } if observed == &deferred
    )));
    assert_eq!(chat.bottom_pane.active_view_id(), None);
}
