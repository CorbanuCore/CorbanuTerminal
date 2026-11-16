use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use anyhow::Result;
use codex_uds::UnixListener;
use futures::SinkExt;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use tokio_tungstenite::tungstenite::Message;

use super::BootstrapOptions;
use super::Daemon;
use super::LifecycleCommand;
use super::LifecycleStatus;
use super::RemoteControlMode;
use super::backend;
use super::settings::DaemonSettings;

#[tokio::test]
async fn lifecycle_stops_old_updater_even_when_managed_binary_is_missing() -> Result<()> {
    #[derive(Debug)]
    enum Operation {
        Lifecycle(LifecycleCommand),
        RemoteControl(RemoteControlMode),
        Bootstrap,
    }

    let mut observed = Vec::new();
    for operation in [
        Operation::Lifecycle(LifecycleCommand::Start),
        Operation::Lifecycle(LifecycleCommand::Restart),
        Operation::Lifecycle(LifecycleCommand::Stop),
        Operation::RemoteControl(RemoteControlMode::Enabled),
        Operation::RemoteControl(RemoteControlMode::Disabled),
        Operation::Bootstrap,
    ] {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let daemon = Daemon {
            socket_path: root.join("control.sock"),
            pid_file: root.join("server.pid"),
            update_pid_file: root.join("updater.pid"),
            operation_lock_file: root.join("daemon.lock"),
            settings_file: root.join("settings.json"),
            managed_codex_bin: root.join("corbanu"),
        };
        let quoted_root = root.display().to_string().replace('\'', "'\\''");
        tokio::fs::write(
            &daemon.managed_codex_bin,
            format!("#!/bin/sh\ntouch '{quoted_root}/ready'\nexec /bin/sleep 30\n"),
        )
        .await?;
        tokio::fs::set_permissions(
            &daemon.managed_codex_bin,
            std::fs::Permissions::from_mode(0o700),
        )
        .await?;
        let settings = DaemonSettings::default();
        let updater = backend::pid_update_loop_backend(daemon.backend_paths(&settings));
        updater.start().await?;
        let ready = tokio::time::timeout(Duration::from_secs(2), async {
            while !root.join("ready").exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;
        if ready.is_err() {
            updater.stop().await?;
        }
        ready?;
        // The script is now open. Move its directory entry to simulate a missing
        // managed install while the updater's old executable image keeps running.
        let updater_bin = root.join("updater-fixture");
        tokio::fs::rename(&daemon.managed_codex_bin, &updater_bin).await?;
        let result = match operation {
            Operation::Lifecycle(command) => daemon.run(command).await.map(|_| ()),
            Operation::RemoteControl(mode) => daemon.set_remote_control(mode).await.map(|_| ()),
            Operation::Bootstrap => daemon
                .bootstrap(BootstrapOptions {
                    remote_control_enabled: true,
                })
                .await
                .map(|_| ()),
        };
        let still_running = updater.is_starting_or_running().await?;
        updater.stop().await?;
        observed.push((format!("{operation:?}"), still_running, result.is_ok()));
    }

    assert_eq!(
        observed,
        vec![
            ("Lifecycle(Start)".to_string(), false, false),
            ("Lifecycle(Restart)".to_string(), false, false),
            ("Lifecycle(Stop)".to_string(), false, true),
            ("RemoteControl(Enabled)".to_string(), false, true),
            ("RemoteControl(Disabled)".to_string(), false, true),
            ("Bootstrap".to_string(), false, false),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn bootstrap_stops_old_updater_without_relaunching_and_keeps_daemon_working() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path();
    let daemon = Daemon {
        socket_path: root.join("control.sock"),
        pid_file: root.join("server.pid"),
        update_pid_file: root.join("updater.pid"),
        operation_lock_file: root.join("daemon.lock"),
        settings_file: root.join("settings.json"),
        managed_codex_bin: root.join("corbanu"),
    };
    let quoted_root = root.display().to_string().replace('\'', "'\\''");
    tokio::fs::write(
        &daemon.managed_codex_bin,
        format!(
            "#!/bin/sh\ncd '{quoted_root}' || exit 1\n\
             case \"$*\" in\n\
             --version) echo 'corbanu 1.2.3'; exit 0 ;;\n\
             *pid-update-loop*) echo updater >> updater-runs ;;\n\
             *) touch server-started ;;\n\
             esac\nexec /bin/sleep 30\n"
        ),
    )
    .await?;
    tokio::fs::set_permissions(
        &daemon.managed_codex_bin,
        std::fs::Permissions::from_mode(0o700),
    )
    .await?;

    // Simulate an updater left by an earlier build, using only a local fixture.
    let settings = DaemonSettings::default();
    let updater = backend::pid_update_loop_backend(daemon.backend_paths(&settings));
    updater.start().await?;
    let server_started = root.join("server-started");
    let socket = daemon.socket_path.clone();
    let fake_server = tokio::spawn(async move {
        while !server_started.exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let mut listener = UnixListener::bind(&socket).await?;
        for _ in 0..2 {
            let stream = listener.accept().await?;
            let mut ws = tokio_tungstenite::accept_async(stream).await?;
            let _ = ws.next().await;
            ws.send(Message::Text(
                serde_json::json!({
                    "id": 1,
                    "result": {
                        "userAgent": "corbanu/1.2.3",
                        "codexHome": "/",
                        "platformFamily": "unix",
                        "platformOs": "test"
                    }
                })
                .to_string()
                .into(),
            ))
            .await?;
            let _ = ws.next().await;
        }
        Ok::<(), anyhow::Error>(())
    });

    let outcome = tokio::time::timeout(Duration::from_secs(15), async {
        // Wait for the fixture to have really started before bootstrap stops it.
        while !root.join("updater-runs").exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let output = daemon
            .bootstrap(BootstrapOptions {
                remote_control_enabled: true,
            })
            .await?;
        let version = daemon.run(LifecycleCommand::Version).await?;
        let updater_running = updater.is_starting_or_running().await?;
        Ok::<_, anyhow::Error>((output, version, updater_running))
    })
    .await;

    // Clean up both fixture process groups even when the regression fails.
    let stop = daemon.run(LifecycleCommand::Stop).await;
    let updater_stop = updater.stop().await;
    fake_server.abort();
    let _ = fake_server.await;
    stop?;
    updater_stop?;

    let (output, version, updater_running) = outcome??;
    assert_eq!(
        (
            output.auto_update_enabled,
            output.remote_control_enabled,
            output.app_server_version.as_str(),
            version.status,
            updater_running,
            tokio::fs::read_to_string(root.join("updater-runs")).await?,
        ),
        (
            false,
            true,
            "1.2.3",
            LifecycleStatus::Running,
            false,
            "updater\n".to_string(),
        )
    );
    Ok(())
}
