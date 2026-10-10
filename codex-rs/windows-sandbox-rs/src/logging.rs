use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use codex_log_guard::redact_credentials;
use codex_log_guard::redact_credentials_bytes;
use codex_utils_string::take_bytes_at_char_boundary;
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;

const LOG_COMMAND_PREVIEW_LIMIT: usize = 200;
pub const LOG_FILE_PREFIX: &str = "sandbox";
pub const LOG_FILE_SUFFIX: &str = "log";
pub const MAX_LOG_FILES: usize = 90;

fn exe_label() -> &'static str {
    static LABEL: OnceLock<String> = OnceLock::new();
    LABEL.get_or_init(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "proc".to_string())
    })
}

/// The command line as logged: credentials redacted (#398) before
/// truncation, so a cut can't leave part of a value unrecognised.
fn preview(command: &[String]) -> String {
    let joined = command.join(" ");
    let redacted = redact_credentials(&joined);
    take_bytes_at_char_boundary(&redacted, LOG_COMMAND_PREVIEW_LIMIT).to_string()
}

pub fn log_file_path_for_utc_date(base_dir: &Path, date: chrono::NaiveDate) -> PathBuf {
    base_dir.join(format!(
        "{LOG_FILE_PREFIX}.{}.{}",
        date.format("%Y-%m-%d"),
        LOG_FILE_SUFFIX
    ))
}

pub fn current_log_file_path(base_dir: &Path) -> PathBuf {
    log_file_path_for_utc_date(base_dir, chrono::Utc::now().date_naive())
}

pub fn current_log_file_path_for_codex_home(codex_home: &Path) -> PathBuf {
    current_log_file_path(&crate::sandbox_dir(codex_home))
}

/// The daily sandbox log. Every line passes through the credential
/// redaction of the `tracing` sinks (#398): lines carry command lines and
/// error text.
pub fn log_writer(base_dir: &Path) -> Option<RedactedLogWriter> {
    if !base_dir.is_dir() {
        return None;
    }

    RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(LOG_FILE_PREFIX)
        .filename_suffix(LOG_FILE_SUFFIX)
        .max_log_files(MAX_LOG_FILES)
        .build(base_dir)
        .ok()
        .map(|inner| RedactedLogWriter {
            inner,
            pending: Vec::new(),
        })
}

/// Buffers writes into whole lines and redacts each before writing it, so
/// `writeln!` pieces can't split a credential from its name. A partial
/// last line is written on flush or drop.
pub struct RedactedLogWriter {
    inner: RollingFileAppender,
    pending: Vec<u8>,
}

impl RedactedLogWriter {
    fn write_redacted(&mut self, line: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(&redact_credentials_bytes(line))
    }
}

impl Write for RedactedLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.pending.extend_from_slice(buf);
        if let Some(end) = self.pending.iter().rposition(|byte| *byte == b'\n') {
            let lines: Vec<u8> = self.pending.drain(..=end).collect();
            self.write_redacted(&lines)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.pending.is_empty() {
            let partial = std::mem::take(&mut self.pending);
            self.write_redacted(&partial)?;
        }
        self.inner.flush()
    }
}

impl Drop for RedactedLogWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

fn append_line(line: &str, base_dir: Option<&Path>) {
    if let Some(dir) = base_dir
        && let Some(mut f) = log_writer(dir)
    {
        let _ = writeln!(f, "{line}");
    }
}

pub fn log_start(command: &[String], base_dir: Option<&Path>) {
    let p = preview(command);
    log_note(&format!("START: {p}"), base_dir);
}

pub fn log_success(command: &[String], base_dir: Option<&Path>) {
    let p = preview(command);
    log_note(&format!("SUCCESS: {p}"), base_dir);
}

pub fn log_failure(command: &[String], detail: &str, base_dir: Option<&Path>) {
    let p = preview(command);
    log_note(&format!("FAILURE: {p} ({detail})"), base_dir);
}

// Debug logging helper. Emits only when SBX_DEBUG=1 to avoid noisy logs.
pub fn debug_log(msg: &str, base_dir: Option<&Path>) {
    if std::env::var("SBX_DEBUG").ok().as_deref() == Some("1") {
        append_line(&format!("DEBUG: {msg}"), base_dir);
        eprintln!("{}", redact_credentials(msg));
    }
}

// Unconditional note logging to the daily sandbox log.
pub fn log_note(msg: &str, base_dir: Option<&Path>) {
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    append_line(&format!("[{ts} {}] {}", exe_label(), msg), base_dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_does_not_panic_on_utf8_boundary() {
        // Place a 4-byte emoji such that naive (byte-based) truncation would split it.
        let prefix = "x".repeat(LOG_COMMAND_PREVIEW_LIMIT - 1);
        let command = vec![format!("{prefix}😀")];
        let result = std::panic::catch_unwind(|| preview(&command));
        assert!(result.is_ok());
        let previewed = result.unwrap();
        assert!(previewed.len() <= LOG_COMMAND_PREVIEW_LIMIT);
    }

    #[test]
    fn log_note_writes_to_daily_rolling_log() {
        let tempdir = tempfile::tempdir().expect("tempdir");

        log_note("hello daily log", Some(tempdir.path()));

        let entries = std::fs::read_dir(tempdir.path())
            .expect("read log dir")
            .collect::<Result<Vec<_>, _>>()
            .expect("read entries");
        assert_eq!(entries.len(), 1);

        let log_path = entries[0].path();
        let filename = log_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("utf-8 filename");
        assert!(filename.starts_with("sandbox."));
        assert!(filename.ends_with(".log"));

        let log = std::fs::read_to_string(log_path).expect("read log");
        assert!(log.contains("hello daily log"));
    }

    fn read_only_log(dir: &Path) -> String {
        let entries = std::fs::read_dir(dir)
            .expect("read log dir")
            .collect::<Result<Vec<_>, _>>()
            .expect("read entries");
        assert_eq!(entries.len(), 1);
        std::fs::read_to_string(entries[0].path()).expect("read log")
    }

    /// #398: credentials typed into a sandboxed command never reach the
    /// sandbox log, including one the 200-byte preview would cut in half.
    #[test]
    fn command_log_lines_redact_credentials() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let secrets = [
            "fake-398-bearer-0001",
            "fake-398-password-0002",
            "fake-398-xapikey-0003",
            "fake-398-query-0004",
            // An OpenAI-format key is only recognised whole: truncating
            // before redacting would leave its first 16 bytes.
            "sk-proj-FAKE398STRADDLE0005AAAAAAAAAA",
        ];
        let command =
            |args: &[&str]| -> Vec<String> { args.iter().map(ToString::to_string).collect() };
        let curl = command(&[
            "curl.exe",
            "-H",
            &format!("Authorization: Bearer {}", secrets[0]),
            "-H",
            &format!("X-Api-Key: {}", secrets[2]),
            &format!("https://api.example/v1?api_key={}", secrets[3]),
        ]);
        let mysql = command(&["mysql", &format!("--password={}", secrets[1])]);
        let padding = "x".repeat(LOG_COMMAND_PREVIEW_LIMIT - " tool ".len() - 16);
        let straddle = command(&[&padding, "tool", secrets[4]]);
        log_start(&curl, Some(tempdir.path()));
        log_success(&mysql, Some(tempdir.path()));
        log_failure(&straddle, "exit code 1", Some(tempdir.path()));
        log_note(
            &format!("runner failed: {}", curl.join(" ")),
            Some(tempdir.path()),
        );

        let log = read_only_log(tempdir.path());
        assert!(
            log.contains("START: curl.exe -H Authorization: REDACTED"),
            "{log}"
        );
        assert!(log.contains("SUCCESS: mysql --password=REDACTED"), "{log}");
        assert!(log.contains("FAILURE: "), "{log}");
        assert!(log.contains("runner failed: curl.exe"), "{log}");
        for secret in secrets {
            // No secret, nor the start of one a truncation could leave.
            assert!(!log.contains(&secret[..12]), "{secret} leaked: {log}");
        }
    }

    /// Writers built by `log_writer` (the setup helper's) redact whole
    /// lines, however `writeln!` splits them.
    #[test]
    fn log_writer_redacts_lines_split_across_writes() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        {
            let mut writer = log_writer(tempdir.path()).expect("log writer");
            writer.write_all(b"setup: tool --password ").unwrap();
            writer.write_all(b"fake-398-split-0006\nnext ").unwrap();
            writer
                .write_all(b"line OPENAI_API_KEY=fake-398-partial-0007")
                .unwrap();
        }
        let log = read_only_log(tempdir.path());
        assert_eq!(
            log,
            "setup: tool --password REDACTED\nnext line OPENAI_API_KEY=REDACTED"
        );
    }

    #[test]
    fn log_file_path_for_utc_date_matches_rolling_appender_name() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 5, 21).expect("valid date");

        assert_eq!(
            log_file_path_for_utc_date(Path::new("logs"), date),
            PathBuf::from("logs").join("sandbox.2026-05-21.log")
        );
    }

    #[test]
    fn current_log_file_path_for_codex_home_uses_sandbox_dir() {
        let codex_home = Path::new("codex-home");

        assert_eq!(
            current_log_file_path_for_codex_home(codex_home),
            current_log_file_path(&codex_home.join(".sandbox"))
        );
    }
}
