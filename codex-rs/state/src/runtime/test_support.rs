#[cfg(test)]
use chrono::DateTime;
#[cfg(test)]
use chrono::Utc;
#[cfg(test)]
use codex_protocol::ThreadId;
#[cfg(test)]
use codex_protocol::openai_models::ReasoningEffort;
#[cfg(test)]
use codex_protocol::protocol::AskForApproval;
#[cfg(test)]
use codex_protocol::protocol::SandboxPolicy;
#[cfg(test)]
use codex_protocol::protocol::ThreadHistoryMode;
#[cfg(test)]
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use std::time::SystemTime;
#[cfg(test)]
use std::time::UNIX_EPOCH;
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
use crate::ThreadMetadata;

#[cfg(test)]
pub(crate) fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    std::env::temp_dir().join(format!(
        "codex-state-runtime-test-{nanos}-{}",
        Uuid::new_v4()
    ))
}

#[cfg(test)]
pub(super) fn test_thread_metadata(
    codex_home: &Path,
    thread_id: ThreadId,
    cwd: PathBuf,
) -> ThreadMetadata {
    let now = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("timestamp");
    ThreadMetadata {
        id: thread_id,
        rollout_path: codex_home.join(format!("rollout-{thread_id}.jsonl")),
        created_at: now,
        updated_at: now,
        recency_at: now,
        source: "cli".to_string(),
        history_mode: ThreadHistoryMode::Legacy,
        thread_source: None,
        agent_nickname: None,
        agent_role: None,
        agent_path: None,
        model_provider: "test-provider".to_string(),
        model: Some("gpt-5".to_string()),
        reasoning_effort: Some(ReasoningEffort::Medium),
        cwd,
        cli_version: "0.0.0".to_string(),
        title: String::new(),
        name: None,
        preview: Some("hello".to_string()),
        sandbox_policy: crate::extract::enum_to_string(&SandboxPolicy::new_read_only_policy()),
        approval_mode: crate::extract::enum_to_string(&AskForApproval::OnRequest),
        tokens_used: 0,
        first_user_message: Some("hello".to_string()),
        archived_at: None,
        section: None,
        section_position: None,
        section_entered_at: None,
        git_sha: None,
        git_branch: None,
        git_origin_url: None,
    }
}

/// Lets the child of `fork_child_holding_descriptors` start its program when
/// dropped, on every path, so a failing check cannot leave it waiting.
#[cfg(all(test, unix))]
pub(crate) struct ReleaseChild(std::io::PipeWriter);

#[cfg(all(test, unix))]
impl Drop for ReleaseChild {
    fn drop(&mut self) {
        use std::io::Write;
        let _ = self.0.write_all(b"r");
    }
}

/// Fork a child, from another thread as any spawn does, that keeps a copy of
/// every descriptor of this process open until the returned `ReleaseChild` is
/// dropped, and return once it has forked. #928: a lock released only by
/// closing its file stays held through that copy.
#[cfg(all(test, unix))]
pub(crate) fn fork_child_holding_descriptors() -> std::io::Result<(
    ReleaseChild,
    std::thread::JoinHandle<std::io::Result<std::process::ExitStatus>>,
)> {
    use std::io::Read;
    use std::io::Write;
    use std::os::unix::process::CommandExt;
    // The child reports the fork, then waits for the release byte, so its
    // copies stay open until the caller's checks have run.
    let (mut forked, mut forked_tx) = std::io::pipe()?;
    let (mut release_rx, release_tx) = std::io::pipe()?;
    let release = ReleaseChild(release_tx);
    let spawner = std::thread::spawn(move || {
        let mut command = std::process::Command::new("/bin/sh");
        command.args(["-c", "exit 0"]);
        // SAFETY: the hook only writes to and reads from pipes, both safe after fork.
        unsafe {
            command.pre_exec(move || {
                forked_tx.write_all(b"f")?;
                release_rx.read_exact(&mut [0; 1])?;
                Ok(())
            });
        }
        command.status()
    });
    let mut byte = [0; 1];
    forked.read_exact(&mut byte)?;
    Ok((release, spawner))
}
