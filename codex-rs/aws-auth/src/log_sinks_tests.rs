//! #548: a `credential_process` helper's stderr and command line reach no
//! log sink: the trace-level log file, the `/feedback` buffer or the logs
//! database, installed as the binaries install them.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use aws_config::profile::profile_file::ProfileFileKind;
use codex_utils_absolute_path::AbsolutePathBuf;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

use super::*;

const STDERR_CANARY: &str = "fake-helper-stderr-canary-548-3f9a61";
const ARG_CANARY: &str = "fake-helper-arg-canary-548-c07e2d";

/// A profile whose `credential_process` prints [`STDERR_CANARY`] to stderr
/// and fails; its arguments carry [`ARG_CANARY`].
fn failing_helper_files(dir: &Path) -> ProfileFiles {
    let helper = dir.join("helper.sh");
    std::fs::write(
        &helper,
        format!("#!/bin/sh\necho \"helper failed: {STDERR_CANARY}\" >&2\nexit 3\n"),
    )
    .expect("write helper");
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o700))
        .expect("chmod helper");
    ProfileFiles::builder()
        .with_contents(
            ProfileFileKind::Config,
            format!(
                "[profile canary]\nregion = us-west-2\ncredential_process = {} --token {ARG_CANARY}\n",
                helper.display()
            ),
        )
        .with_contents(ProfileFileKind::Credentials, "")
        .build()
}

/// The error and every source as text, helper stderr included (the SDK
/// copies it into a source error).
fn raw_chain(error: &AwsAuthError) -> String {
    let mut text = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(error) = source {
        text.push_str(" | ");
        text.push_str(&error.to_string());
        source = error.source();
    }
    text
}

/// Runs the helper written by `failing_helper_files`. Retried while Linux
/// reports it busy: another test's child can briefly hold a just-written
/// file open (`sh` then says "Text file busy" on its stderr). Returns only
/// once the helper ran and printed the canary.
async fn load_failing_profile(files: &ProfileFiles) -> AwsAuthError {
    for _ in 0..40 {
        let error = load_named_profile(
            "canary",
            /*region_override*/ None,
            "bedrock-mantle",
            Some(files.clone()),
        )
        .await
        .expect_err("the helper fails");
        let chain = raw_chain(&error);
        if chain.contains(STDERR_CANARY) {
            return error;
        }
        assert!(
            chain.contains("Text file busy"),
            "unexpected failure: {chain}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("helper stayed busy");
}

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Buffer {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("buffer lock")).into_owned()
    }
}

impl std::io::Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("buffer lock").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn contains(bytes: &[u8], needle: &str) -> bool {
    bytes
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

/// Current-thread runtime: the sinks are installed with `set_default`, so
/// every task must log from this thread.
#[tokio::test(flavor = "current_thread")]
async fn credential_process_output_reaches_no_log_sink() {
    let dir = tempfile::tempdir().expect("tempdir");
    let files = failing_helper_files(dir.path());

    // Control: the SDK does log the helper's stderr (it redacts the
    // arguments itself; they are checked below all the same).
    let unguarded = Buffer::default();
    {
        let writer = unguarded.clone();
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .with_writer(move || writer.clone())
                .with_ansi(false)
                .with_filter(EnvFilter::new("trace")),
        );
        let _default = tracing::subscriber::set_default(subscriber);
        tracing::callsite::rebuild_interest_cache();
        load_failing_profile(&files).await;
    }
    let control = unguarded.text();
    assert!(
        control.contains(STDERR_CANARY),
        "control: the SDK should log the helper's stderr: {control}"
    );

    // The binaries' sinks, behind the guard.
    let state_home = tempfile::tempdir().expect("state home");
    let state_db = codex_state::StateRuntime::init(
        codex_state::SqliteConfig::new_for_testing(
            AbsolutePathBuf::try_from(state_home.path().to_path_buf()).expect("absolute"),
        ),
        "test-provider".to_string(),
    )
    .await
    .expect("state runtime");
    let log_db = codex_state::log_db::start(Arc::clone(&state_db));
    let feedback = codex_feedback::CodexFeedback::new();
    let file = Buffer::default();
    let file_writer = file.clone();
    let subscriber = codex_log_guard::guard(
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(move || file_writer.clone())
                    .with_ansi(false)
                    .with_filter(EnvFilter::new("trace,aws_config=trace")),
            )
            .with(feedback.logger_layer())
            .with(
                log_db
                    .clone()
                    .with_filter(codex_state::log_db::default_filter()),
            ),
    );
    let error = {
        let _default = tracing::subscriber::set_default(subscriber);
        tracing::callsite::rebuild_interest_cache();
        let error = load_failing_profile(&files).await;
        tracing::warn!(error = ?error, "bedrock auth failed: {}", error.detailed_message());
        tracing::info!("marker-548 logged");
        error
    };
    log_db.flush().await;

    // Corbanu's own messages say what failed, without the helper's output.
    for text in [error.detailed_message(), format!("{error:?}")] {
        assert!(text.contains("(helper output withheld)"), "{text}");
        assert!(!text.contains(STDERR_CANARY), "{text}");
    }

    let file_text = file.text();
    let feedback_bytes = feedback.snapshot(/*session_id*/ None).log_bytes().to_vec();
    let rows = state_db
        .query_logs(&codex_state::LogQuery::default())
        .await
        .expect("query logs");
    assert!(file_text.contains("marker-548"), "{file_text}");
    assert!(contains(&feedback_bytes, "marker-548"));
    assert!(rows.iter().any(|row| {
        row.message
            .as_deref()
            .is_some_and(|message| message.contains("marker-548"))
    }));
    drop(state_db);
    let mut db_files = Vec::new();
    for entry in walkdir::WalkDir::new(state_home.path()) {
        let entry = entry.expect("walk state home");
        if entry.file_type().is_file() {
            db_files.push((
                entry.path().to_path_buf(),
                std::fs::read(entry.path()).expect("read db file"),
            ));
        }
    }
    assert!(!db_files.is_empty());
    for canary in [STDERR_CANARY, ARG_CANARY] {
        assert!(!file_text.contains(canary), "leaked into the log file");
        assert!(
            !contains(&feedback_bytes, canary),
            "leaked into the feedback buffer"
        );
        for (path, bytes) in &db_files {
            assert!(!contains(bytes, canary), "leaked into {}", path.display());
        }
    }
}
