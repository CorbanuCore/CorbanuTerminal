//! Optional local control inbox for supervised terminal agents. Work enters the
//! normal user-turn path only when the composer is empty and the TUI is ready.
use super::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkOrder {
    id: String,
    prompt: String,
}

pub(crate) fn start(tx: &AppEventSender) {
    let Some(directory) = std::env::var_os("CORBANU_AGENT_CONTROL_DIR") else {
        return;
    };
    let directory = PathBuf::from(directory);
    if !directory.is_absolute() {
        return;
    }
    let tx = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            tx.send(AppEvent::AgentControlTick {
                directory: directory.clone(),
            });
        }
    });
}

fn write_json(path: &Path, value: &serde_json::Value) -> std::io::Result<()> {
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, serde_json::to_vec(value)?)?;
    std::fs::rename(temporary, path)
}

impl ChatWidget {
    pub(crate) fn handle_agent_control_tick(&mut self, directory: &Path) {
        if let Err(error) = self.poll_agent_control(directory) {
            tracing::warn!(%error, "agent control inbox unavailable");
        }
    }

    fn poll_agent_control(&mut self, directory: &Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))?;
        }
        let ready = self.thread_id().is_some()
            && self.bottom_pane.is_normal_backtrack_mode()
            && self.bottom_pane.composer_is_empty();
        write_json(
            &directory.join("status.json"),
            &json!({
                "version": 1, "pid": std::process::id(), "ready": ready,
                "threadId": self.thread_id().map(|id| id.to_string()),
                "updatedAt": chrono::Utc::now().to_rfc3339(),
            }),
        )?;
        if !ready {
            return Ok(());
        }
        let inbox = directory.join("inbox.json");
        let metadata = match std::fs::metadata(&inbox) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        anyhow::ensure!(metadata.len() <= 128 * 1024, "agent work order too large");
        let work: WorkOrder = serde_json::from_slice(&std::fs::read(&inbox)?)?;
        anyhow::ensure!(
            !work.id.is_empty()
                && work.id.len() <= 180
                && work
                    .id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte)),
            "agent work order id invalid"
        );
        anyhow::ensure!(
            !work.prompt.trim().is_empty(),
            "agent work order prompt required"
        );
        let receipt = directory.join(format!("accepted-{}.json", work.id));
        if !receipt.exists() {
            self.campaign_tracker_automated_input = true;
            self.submit_user_message(work.prompt.into());
            self.campaign_tracker_automated_input = false;
            write_json(
                &receipt,
                &json!({ "id": work.id, "pid": std::process::id(), "acceptedAt": chrono::Utc::now().to_rfc3339() }),
            )?;
        }
        std::fs::remove_file(inbox)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn ready_inbox_submits_once_and_never_overwrites_user_input() {
        let (mut chat, _, _, mut operations) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        chat.thread_id = Some(codex_protocol::ThreadId::new());
        let directory = tempfile::tempdir().expect("control directory");
        let inbox = directory.path().join("inbox.json");
        let work = br#"{"id":"round_fixture","prompt":"Review the fixture work order."}"#;
        chat.bottom_pane
            .set_composer_text("Unsent user input".to_string(), vec![], vec![]);
        std::fs::write(&inbox, work).expect("work order");
        chat.poll_agent_control(directory.path())
            .expect("busy poll");
        assert!(inbox.exists());
        assert!(operations.try_recv().is_err());
        chat.bottom_pane
            .set_composer_text(String::new(), vec![], vec![]);
        chat.poll_agent_control(directory.path())
            .expect("ready poll");
        assert!(!inbox.exists());
        assert!(
            directory
                .path()
                .join("accepted-round_fixture.json")
                .exists()
        );
        assert!(operations.try_recv().is_ok());
        // A replayed delivery is already acknowledged even after the turn ends.
        chat.bottom_pane.set_task_running(false);
        std::fs::write(&inbox, work).expect("redelivery");
        chat.poll_agent_control(directory.path())
            .expect("replay poll");
        assert!(!inbox.exists());
        assert!(operations.try_recv().is_err());
    }

    #[tokio::test]
    async fn pending_work_is_preserved_until_the_terminal_has_a_ready_thread() {
        let (mut chat, _, _, _) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        let directory = tempfile::tempdir().expect("control directory");
        let inbox = directory.path().join("inbox.json");
        std::fs::write(
            &inbox,
            br#"{"id":"round_fixture","prompt":"Review the fixture work order."}"#,
        )
        .expect("work order");
        chat.poll_agent_control(directory.path())
            .expect("poll without an initialized thread");
        assert!(inbox.exists());
        let status: serde_json::Value = serde_json::from_slice(
            &std::fs::read(directory.path().join("status.json")).expect("status"),
        )
        .expect("status JSON");
        assert_eq!(status["ready"], false);
    }
}
