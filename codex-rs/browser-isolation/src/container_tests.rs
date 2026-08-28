use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

fn confined() -> Value {
    json!({
        "Image":"sha256:abc", "State":{"Running":true},
        "Config":{"User":"65532:65532","Labels":{"org.corbanu.browser.owner":"host-token"}},
        "Mounts":[],
        "HostConfig":{
            "NetworkMode":"none", "ReadonlyRootfs":true, "Privileged":false,
            "Binds":null, "Devices":[], "CapAdd":[], "CapDrop":["ALL"],
            "VolumesFrom":null, "PidMode":"private", "IpcMode":"private",
            "Memory":1073741824, "MemorySwap":1073741824, "PidsLimit":256,
            "CpuPeriod":100000, "CpuQuota":100000,
            "SecurityOpt":["no-new-privileges"],
            "Tmpfs":{"/tmp":"rw,nosuid,nodev,noexec,size=268435456,mode=1777"}
        }
    })
}

#[test]
fn inspect_requires_ownership_and_expected_image_before_mutation() {
    let state = confined();
    assert_eq!(verify_owner(&state, "host-token", "sha256:abc"), Ok(()));
    assert_eq!(
        verify_owner(&state, "other-owner", "sha256:abc"),
        Err(BrowserError::ContainerMismatch)
    );
    assert_eq!(
        verify_owner(&state, "host-token", "sha256:other"),
        Err(BrowserError::ContainerMismatch)
    );
}

#[test]
fn inspect_rejects_escape_hatches_and_missing_resource_limits() {
    assert_eq!(verify_confinement(&confined()), Ok(()));
    for (pointer, replacement) in [
        ("/Config/User", json!("0")),
        ("/HostConfig/NetworkMode", json!("host")),
        ("/HostConfig/ReadonlyRootfs", json!(false)),
        ("/HostConfig/Privileged", json!(true)),
        (
            "/HostConfig/Binds",
            json!(["/var/run/docker.sock:/var/run/docker.sock"]),
        ),
        ("/HostConfig/Devices", json!([{"PathOnHost":"/dev/kvm"}])),
        ("/HostConfig/CapAdd", json!(["SYS_ADMIN"])),
        ("/HostConfig/CapDrop", json!([])),
        ("/HostConfig/PidMode", json!("host")),
        ("/HostConfig/IpcMode", json!("host")),
        ("/HostConfig/Memory", json!(0)),
        ("/HostConfig/MemorySwap", json!(-1)),
        ("/HostConfig/PidsLimit", json!(0)),
        ("/HostConfig/CpuQuota", json!(-1)),
        (
            "/HostConfig/SecurityOpt",
            json!(["no-new-privileges", "seccomp=unconfined"]),
        ),
        ("/HostConfig/Tmpfs", json!({"/tmp":"rw", "/host":"rw"})),
        ("/HostConfig/Tmpfs", json!({"/tmp":"rw"})),
        ("/Mounts", json!([{"Type":"bind","Destination":"/tmp"}])),
    ] {
        let mut state = confined();
        *state.pointer_mut(pointer).unwrap() = replacement;
        assert_eq!(
            verify_confinement(&state),
            Err(BrowserError::ContainerMismatch),
            "{pointer}"
        );
    }
}

#[cfg(unix)]
fn fixture(failures: u32) -> (tempfile::TempDir, OwnedContainer) {
    use codex_utils_absolute_path::AbsolutePathBuf;
    use std::collections::BTreeMap;
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let executable = directory.path().join("engine");
    std::fs::write(&executable, include_str!("../worker/fixture_engine.py")).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let state_path = directory.path().join("state.json");
    let mut state = confined();
    state["State"]["Running"] = json!(false);
    std::fs::write(
        &state_path,
        serde_json::to_vec(
            &json!({"container":state,"created":false,"failures":failures,"calls":[]}),
        )
        .unwrap(),
    )
    .unwrap();
    let engine = ContainerEngine {
        kind: crate::EngineKind::Docker,
        command: crate::command::EngineCommand {
            executable: AbsolutePathBuf::from_absolute_path_checked(executable).unwrap(),
            environment: BTreeMap::from([(
                "PF30_FIXTURE_STATE".into(),
                state_path.into_os_string(),
            )]),
        },
    };
    (
        directory,
        OwnedContainer::new(engine, "sha256:abc".to_owned()),
    )
}

#[cfg(unix)]
fn read_fixture(directory: &tempfile::TempDir) -> Value {
    serde_json::from_slice(&std::fs::read(directory.path().join("state.json")).unwrap()).unwrap()
}

#[cfg(unix)]
#[tokio::test]
async fn stopped_owned_service_starts_and_stalled_service_restarts_once() {
    for failures in [0, 1, 2] {
        let (directory, mut container) = fixture(failures);
        container.create().await.unwrap();
        let result = container.ensure_ready().await;
        assert_eq!(
            result,
            if failures < 2 {
                Ok(())
            } else {
                Err(BrowserError::HealthCheckFailed)
            }
        );
        container.close().await.unwrap();
        let state = read_fixture(&directory);
        let calls = state["calls"].as_array().unwrap();
        assert_eq!(
            calls.iter().filter(|call| call[0] == "restart").count(),
            usize::from(failures > 0)
        );
        assert_eq!(calls.iter().filter(|call| call[0] == "start").count(), 1);
        assert_eq!(state["created"], false);
    }
}

#[cfg(unix)]
#[tokio::test]
async fn collision_cannot_restart_or_remove_another_owners_service() {
    let (directory, mut container) = fixture(1);
    container.create().await.unwrap();
    let mut state = read_fixture(&directory);
    state["container"]["Config"]["Labels"]["org.corbanu.browser.owner"] = json!("other-owner");
    std::fs::write(
        directory.path().join("state.json"),
        serde_json::to_vec(&state).unwrap(),
    )
    .unwrap();
    assert_eq!(
        container.ensure_ready().await,
        Err(BrowserError::ContainerMismatch)
    );
    assert_eq!(
        container.close().await,
        Err(BrowserError::ContainerMismatch)
    );
    let state = read_fixture(&directory);
    assert!(
        !state["calls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|call| matches!(call[0].as_str(), Some("start" | "restart" | "rm")))
    );
}

#[cfg(unix)]
#[tokio::test]
async fn cancellation_can_await_cleanup_after_dropping_the_worker_future() {
    let (directory, mut container) = fixture(0);
    container.create().await.unwrap();
    let mut state = read_fixture(&directory);
    state["stall"] = json!(true);
    std::fs::write(
        directory.path().join("state.json"),
        serde_json::to_vec(&state).unwrap(),
    )
    .unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(500), container.ensure_ready())
            .await
            .is_err()
    );
    container.close().await.unwrap();
    assert_eq!(read_fixture(&directory)["created"], false);
}
