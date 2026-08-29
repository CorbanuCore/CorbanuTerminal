use std::io::Write;
use std::process::Command;
use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;
use codex_vault::AddCredential;
use codex_vault::CredentialType;
use codex_vault::ScopedCredentialError;
use codex_vault::Vault;
use codex_vault::VaultCredentialRef;
use pretty_assertions::assert_eq;
use serde_json::json;

// A fixed, secret-free policy fixture avoids adding a TUI -> policy dependency.
// Vault validates the grant digest and complete request before the callback.
fn reference() -> VaultCredentialRef {
    let actors = json!([{"kind":"human","id":"human:owner"}, {"kind":"agent","id":"agent:root"}]);
    let resource = json!({"kind":"vault_credential","id":"provider.openai"});
    let context = json!({"session_id":"session:panic","task_id":"task:panic","purpose":"model-inference","operation":"responses.create"});
    let mut authorization_context = context.clone();
    authorization_context["now_unix_seconds"] = json!(100);
    authorization_context["destination"] = json!("https://api.openai.com:443");
    let request = serde_json::from_value(json!({
        "schema_version":1,
        "authorization":{"schema_version":1,"subject":actors,"resource":resource,"action":"use","context":authorization_context},
        "grant":{
            "schema_version":2,"grant_id":"bccd08a7551c23777b106d7ff2dc8ba17674c155a8df29c05900935cb3b490e8",
            "issuer":{"kind":"human","id":"human:owner"},"actor_chain":actors,
            "scope":{"resource":resource,"actions":["use"],"context":context,"destination":"https://api.openai.com:443"},
            "issued_at_unix_seconds":90,"expires_at_unix_seconds":200,"nonce":"panic-test-grant"
        },
        "credential":{"label":"provider.openai","scope":"responses.create"},
        "method":"post","destination":{"transport":"https","host":"api.openai.com","port":443},
        "path":"/v1/responses","issued_at_unix_seconds":100,"expires_at_unix_seconds":180,"revocation_generation":0
    })).expect("policy fixture");
    VaultCredentialRef::from_authorized(
        serde_json::from_value(json!("a".repeat(64))).expect("capability id"),
        request,
    )
    .expect("validated credential reference")
}

#[test]
fn production_panic_hook_does_not_log_scoped_credentials() {
    const CHILD: &str = "CORBANU_PF13_TUI_PANIC_CHILD";
    const SECRET: &str = "sk-synthetic-production-panic-canary";
    if let Some(order) = std::env::var_os(CHILD) {
        let credential = reference();
        let directory = tempfile::tempdir().expect("isolated vault");
        let vault = Vault::new_with_keyring_store(
            directory.path().to_path_buf(),
            Arc::new(MockKeyringStore::default()),
        );
        vault
            .add(AddCredential {
                label: "provider.openai".to_string(),
                credential_type: CredentialType::ApiKey,
                provider: None,
                notes: None,
                revocation_notes: None,
                secret: SECRET.to_string(),
            })
            .expect("encrypted synthetic credential");
        if order == "vault-first" {
            vault
                .with_scoped_credential(&credential, 110, &Default::default(), |_| Ok(()))
                .expect("initialize vault hook first");
        }
        color_eyre::install().expect("production previous hook");
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(std::io::stderr)
            .try_init()
            .expect("production log capture");
        super::install_panic_hook();
        super::tui::set_panic_hook();
        let error = vault
            .with_scoped_credential(&credential, 110, &Default::default(), |secret| {
                panic!("callback secret: {secret}")
            })
            .expect_err("contained callback panic");
        assert_eq!(error, ScopedCredentialError::CallbackPanicked);
        assert!(!codex_vault::scoped_credential_callback_active());
        vault
            .with_scoped_credential(&credential, 110, &Default::default(), |_| Ok(()))
            .expect("recovery after panic");
        assert!(std::panic::catch_unwind(|| panic!("ordinary-tui-panic-visible")).is_err());
        writeln!(std::io::stdout(), "SCOPED_PANIC_RECOVERED").expect("subprocess checkpoint");
        return;
    }
    for order in ["vault-first", "tui-first"] {
        let output = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "credential_panic_tests::production_panic_hook_does_not_log_scoped_credentials",
                "--nocapture",
            ])
            .env(CHILD, order)
            .output()
            .expect("production panic-hook subprocess");
        assert!(
            output.status.success(),
            "production hook child must complete"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stdout.contains("SCOPED_PANIC_RECOVERED"));
        assert!(!stdout.contains(SECRET) && !stderr.contains(SECRET));
        assert!(stderr.contains("ordinary-tui-panic-visible"));
        assert!(stderr.contains("panic:"));
    }
}
