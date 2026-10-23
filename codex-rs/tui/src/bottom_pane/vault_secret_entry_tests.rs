use std::sync::Arc;
use std::sync::Mutex;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use super::Field;
use super::VaultSecretEntryView;

use crate::bottom_pane::BottomPaneView;

fn char_event(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn enter_event() -> KeyEvent {
    KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
}

/// Captures the (label, secret) the view ultimately submits.
#[derive(Default, Clone)]
struct Capture {
    inner: Arc<Mutex<Option<(String, String)>>>,
}

impl Capture {
    fn callback(self) -> super::VaultSecretSubmitted {
        Box::new(move |label, secret| {
            *self.inner.lock().unwrap() = Some((label, secret));
        })
    }

    fn taken(&self) -> Option<(String, String)> {
        self.inner.lock().unwrap().take()
    }
}

#[test]
fn advances_from_label_to_secret_then_submits() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new(capture.clone().callback());

    // Type the label.
    for c in "ambient/prod".chars() {
        view.handle_key_event(char_event(c));
    }
    // Submitting the label must NOT fire the callback yet.
    view.handle_key_event(enter_event());
    assert!(capture.taken().is_none(), "label submit must not finalize");
    assert_eq!(view.field, Field::Secret);

    // Type the (masked) secret.
    for c in "sk-secret-123".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());

    assert_eq!(
        capture.taken(),
        Some(("ambient/prod".to_string(), "sk-secret-123".to_string())),
    );
    assert!(view.is_complete());
}

#[test]
fn empty_label_does_not_advance() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new(capture.clone().callback());
    view.handle_key_event(enter_event());
    assert_eq!(view.field, Field::Label);
    assert!(capture.taken().is_none());
    assert!(!view.is_complete());
}

#[test]
fn empty_secret_does_not_finalize() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new(capture.clone().callback());
    for c in "label".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());
    assert_eq!(view.field, Field::Secret);
    // Enter with empty secret stays on the secret field.
    view.handle_key_event(enter_event());
    assert_eq!(view.field, Field::Secret);
    assert!(capture.taken().is_none());
    assert!(!view.is_complete());
}

#[test]
fn escape_cancels_without_submitting() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new(capture.clone().callback());
    view.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(view.is_complete());
    assert!(
        capture.taken().is_none(),
        "cancel must not submit the secret"
    );
}

#[test]
fn label_is_trimmed_before_submission() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new(capture.clone().callback());
    for c in "  spaced-label  ".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());
    for c in "secret".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());
    assert_eq!(capture.taken().unwrap().0, "spaced-label");
}

#[test]
fn secret_is_preserved_byte_for_byte_without_trimming() {
    // Leading/trailing whitespace can be meaningful for PEM blobs, seed phrases, pasted keys.
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new(capture.clone().callback());
    for c in "label".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());
    for c in "  sk-with-padding  ".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());

    let (_label, secret) = capture.taken().expect("secret submitted");
    assert_eq!(secret, "  sk-with-padding  ", "secret must not be trimmed");
}

#[test]
fn all_whitespace_secret_is_rejected() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new(capture.clone().callback());
    for c in "label".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());
    // Whitespace-only secret must not finalize.
    view.handle_key_event(char_event(' '));
    view.handle_key_event(char_event(' '));
    view.handle_key_event(enter_event());
    assert_eq!(view.field, Field::Secret);
    assert!(capture.taken().is_none());
    assert!(!view.is_complete());
}

#[test]
fn fixed_secret_mode_submits_known_label_without_label_step() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new_fixed_secret(
        "provider/zai_api_key".to_string(),
        "Add Provider: Z.AI API Key".to_string(),
        "API key — masked".to_string(),
        "ZAI_API_KEY (masked)".to_string(),
        capture.clone().callback(),
    );
    assert_eq!(view.field, Field::Secret);

    for c in "zai-secret".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());

    assert_eq!(
        capture.taken(),
        Some(("provider/zai_api_key".to_string(), "zai-secret".to_string()))
    );
    assert!(view.is_complete());
}

#[test]
fn fixed_secret_cancel_callback_runs_once() {
    let cancellations = Arc::new(Mutex::new(0));
    let cancellations_for_callback = Arc::clone(&cancellations);
    let mut view = VaultSecretEntryView::new_fixed_secret_with_cancel(
        "oauth".to_string(),
        "OAuth".to_string(),
        "Authorization code — masked".to_string(),
        "Paste code".to_string(),
        Box::new(|_, _| {}),
        Box::new(move || {
            *cancellations_for_callback.lock().unwrap() += 1;
        }),
    );
    let escape = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

    view.handle_key_event(escape);
    view.handle_key_event(escape);

    assert_eq!(*cancellations.lock().unwrap(), 1);
    assert_eq!(
        view.completion(),
        Some(crate::bottom_pane::ViewCompletion::Cancelled)
    );
}

#[test]
fn confirmed_secret_submits_only_after_matching_second_entry() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new_confirmed_secret(
        "wallet-passcode".to_string(),
        "Protect wallet".to_string(),
        "Wallet passcode — masked".to_string(),
        "New passcode".to_string(),
        capture.clone().callback(),
    );

    for c in "same-passcode".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());
    assert_eq!(view.field, Field::SecretConfirm);
    assert!(capture.taken().is_none());

    for c in "same-passcode".chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());

    assert_eq!(
        capture.taken(),
        Some(("wallet-passcode".to_string(), "same-passcode".to_string()))
    );
    assert!(view.is_complete());
}

#[test]
fn confirmed_secret_mismatch_is_visible_and_next_matching_pair_succeeds() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new_confirmed_secret(
        "wallet-passcode".to_string(),
        "Protect wallet".to_string(),
        "Wallet passcode — masked".to_string(),
        "New passcode".to_string(),
        capture.clone().callback(),
    );

    for value in ["first-passcode", "different-passcode"] {
        for c in value.chars() {
            view.handle_key_event(char_event(c));
        }
        view.handle_key_event(enter_event());
    }

    assert_eq!(view.field, Field::Secret);
    assert_eq!(
        view.validation_error.as_deref(),
        Some("Values did not match. Enter the value twice again.")
    );
    assert!(capture.taken().is_none());
    assert!(!view.is_complete());

    for value in ["replacement", "replacement"] {
        for c in value.chars() {
            view.handle_key_event(char_event(c));
        }
        view.handle_key_event(enter_event());
    }

    assert_eq!(
        capture.taken(),
        Some(("wallet-passcode".to_string(), "replacement".to_string()))
    );
    assert!(view.is_complete());
}

const SHOWN_LOCK_ERROR: &str = "Failed to add vault credential \"moonshot_api_key\": failed to open vault lock /Users/example/.corbanu/secrets/.vault.lock";

fn render_view(view: &VaultSecretEntryView, width: u16) -> String {
    use crate::render::renderable::Renderable;
    let height = view.desired_height(width);
    let area = ratatui::layout::Rect::new(0, 0, width, height);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    view.render(area, &mut buf);
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn enter_label(view: &mut VaultSecretEntryView, label: &str) {
    for c in label.chars() {
        view.handle_key_event(char_event(c));
    }
    view.handle_key_event(enter_event());
}

/// #855: the add failed with this error, the person copied it, and then
/// pasted it into the masked field of the next add instead of the key.
#[test]
fn a_pasted_vault_error_is_refused_and_the_entry_stays_open() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new(capture.clone().callback())
        .rejecting_shown_errors(vec![SHOWN_LOCK_ERROR.to_string()]);
    enter_label(&mut view, "kimi_api");

    assert!(view.handle_paste(SHOWN_LOCK_ERROR.to_string()));
    view.handle_key_event(enter_event());

    assert!(
        capture.taken().is_none(),
        "the error text must not be submitted"
    );
    assert!(!view.is_complete());
    assert_eq!(view.field, Field::Secret);
    assert_eq!(view.textarea.text(), "");
    assert_eq!(
        view.validation_error.as_deref(),
        Some(super::ERROR_TEXT_REJECTED)
    );
    insta::assert_snapshot!(
        "vault_secret_entry_refuses_pasted_error_text",
        render_view(&view, /*width*/ 96)
    );

    assert!(view.handle_paste("sk-lock-test-value-0001".to_string()));
    assert_eq!(view.validation_error, None);
    view.handle_key_event(enter_event());
    assert_eq!(
        capture.taken(),
        Some((
            "kimi_api".to_string(),
            "sk-lock-test-value-0001".to_string()
        ))
    );
}

/// The text of a vault error is refused even when it was not shown in this
/// session (another window, an earlier run), and in every masked entry.
#[test]
fn vault_error_text_is_refused_without_a_shown_error() {
    let capture = Capture::default();
    let mut view = VaultSecretEntryView::new_fixed_secret(
        "provider/moonshot_api_key".to_string(),
        "Add Moonshot".to_string(),
        "API key — masked".to_string(),
        "MOONSHOT_API_KEY".to_string(),
        capture.clone().callback(),
    );
    // As copied from a narrow terminal: with the bullet and line breaks.
    let wrapped = SHOWN_LOCK_ERROR.replace(" vault ", "\nvault\n");
    assert!(view.handle_paste(format!("■ {wrapped}\n")));
    view.handle_key_event(enter_event());

    assert!(capture.taken().is_none());
    assert_eq!(
        view.validation_error.as_deref(),
        Some(super::ERROR_TEXT_REJECTED)
    );
}

/// Any error shown in the transcript is refused, whole or in part; short or
/// unrelated values are not.
#[test]
fn shown_error_text_is_refused_whole_or_in_part() {
    let shown =
        "Provider request failed: 401 Invalid Authentication from https://api.example.com/v1/models"
            .to_string();
    for (value, refused) in [
        (shown.as_str(), true),
        ("401 Invalid Authentication from https://api", true),
        (
            "Provider request failed: 401 Invalid Authentication from https://api.example.com/v1/models and more",
            true,
        ),
        // Copied from a narrow terminal: the line breaks of the wrapped error.
        (
            "provider request failed: 401 Invalid\nAuthentication from https://api.example.com/v1/models",
            true,
        ),
        ("401", false),
        ("sk-lock-test-value-0001", false),
        // A token the error happened to quote is not refused.
        ("https://api.example.com/v1/models", false),
    ] {
        let capture = Capture::default();
        let mut view = VaultSecretEntryView::new(capture.clone().callback())
            .rejecting_shown_errors(vec![shown.clone()]);
        enter_label(&mut view, "label");
        assert!(view.handle_paste(value.to_string()));
        view.handle_key_event(enter_event());
        assert_eq!(capture.taken().is_none(), refused, "{value}");
    }
}
