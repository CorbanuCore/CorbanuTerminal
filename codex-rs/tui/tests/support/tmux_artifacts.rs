use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use serde_json::json;

const MAX_ATTACHMENT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct FailureCapture {
    pub(super) reason: String,
    pub(super) viewport: String,
    pub(super) scrollback: String,
    pub(super) pane_metadata: String,
}

#[derive(Debug)]
pub(super) struct ArtifactRecorder {
    scenario: String,
    directory: PathBuf,
    reproduction: String,
    state: Mutex<ArtifactState>,
}

#[derive(Debug, Default)]
struct ArtifactState {
    commands: Vec<String>,
    inputs: Vec<String>,
    dimensions: Vec<String>,
    attachments: Vec<(String, PathBuf)>,
    emitted: bool,
}

impl ArtifactRecorder {
    pub(super) fn new(root: PathBuf, scenario: &str, id: u64) -> Self {
        let scenario = safe_name(scenario);
        let directory = root.join(format!("{scenario}-{}-{id}", std::process::id()));
        Self {
            reproduction: format!(
                "CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all {scenario}"
            ),
            scenario,
            directory,
            state: Mutex::new(ArtifactState::default()),
        }
    }

    pub(super) fn default_root() -> PathBuf {
        std::env::var_os("CORBANU_TMUX_ARTIFACT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/tmux-artifacts"))
    }

    pub(super) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(super) fn record_command(&self, command: String) {
        self.state
            .lock()
            .expect("artifact state poisoned")
            .commands
            .push(command);
    }

    pub(super) fn record_input(&self, input: String) {
        self.state
            .lock()
            .expect("artifact state poisoned")
            .inputs
            .push(input);
    }

    pub(super) fn record_dimensions(&self, dimensions: String) {
        self.state
            .lock()
            .expect("artifact state poisoned")
            .dimensions
            .push(dimensions);
    }

    pub(super) fn register_attachment(&self, label: &str, path: PathBuf) {
        self.state
            .lock()
            .expect("artifact state poisoned")
            .attachments
            .push((safe_name(label), path));
    }

    pub(super) fn emit(&self, capture: FailureCapture) -> Result<PathBuf> {
        let (commands, inputs, dimensions, attachments) = {
            let mut state = self.state.lock().expect("artifact state poisoned");
            if state.emitted {
                return Ok(self.directory.clone());
            }
            state.emitted = true;
            (
                state.commands.clone(),
                state.inputs.clone(),
                state.dimensions.clone(),
                state.attachments.clone(),
            )
        };

        fs::create_dir_all(&self.directory)
            .with_context(|| format!("create artifact directory {}", self.directory.display()))?;
        write(&self.directory, "reason.txt", &capture.reason)?;
        write(&self.directory, "viewport.txt", &capture.viewport)?;
        write(&self.directory, "scrollback.txt", &capture.scrollback)?;
        write(&self.directory, "pane-metadata.txt", &capture.pane_metadata)?;
        write(&self.directory, "command-log.txt", &commands.join("\n"))?;
        write(&self.directory, "input-events.txt", &inputs.join("\n"))?;
        write(&self.directory, "dimensions.txt", &dimensions.join("\n"))?;
        write(&self.directory, "reproduce.sh", &self.reproduction)?;

        let attachment_labels = attachments
            .iter()
            .map(|(label, _)| label.as_str())
            .collect::<Vec<_>>();
        let manifest = serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "scenario": self.scenario,
            "files": [
                "reason.txt",
                "viewport.txt",
                "scrollback.txt",
                "pane-metadata.txt",
                "command-log.txt",
                "input-events.txt",
                "dimensions.txt",
                "reproduce.sh"
            ],
            "attachments": attachment_labels,
        }))?;
        write(&self.directory, "manifest.json", &manifest)?;

        for (label, path) in attachments {
            copy_attachment(&self.directory, &label, &path)?;
        }
        Ok(self.directory.clone())
    }
}

fn write(directory: &Path, name: &str, contents: &str) -> Result<()> {
    fs::write(directory.join(name), contents).with_context(|| format!("write tmux artifact {name}"))
}

fn copy_attachment(directory: &Path, label: &str, source: &Path) -> Result<()> {
    if !source.is_file() {
        return write(
            directory,
            &format!("{label}.missing.txt"),
            &format!("registered artifact was unavailable: {}", source.display()),
        );
    }
    let mut contents = fs::read(source)
        .with_context(|| format!("read registered artifact {}", source.display()))?;
    if contents.len() > MAX_ATTACHMENT_BYTES {
        contents.truncate(MAX_ATTACHMENT_BYTES);
    }
    fs::write(directory.join(label), contents)
        .with_context(|| format!("copy registered artifact {label}"))
}

fn safe_name(value: &str) -> String {
    let name = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    name.trim_matches('-').to_string()
}
