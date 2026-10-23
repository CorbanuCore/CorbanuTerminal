//! #855: a `/vault` add that fails stores nothing and shows why, and the text
//! of the error it showed is refused when pasted into the next add.

use super::*;
use pretty_assertions::assert_eq;

const VALUE: &str = "lock-test-value-0001";

async fn next_vault_add_result(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
) -> (String, Result<(), String>) {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            match rx.recv().await {
                Some(AppEvent::VaultCredentialAdded { label, result }) => break (label, result),
                Some(_) => continue,
                None => panic!("app event channel closed before the vault add finished"),
            }
        }
    })
    .await
    .expect("the vault add finished")
}

fn type_text(chat: &mut ChatWidget, text: &str) {
    for character in text.chars() {
        chat.handle_key_event(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
    }
}

#[tokio::test]
async fn a_failed_vault_add_stores_nothing_and_shows_the_error() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let home = tempdir().expect("tempdir");
    chat.config.codex_home =
        codex_utils_absolute_path::AbsolutePathBuf::from_absolute_path(home.path())
            .expect("temp dir is absolute");
    // The lock file cannot be opened (#855 saw EPERM from an unreadable home).
    let lock = home.path().join("secrets").join(".vault.lock");
    std::fs::create_dir_all(&lock).expect("a directory where the lock file belongs");

    // `add_vault_credential` uses the default (OS keyring) vault. It cannot
    // reach the keyring here: the lock open fails before any key is read, and
    // `just test` also sets CORBANU_TEST_NO_NATIVE_KEYRING.
    chat.add_vault_credential(
        "moonshot_api_key".to_string(),
        crate::app_event::VaultSecret::new(VALUE.to_string()),
    );
    let (label, result) = next_vault_add_result(&mut rx).await;
    assert_eq!(label, "moonshot_api_key");
    let error = result.expect_err("the add must fail");
    assert!(
        error.starts_with(&format!("failed to open vault lock {}: ", lock.display())),
        "{error}"
    );
    chat.on_vault_credential_added(label, Err(error));

    let shown = drain_insert_history(&mut rx)
        .iter()
        .map(|lines| lines_to_single_string(lines))
        .collect::<String>();
    let shown_flat = shown.replace('\n', "");
    assert!(
        shown_flat.contains(
            "Failed to add vault credential \"moonshot_api_key\": failed to open vault lock"
        ),
        "{shown}"
    );
    assert!(!shown.contains(VALUE));
    assert_eq!(
        std::fs::read_dir(home.path().join("secrets"))
            .expect("secrets dir")
            .map(|entry| entry.expect("entry").file_name())
            .collect::<Vec<_>>(),
        vec![std::ffi::OsString::from(".vault.lock")],
        "nothing but the lock may exist after a failed add"
    );
}

#[tokio::test]
async fn the_vault_add_error_state_snapshot() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.on_vault_credential_added(
        "moonshot_api_key".to_string(),
        Err("failed to open vault lock /Users/example/.corbanu/secrets/.vault.lock: Operation not permitted (os error 1)".to_string()),
    );
    let shown = drain_insert_history(&mut rx)
        .iter()
        .map(|lines| lines_to_single_string(lines))
        .collect::<String>();
    assert_chatwidget_snapshot!("vault_add_lock_failure_error", shown);
}

#[tokio::test]
async fn the_next_add_refuses_the_error_text_it_just_showed() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.add_error_message(
        "Failed to add moonshot credential: request refused by api.example.com (account locked)"
            .to_string(),
    );
    let _ = drain_insert_history(&mut rx);
    let shown = chat
        .shown_error_texts()
        .last()
        .cloned()
        .expect("the error is remembered");

    chat.open_vault_credential_add();
    type_text(&mut chat, "kimi_api");
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    chat.handle_paste(shown);
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));

    assert_matches!(rx.try_recv(), Err(TryRecvError::Empty));
    assert!(chat.bottom_pane.has_active_view(), "the entry stays open");

    chat.handle_paste(VALUE.to_string());
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    match rx.try_recv() {
        Ok(AppEvent::VaultCredentialAddRequested { label, secret }) => {
            assert_eq!(label, "kimi_api");
            assert_eq!(secret.into_inner(), VALUE);
        }
        other => panic!("expected the vault add request, got {other:?}"),
    }
}
