use crate::BrowserError;
use crate::engine::ContainerEngine;
use crate::engine::strings;
use sha2::Digest;
use sha2::Sha256;
use std::time::Duration;

pub(crate) const BASE_IMAGE: &str = "ghcr.io/d4vinci/scrapling@sha256:1bacbc8ec90b3090d462e12f6555e241daf0dfeb684ab326ffa09d52d8226e69";
const RECIPE: &str = include_str!("../worker/Dockerfile");
pub(crate) const WORKER: &str = include_str!("../worker/worker.py");

pub(crate) fn recipe_digest() -> String {
    let mut hash = Sha256::new();
    hash.update(RECIPE.as_bytes());
    hash.update([0]);
    hash.update(WORKER.as_bytes());
    format!("{:x}", hash.finalize())
}

pub(crate) fn is_image_id(id: &str) -> bool {
    // Docker prefixes image IDs; Podman returns bare hashes. Both are immutable
    // engine-native IDs accepted by create and normalized by verify_owner.
    let digest = id.strip_prefix("sha256:").unwrap_or(id);
    digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod tests;

impl ContainerEngine {
    pub(crate) async fn prepare_image(&self) -> Result<String, BrowserError> {
        let recipe = recipe_digest();
        let tag = format!("localhost/corbanu-browser:recipe-{recipe}");
        // A local tag and image labels are writable by every principal with
        // access to the selected engine. Never treat either as a trusted cache
        // key: rebuild from the checked-in recipe and pinned base, then bind
        // this runtime to the immutable ID returned by that exact build.
        if self
            .json(&strings(&["image", "inspect", BASE_IMAGE]))
            .await
            .is_err()
        {
            self.command
                .run(&strings(&["pull", BASE_IMAGE]), Duration::from_secs(300))
                .await
                .map_err(|_| BrowserError::ImageUnavailable)?;
        }
        let context = tempfile::tempdir().map_err(|_| BrowserError::ImageUnavailable)?;
        tokio::fs::write(context.path().join("Dockerfile"), RECIPE)
            .await
            .map_err(|_| BrowserError::ImageUnavailable)?;
        tokio::fs::write(context.path().join("worker.py"), WORKER)
            .await
            .map_err(|_| BrowserError::ImageUnavailable)?;
        let build_output = self
            .command
            .run(
                &strings(&[
                    "build",
                    "--network=none",
                    "--pull=false",
                    "--no-cache",
                    "--quiet",
                    "--label",
                    &format!("org.corbanu.browser.recipe={recipe}"),
                    "--tag",
                    &tag,
                    &context.path().to_string_lossy(),
                ]),
                Duration::from_secs(300),
            )
            .await
            .map_err(|_| BrowserError::ImageUnavailable)?;
        let built_id = std::str::from_utf8(&build_output)
            .map(str::trim)
            .ok()
            .filter(|id| is_image_id(id))
            .ok_or(BrowserError::ContainerMismatch)?;
        let image = self
            .json(&strings(&["image", "inspect", built_id]))
            .await
            .map_err(|_| BrowserError::ImageUnavailable)?;
        let image = image.get(0).ok_or(BrowserError::ImageUnavailable)?;
        let id = image["Id"].as_str().ok_or(BrowserError::ImageUnavailable)?;
        if !is_image_id(id)
            || !id
                .trim_start_matches("sha256:")
                .eq_ignore_ascii_case(built_id.trim_start_matches("sha256:"))
            || image
                .pointer("/Config/Labels/org.corbanu.browser.recipe")
                .and_then(|v| v.as_str())
                != Some(&recipe)
            || image.pointer("/Config/User").and_then(|v| v.as_str()) != Some("65532:65532")
            || image.pointer("/Config/Entrypoint")
                != Some(&serde_json::json!([
                    "/app/.venv/bin/python",
                    "-I",
                    "-u",
                    "/opt/corbanu/worker.py"
                ]))
        {
            return Err(BrowserError::ContainerMismatch);
        }
        Ok(id.to_owned())
    }
}
