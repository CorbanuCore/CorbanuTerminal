use std::fs;
use std::fs::OpenOptions;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use pretty_assertions::assert_eq;
use tempfile::tempdir;

use super::*;

fn candidate() -> TelegramChatCandidate {
    TelegramChatCandidate {
        chat_id: 42,
        actor_user_id: 42,
        display_name: "Alice".to_string(),
        chat_kind: "private".to_string(),
    }
}

fn defaults(workspace: &Path) -> TelegramConnectionDefaults {
    TelegramConnectionDefaults {
        model: Some("k3".to_string()),
        cwd: workspace.to_path_buf(),
        approval_policy: "on-request".to_string(),
        sandbox_mode: "workspace-write".to_string(),
    }
}

#[test]
fn telegram_config_write_preserves_unrelated_settings_and_authorizes_exact_identity() {
    let home = tempdir().expect("home");
    fs::write(
        home.path().join("config.toml"),
        "model = \"gpt-5.6-sol\"\n\n[telegram]\nmax_attachment_bytes = 12345\n",
    )
    .expect("seed config");

    write_telegram_config(home.path(), Some((&candidate(), &defaults(home.path()))))
        .expect("write Telegram config");

    let config = read_config(home.path()).expect("read config");
    let telegram = config["telegram"].as_table().expect("telegram table");
    assert_eq!(config["model"].as_str(), Some("gpt-5.6-sol"));
    assert_eq!(telegram["max_attachment_bytes"].as_integer(), Some(12345));
    assert_eq!(telegram["enabled"].as_bool(), Some(true));
    assert_eq!(telegram["allowed_chat_ids"][0].as_integer(), Some(42));
    assert_eq!(telegram["allowed_user_ids"][0].as_integer(), Some(42));
    assert_eq!(telegram["default_model"].as_str(), Some("k3"));
    assert_eq!(telegram["approval_policy"].as_str(), Some("on-request"));
    assert_eq!(telegram["sandbox_mode"].as_str(), Some("workspace-write"));
}

#[test]
fn telegram_disconnect_removes_only_telegram_configuration() {
    let home = tempdir().expect("home");
    fs::write(
        home.path().join("config.toml"),
        "model = \"k3\"\n\n[telegram]\nenabled = true\nallowed_chat_ids = [42]\n",
    )
    .expect("seed config");

    write_telegram_config(home.path(), /*connection*/ None).expect("remove Telegram config");

    let config = read_config(home.path()).expect("read config");
    assert_eq!(config["model"].as_str(), Some("k3"));
    assert_eq!(config.get("telegram"), None);
}

#[test]
fn connector_operations_wait_for_another_process_instead_of_racing() {
    let home = tempdir().expect("home");
    let telegram_dir = home.path().join("telegram");
    fs::create_dir_all(&telegram_dir).expect("telegram dir");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(telegram_dir.join("connector-operation.lock"))
        .expect("lock file");
    lock.try_lock().expect("hold operation lock");
    let ran = Arc::new(AtomicBool::new(false));
    let thread_home = home.path().to_path_buf();
    let thread_ran = Arc::clone(&ran);
    let operation = std::thread::spawn(move || {
        with_operation_lock(&thread_home, || {
            thread_ran.store(true, Ordering::SeqCst);
            Ok(())
        })
    });

    std::thread::sleep(Duration::from_millis(100));
    assert!(!ran.load(Ordering::SeqCst));
    drop(lock);

    operation
        .join()
        .expect("operation thread")
        .expect("operation result");
    assert!(ran.load(Ordering::SeqCst));
}

#[test]
fn oversized_connector_log_is_rotated_before_the_next_start() {
    let home = tempdir().expect("home");
    let log = home.path().join("connector.log");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&log)
        .expect("log");
    file.set_len(CONNECTOR_LOG_MAX_BYTES + 1)
        .expect("grow sparse log");
    drop(file);

    rotate_connector_log(&log).expect("rotate");

    assert!(!log.exists());
    assert_eq!(
        fs::metadata(log.with_extension("log.previous"))
            .expect("previous log")
            .len(),
        CONNECTOR_LOG_MAX_BYTES + 1
    );
}

#[test]
fn startup_recovery_is_a_noop_until_a_bot_is_fully_configured() {
    let home = tempdir().expect("home");

    ensure_connector(home.path()).expect("unconfigured startup recovery");

    assert!(!runtime_path(home.path()).exists());
    assert!(
        !home.path().join("secrets").exists(),
        "unconfigured Telegram startup must not open the shared vault"
    );
}
