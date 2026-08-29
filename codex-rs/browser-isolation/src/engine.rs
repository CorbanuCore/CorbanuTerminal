use crate::BrowserError;
use crate::command::EngineCommand;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde_json::Value;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::time::Duration;
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineKind {
    Podman,
    Docker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnginePreference {
    Discover,
    Preserve(EngineKind),
}

/// Host-selected engine. No model-supplied executable, flags, or shell commands.
#[derive(Clone)]
pub struct ContainerEngine {
    pub(crate) kind: EngineKind,
    pub(crate) command: EngineCommand,
}

impl ContainerEngine {
    pub fn kind(&self) -> EngineKind {
        self.kind
    }

    pub async fn discover(preference: EnginePreference) -> Result<Self, BrowserError> {
        let mut environment = BTreeMap::<OsString, OsString>::new();
        for key in [
            "PATH",
            "HOME",
            "USERPROFILE",
            "LOCALAPPDATA",
            "SystemRoot",
            "XDG_RUNTIME_DIR",
            "DOCKER_CONFIG",
            "DOCKER_CONTEXT",
            "DOCKER_HOST",
            "CONTAINER_HOST",
            "CONTAINER_CONNECTION",
            "CONTAINER_SSHKEY",
        ] {
            if let Some(value) = std::env::var_os(key) {
                environment.insert(key.into(), value);
            }
        }
        let path = environment
            .get(&OsString::from("PATH"))
            .cloned()
            .unwrap_or_default();
        let find = |kind: EngineKind| {
            let binary = match kind {
                EngineKind::Podman => "podman",
                EngineKind::Docker => "docker",
            };
            let binary = if cfg!(windows) {
                format!("{binary}.exe")
            } else {
                binary.to_owned()
            };
            std::env::split_paths(&path)
                .filter(|dir| dir.is_absolute())
                .map(|dir| dir.join(&binary))
                .find(|path| path.is_file())
                .and_then(|path| AbsolutePathBuf::from_absolute_path_checked(path).ok())
                .map(|executable| Self {
                    kind,
                    command: EngineCommand {
                        executable,
                        environment: environment.clone(),
                    },
                })
        };
        if let EnginePreference::Preserve(kind) = preference {
            let mut engine = find(kind).ok_or(BrowserError::RuntimeMissing)?;
            engine.pin_local_endpoint().await?;
            engine.verify_local_linux().await?;
            return Ok(engine);
        }
        let podman = find(EngineKind::Podman);
        let docker = find(EngineKind::Docker);
        match (podman, docker) {
            (None, None) => Err(BrowserError::RuntimeMissing),
            (Some(mut engine), None) | (None, Some(mut engine)) => {
                engine.pin_local_endpoint().await?;
                engine.verify_local_linux().await?;
                Ok(engine)
            }
            (Some(mut podman), Some(mut docker)) => {
                if podman.pin_local_endpoint().await.is_ok()
                    && podman.verify_local_linux().await.is_ok()
                {
                    Ok(podman)
                } else {
                    docker.pin_local_endpoint().await?;
                    docker.verify_local_linux().await?;
                    Ok(docker)
                }
            }
        }
    }

    pub(crate) async fn json(&self, args: &[String]) -> Result<Value, BrowserError> {
        serde_json::from_slice(&self.command.run(args, Duration::from_secs(15)).await?)
            .map_err(|_| BrowserError::RuntimeUnavailable)
    }

    // Freeze the selected endpoint for this handle without changing global
    // contexts. Docker's DOCKER_CONTEXT takes precedence over DOCKER_HOST.
    async fn pin_local_endpoint(&mut self) -> Result<(), BrowserError> {
        let environment = self.command.environment.clone();
        match self.kind {
            EngineKind::Docker => {
                let endpoint = if !environment.contains_key(&OsString::from("DOCKER_CONTEXT"))
                    && let Some(host) = environment.get(&OsString::from("DOCKER_HOST"))
                {
                    host.to_string_lossy().into_owned()
                } else {
                    self.json(&strings(&["context", "inspect"]))
                        .await?
                        .pointer("/0/Endpoints/docker/Host")
                        .and_then(Value::as_str)
                        .ok_or(BrowserError::UnsupportedRuntime)?
                        .to_owned()
                };
                if !(endpoint.starts_with("unix:///")
                    || (cfg!(windows) && endpoint.starts_with("npipe:////./pipe/")))
                {
                    return Err(BrowserError::UnsupportedRuntime);
                }
                self.command
                    .environment
                    .remove(&OsString::from("DOCKER_CONTEXT"));
                self.command
                    .environment
                    .insert("DOCKER_HOST".into(), endpoint.into());
            }
            EngineKind::Podman => {
                if let Some(host) = environment.get(&OsString::from("CONTAINER_HOST")) {
                    verify_podman_endpoint(&host.to_string_lossy())?;
                } else if cfg!(any(target_os = "macos", windows))
                    || environment.contains_key(&OsString::from("CONTAINER_CONNECTION"))
                {
                    let connections = self
                        .json(&strings(&[
                            "system",
                            "connection",
                            "list",
                            "--format",
                            "json",
                        ]))
                        .await?;
                    let connection = connections
                        .as_array()
                        .and_then(|items| {
                            items.iter().find(|item| {
                                environment
                                    .get(&OsString::from("CONTAINER_CONNECTION"))
                                    .map(|name| item["Name"].as_str() == name.to_str())
                                    .unwrap_or(item["Default"] == true)
                            })
                        })
                        .ok_or(BrowserError::UnsupportedRuntime)?;
                    let endpoint = connection["URI"]
                        .as_str()
                        .ok_or(BrowserError::UnsupportedRuntime)?;
                    verify_podman_endpoint(endpoint)?;
                    self.command
                        .environment
                        .insert("CONTAINER_HOST".into(), endpoint.into());
                    if let Some(identity) =
                        connection["Identity"].as_str().filter(|s| !s.is_empty())
                    {
                        AbsolutePathBuf::from_absolute_path_checked(identity)
                            .map_err(|_| BrowserError::UnsupportedRuntime)?;
                        self.command
                            .environment
                            .insert("CONTAINER_SSHKEY".into(), identity.into());
                    }
                }
                self.command
                    .environment
                    .remove(&OsString::from("CONTAINER_CONNECTION"));
            }
        }
        Ok(())
    }

    pub(crate) async fn verify_local_linux(&self) -> Result<(), BrowserError> {
        match self.kind {
            EngineKind::Docker => {
                let info = self
                    .json(&strings(&["info", "--format", "{{json .}}"]))
                    .await?;
                verify_docker_info(&info)?;
            }
            EngineKind::Podman => {
                let info = self.json(&strings(&["info", "--format", "json"])).await?;
                if info.pointer("/host/os").and_then(Value::as_str) != Some("linux")
                    || info
                        .pointer("/host/security/rootless")
                        .and_then(Value::as_bool)
                        != Some(true)
                {
                    return Err(BrowserError::UnsupportedRuntime);
                }
            }
        }
        Ok(())
    }
}

fn verify_docker_info(info: &Value) -> Result<(), BrowserError> {
    // Docker can return client-only JSON with exit 0 when the daemon is down.
    // That is unavailable infrastructure, not an unsupported operating system.
    if info["ServerErrors"]
        .as_array()
        .is_some_and(|errors| !errors.is_empty())
    {
        return Err(BrowserError::RuntimeUnavailable);
    }
    match info["OSType"].as_str() {
        Some("linux") => Ok(()),
        Some("") | None => Err(BrowserError::RuntimeUnavailable),
        Some(_) => Err(BrowserError::UnsupportedRuntime),
    }
}

fn verify_podman_endpoint(endpoint: &str) -> Result<(), BrowserError> {
    if endpoint.starts_with("unix:///") {
        return Ok(());
    }
    let url = Url::parse(endpoint).map_err(|_| BrowserError::UnsupportedRuntime)?;
    if url.scheme() == "ssh"
        && url.password().is_none()
        && matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "localhost"))
    {
        Ok(())
    } else {
        Err(BrowserError::UnsupportedRuntime)
    }
}

pub(crate) fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_owned()).collect()
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
