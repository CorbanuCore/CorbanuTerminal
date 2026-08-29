use crate::BrowserError;
use crate::ContainerEngine;
use crate::EngineKind;
use crate::engine::strings;
use serde_json::Value;
use std::time::Duration;
use tokio::process::Child;

const OWNER_LABEL: &str = "org.corbanu.browser.owner";
const MEMORY: u64 = 1_073_741_824;
const PIDS: u64 = 256;

/// One acquisition, one disposable namespace/profile. Names and ownership tokens
/// are host-generated; neither a URL nor worker output can select a container.
pub(crate) struct OwnedContainer {
    engine: ContainerEngine,
    name: String,
    owner: String,
    image: String,
    armed: bool,
}

impl OwnedContainer {
    pub fn new(engine: ContainerEngine, image: String) -> Self {
        Self {
            engine,
            name: format!("corbanu-browser-{}", uuid::Uuid::new_v4().simple()),
            owner: uuid::Uuid::new_v4().to_string(),
            image,
            armed: false,
        }
    }

    pub async fn create(&mut self) -> Result<(), BrowserError> {
        // Arm before create: cancellation may lose the CLI result after the
        // daemon creates the container. The owner stays outside the cancelled
        // future, so normal cancellation can await cleanup before returning.
        self.armed = true;
        self.engine
            .command
            .run(&self.create_args(), Duration::from_secs(20))
            .await?;
        self.verify().await?;
        Ok(())
    }

    fn create_args(&self) -> Vec<String> {
        let mut args = strings(&[
            "create",
            "--name",
            &self.name,
            "--label",
            &format!("{OWNER_LABEL}={}", self.owner),
            "--network=none",
            "--read-only",
            "--user=65532:65532",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--ipc=private",
            "--cgroupns=private",
            "--memory=1073741824",
            "--memory-swap=1073741824",
            "--pids-limit=256",
            "--cpu-period=100000",
            "--cpu-quota=100000",
            "--tmpfs=/tmp:rw,nosuid,nodev,noexec,size=268435456,mode=1777",
            "--log-driver=none",
            "--init",
        ]);
        match self.engine.kind() {
            // Docker's private PID/UTS defaults are selected by omission; it
            // rejects the literal "private" accepted by Podman. Override only
            // this container's seccomp default, never the shared daemon.
            EngineKind::Docker => args.push("--security-opt=seccomp=builtin".into()),
            EngineKind::Podman => args.extend(strings(&[
                "--pid=private",
                "--uts=private",
                "--seccomp-policy=default",
            ])),
        }
        args.extend([self.image.clone(), "idle".into()]);
        args
    }

    async fn inspect(&self) -> Result<Value, BrowserError> {
        let values = self
            .engine
            .json(&strings(&["container", "inspect", &self.name]))
            .await?;
        let value = values.get(0).ok_or(BrowserError::ContainerMismatch)?;
        verify_owner(value, &self.owner, &self.image)?;
        Ok(value.clone())
    }

    async fn verify(&self) -> Result<Value, BrowserError> {
        let value = self.inspect().await?;
        verify_confinement(self.engine.kind(), &value)?;
        Ok(value)
    }

    /// Start a stopped owned service, or recover a stalled one exactly once.
    /// Never stop/restart/remove a container which fails the ownership check.
    pub async fn ensure_ready(&self) -> Result<(), BrowserError> {
        let state = self.verify().await?;
        if state.pointer("/State/Running").and_then(Value::as_bool) != Some(true) {
            self.engine
                .command
                .run(&strings(&["start", &self.name]), Duration::from_secs(15))
                .await?;
        }
        if self.probe().await.is_ok() {
            return Ok(());
        }
        self.verify().await?;
        self.engine
            .command
            .run(
                &strings(&["restart", "--time=2", &self.name]),
                Duration::from_secs(15),
            )
            .await?;
        self.probe().await
    }

    async fn probe(&self) -> Result<(), BrowserError> {
        let value = self.verify().await?;
        if value.pointer("/State/Running").and_then(Value::as_bool) != Some(true) {
            return Err(BrowserError::HealthCheckFailed);
        }
        let output = self
            .engine
            .command
            .run(&self.exec_args("probe"), Duration::from_secs(25))
            .await
            .map_err(|_| BrowserError::HealthCheckFailed)?;
        if serde_json::from_slice::<Value>(&output).ok()
            != Some(serde_json::json!({"type":"healthy","version":1}))
        {
            return Err(BrowserError::HealthCheckFailed);
        }
        Ok(())
    }

    fn exec_args(&self, mode: &str) -> Vec<String> {
        strings(&[
            "exec",
            "-i",
            "--user=65532:65532",
            &self.name,
            "/app/.venv/bin/python",
            "-I",
            "-u",
            "/opt/corbanu/worker.py",
            mode,
        ])
    }

    pub async fn acquire_worker(&self) -> Result<Child, BrowserError> {
        self.verify().await?;
        self.engine.command.spawn(&self.exec_args("acquire"))
    }

    pub async fn close(mut self) -> Result<(), BrowserError> {
        if !self.armed {
            return Ok(());
        }
        let result = remove_owned(&self.engine, &self.name, &self.owner, &self.image).await;
        if result.is_ok() {
            self.armed = false;
        }
        result
    }
}

impl Drop for OwnedContainer {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let engine = self.engine.clone();
            let name = self.name.clone();
            let owner = self.owner.clone();
            let image = self.image.clone();
            runtime.spawn(async move {
                let _ = remove_owned(&engine, &name, &owner, &image).await;
            });
        }
    }
}

async fn remove_owned(
    engine: &ContainerEngine,
    name: &str,
    owner: &str,
    image: &str,
) -> Result<(), BrowserError> {
    let value = engine
        .json(&strings(&["container", "inspect", name]))
        .await?;
    verify_owner(
        value.get(0).ok_or(BrowserError::ContainerMismatch)?,
        owner,
        image,
    )?;
    engine
        .command
        .run(&strings(&["rm", "--force", name]), Duration::from_secs(15))
        .await?;
    Ok(())
}

fn verify_owner(value: &Value, owner: &str, image: &str) -> Result<(), BrowserError> {
    if value
        .pointer("/Config/Labels/org.corbanu.browser.owner")
        .and_then(Value::as_str)
        != Some(owner)
        || value["Image"]
            .as_str()
            .map(|id| id.trim_start_matches("sha256:"))
            != Some(image.trim_start_matches("sha256:"))
    {
        return Err(BrowserError::ContainerMismatch);
    }
    Ok(())
}

fn empty(value: &Value) -> bool {
    value.is_null() || value.as_array().is_some_and(Vec::is_empty)
}

fn bounded_tmpfs(value: &Value) -> bool {
    let Some(tmpfs) = value.as_object() else {
        return false;
    };
    if tmpfs.len() != 1 {
        return false;
    }
    let Some(options) = tmpfs.get("/tmp").and_then(Value::as_str) else {
        return false;
    };
    let parts = options.split(',').collect::<Vec<_>>();
    ["rw", "nosuid", "nodev", "noexec"]
        .iter()
        .all(|required| parts.contains(required))
        && parts
            .iter()
            .filter(|part| part.starts_with("size="))
            .copied()
            .collect::<Vec<_>>()
            == ["size=268435456"]
        && !parts
            .iter()
            .any(|part| matches!(*part, "exec" | "suid" | "dev"))
}

fn dropped_capabilities(kind: EngineKind, value: &Value) -> bool {
    match kind {
        EngineKind::Docker => value["HostConfig"]["CapDrop"]
            .as_array()
            .is_some_and(|caps| caps.iter().any(|cap| cap == "ALL" || cap == "CAP_ALL")),
        // Podman expands CapDrop to names; its computed capability sets are
        // authoritative. Null denotes an empty set, but missing keys are not
        // evidence. The fixed worker also verifies every kernel capability set
        // before idle/probe/acquire, so inspect alone cannot report readiness.
        EngineKind::Podman => ["EffectiveCaps", "BoundingCaps"]
            .iter()
            .all(|key| value.get(key).is_some_and(empty)),
    }
}

fn verify_confinement(kind: EngineKind, value: &Value) -> Result<(), BrowserError> {
    let host = &value["HostConfig"];
    let security = host["SecurityOpt"]
        .as_array()
        .ok_or(BrowserError::ContainerMismatch)?;
    if value.pointer("/Config/User").and_then(Value::as_str) != Some("65532:65532")
        || host["NetworkMode"] != "none"
        || host["ReadonlyRootfs"] != true
        || host["Privileged"] != false
        || !empty(&host["Binds"])
        || !empty(&host["Devices"])
        || !empty(&host["CapAdd"])
        || !empty(&host["VolumesFrom"])
        || !matches!(host["PidMode"].as_str(), Some("private" | ""))
        || !matches!(host["UTSMode"].as_str(), Some("private" | ""))
        || !matches!(host["IpcMode"].as_str(), Some("private"))
        || match kind {
            EngineKind::Docker => host["CgroupnsMode"] != "private",
            EngineKind::Podman => host["CgroupMode"] != "private",
        }
        || host["Memory"].as_u64() != Some(MEMORY)
        || host["MemorySwap"].as_u64() != Some(MEMORY)
        || host["PidsLimit"].as_u64() != Some(PIDS)
        || host["CpuPeriod"].as_u64() != Some(100_000)
        || host["CpuQuota"].as_u64() != Some(100_000)
        || !dropped_capabilities(kind, value)
        || (kind == EngineKind::Docker && !security.iter().any(|item| item == "seccomp=builtin"))
        || !security.iter().any(|item| {
            matches!(
                item.as_str(),
                Some("no-new-privileges" | "no-new-privileges=true")
            )
        })
        || security
            .iter()
            .any(|item| item.as_str().is_some_and(|s| s.contains("unconfined")))
        || !value["Mounts"].as_array().is_some_and(|mounts| {
            mounts
                .iter()
                .all(|mount| mount["Type"] == "tmpfs" && mount["Destination"] == "/tmp")
        })
        || !bounded_tmpfs(&host["Tmpfs"])
    {
        return Err(BrowserError::ContainerMismatch);
    }
    Ok(())
}

#[cfg(test)]
#[path = "container_tests.rs"]
mod tests;
