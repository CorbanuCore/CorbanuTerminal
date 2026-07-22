use std::path::Path;

use anyhow::Context;

pub(crate) async fn write_atomically(path: &Path, contents: String) -> anyhow::Result<()> {
    let path = path.to_path_buf();
    let display = path.display().to_string();
    tokio::task::spawn_blocking(move || codex_utils_path::write_atomically(&path, &contents))
        .await
        .context("atomic state writer task failed")?
        .with_context(|| format!("atomically write {display}"))
}
