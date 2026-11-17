use super::common::SessionFileCandidate;
use super::common::detect_recent_sessions;
use crate::model::ExternalAgentSessionImportLimits;
use crate::sessions::ExternalAgentSessionMigration;
use crate::sessions::SessionRecordFormat;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Upper bound on filesystem probes spent recovering one Cursor project path.
///
/// Every existing directory on a candidate path costs at most
/// `1 + 3 * CUR_PROJECT_SEPARATORS.len()` = 34 probes, so this allows about 30
/// resolved directory levels (deep or hyphen-heavy paths, such as isolated test
/// homes) while keeping the worst case per project to 1024 `stat` calls.
const MAX_CUR_PROJECT_PATH_PROBES: usize = 1024;
/// Most encoded components rebuilt into a single directory name.
const MAX_CUR_PROJECT_NAME_COMPONENTS: usize = 4;
const CUR_PROJECT_SEPARATORS: [&str; 11] =
    ["-", "_", ".", " ", "--", "..", "__", "  ", "+", "@", "&"];

pub fn detect_recent_cur_sessions(
    external_agent_home: &Path,
    codex_home: &Path,
) -> io::Result<Vec<ExternalAgentSessionMigration>> {
    detect_recent_cur_sessions_with_limits(
        external_agent_home,
        codex_home,
        ExternalAgentSessionImportLimits::default(),
    )
}

pub(crate) fn detect_recent_cur_sessions_with_limits(
    external_agent_home: &Path,
    codex_home: &Path,
    limits: ExternalAgentSessionImportLimits,
) -> io::Result<Vec<ExternalAgentSessionMigration>> {
    let projects_root = external_agent_home.join("projects");
    if !projects_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for project_entry in fs::read_dir(projects_root)? {
        let Ok(project_entry) = project_entry else {
            continue;
        };
        let project_storage = project_entry.path();
        if !project_storage.is_dir() {
            continue;
        }
        let fallback_cwd = cur_project_cwd(&project_storage, external_agent_home);
        for path in cur_transcript_files(&project_storage.join("agent-transcripts")) {
            candidates.push(SessionFileCandidate {
                path,
                fallback_cwd: fallback_cwd.clone(),
                record_format: SessionRecordFormat::Cur,
            });
        }
    }
    detect_recent_sessions(
        codex_home, candidates, /*require_existing_cwd*/ false, limits,
    )
}

fn cur_transcript_files(transcripts_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![transcripts_root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if entry.file_name() != "subagents" {
                    pending.push(path);
                }
            } else if file_type.is_file()
                && path.extension().and_then(|extension| extension.to_str()) == Some("jsonl")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn cur_project_cwd(project_storage: &Path, external_agent_home: &Path) -> Option<PathBuf> {
    let encoded = project_storage.file_name()?.to_str()?;
    // Cursor stores projectless chats under this reserved project name.
    if encoded == "empty-window" {
        let external_agent_home = if external_agent_home.is_absolute() {
            external_agent_home.to_path_buf()
        } else {
            std::env::current_dir().ok()?.join(external_agent_home)
        };
        return external_agent_home.parent().map(Path::to_path_buf);
    }
    decode_cur_project_path(encoded)
}

fn decode_cur_project_path(encoded: &str) -> Option<PathBuf> {
    decode_cur_project_path_with_probe_limit(encoded, MAX_CUR_PROJECT_PATH_PROBES)
}

/// Recovers the project directory from Cursor's lossy encoding, where every path
/// separator and punctuation run became `-`.
///
/// Each directory name is rebuilt from 1 to [`MAX_CUR_PROJECT_NAME_COMPONENTS`]
/// encoded components joined by one separator, and only existing directories are
/// descended into. The search must finish within `probe_limit` filesystem probes and
/// find exactly one directory; ambiguity or an exhausted budget yields `None`, so
/// an unresolved project imports without a recovered working directory.
fn decode_cur_project_path_with_probe_limit(encoded: &str, probe_limit: usize) -> Option<PathBuf> {
    #[cfg(not(windows))]
    let root = PathBuf::from("/");

    #[cfg(windows)]
    let (encoded, root) = {
        let (drive, encoded) = decode_cur_windows_project_drive(encoded)?;
        (encoded, PathBuf::from(format!("{drive}:\\")))
    };

    let encoded = encoded.strip_prefix('-').unwrap_or(encoded);
    let components = encoded.split('-').collect::<Vec<_>>();
    if components.iter().any(|component| {
        component.is_empty()
            || matches!(*component, "." | "..")
            || component.contains(['/', '\\', ':'])
    }) {
        return None;
    }

    let mut probes = 0;
    let mut matched_path: Option<PathBuf> = None;
    let mut pending = vec![(root, 0)];
    while let Some((directory, consumed)) = pending.pop() {
        let remaining = &components[consumed..];
        if remaining.is_empty() {
            if matched_path
                .as_ref()
                .is_some_and(|matched_path| matched_path != &directory)
            {
                return None;
            }
            matched_path = Some(directory);
            continue;
        }

        for count in 1..=remaining.len().min(MAX_CUR_PROJECT_NAME_COMPONENTS) {
            let parts = &remaining[..count];
            let separators: &[&str] = if count == 1 {
                &[""]
            } else {
                &CUR_PROJECT_SEPARATORS
            };
            for separator in separators {
                if probes >= probe_limit {
                    return None;
                }
                probes += 1;
                let candidate = directory.join(parts.join(separator));
                if candidate.is_dir() {
                    pending.push((candidate, consumed + count));
                }
            }
        }
    }

    matched_path
}

#[cfg(any(windows, test))]
fn decode_cur_windows_project_drive(encoded: &str) -> Option<(char, &str)> {
    let drive = encoded.as_bytes().first().copied()?;
    if !drive.is_ascii_alphabetic() || encoded.as_bytes().get(1) != Some(&b'-') {
        return None;
    }

    Some((char::from(drive), encoded.get(2..)?))
}

#[cfg(test)]
#[path = "cur_tests.rs"]
mod tests;
