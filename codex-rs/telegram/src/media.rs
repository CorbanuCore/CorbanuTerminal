//! Inbound media support: fetch Telegram photos/image documents to disk so a
//! turn can carry them as `UserInput::LocalImage` items.
//!
//! WHY (2026-07-11): the connector was text-only inbound. That cost real money
//! — a Task Node verifier docked 20% because a requested screenshot could not
//! reach the agent. Screenshots are first-class operator evidence; the agent
//! must be able to see them.
//!
//! Scope: photos, image documents, and bounded inspectable documents. Video,
//! audio, stickers, archives, executables, and unknown formats remain
//! unsupported.

use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use teloxide::Bot;
use teloxide::net::Download;
use teloxide::prelude::Requester;
use teloxide::types::FileMeta;
use teloxide::types::Message;
use teloxide::types::PhotoSize;
use tracing::info;

use crate::conversation::ConversationKey;

/// Refuse to pull anything bigger than this through the Bot API. Telegram's
/// own `getFile` ceiling is 20 MB; screenshots are far below this.
pub const DEFAULT_MAX_ATTACHMENT_BYTES: u32 = 10 * 1024 * 1024;
const DEFAULT_MEDIA_RETENTION: std::time::Duration =
    std::time::Duration::from_secs(7 * 24 * 60 * 60);
const DEFAULT_MAX_MEDIA_STORE_BYTES: u64 = 256 * 1024 * 1024;

/// Where fetched images land: `<codex_home>/telegram/media/<chat_id>/…`.
/// Files are kept (not temp) so a resumed/forked session can still serialize
/// the `LocalImage` input and the audit trail can reference the exact bytes.
#[derive(Clone, Debug)]
pub struct MediaStore {
    dir: PathBuf,
    max_attachment_bytes: u32,
    retention: std::time::Duration,
    max_store_bytes: u64,
}

impl MediaStore {
    pub fn new(codex_home: &Path) -> Self {
        Self {
            dir: codex_home.join("telegram").join("media"),
            max_attachment_bytes: DEFAULT_MAX_ATTACHMENT_BYTES,
            retention: DEFAULT_MEDIA_RETENTION,
            max_store_bytes: DEFAULT_MAX_MEDIA_STORE_BYTES,
        }
    }

    pub fn with_limits(
        codex_home: &Path,
        max_attachment_bytes: u32,
        media_retention_days: u64,
        max_store_bytes: u64,
    ) -> Self {
        Self {
            dir: codex_home.join("telegram").join("media"),
            max_attachment_bytes,
            retention: std::time::Duration::from_secs(
                media_retention_days.saturating_mul(24 * 60 * 60),
            ),
            max_store_bytes,
        }
    }

    async fn save(
        &self,
        message: &Message,
        name: &str,
        original_name: &str,
        mime_type: &str,
        bytes: &[u8],
    ) -> anyhow::Result<PathBuf> {
        let conversation = ConversationKey::from_message(message);
        let chat_dir = self.dir.join(conversation.storage_key());
        tokio::fs::create_dir_all(&chat_dir)
            .await
            .with_context(|| format!("create media dir {}", chat_dir.display()))?;
        let path = chat_dir.join(name);
        tokio::fs::write(&path, bytes)
            .await
            .with_context(|| format!("write media file {}", path.display()))?;
        let received_at_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());
        let metadata = StoredMediaMetadata {
            source_conversation: conversation.storage_key(),
            original_name: original_name.to_string(),
            mime_type: mime_type.to_string(),
            size: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(bytes)),
            received_at_unix,
            expires_at_unix: received_at_unix.saturating_add(self.retention.as_secs()),
        };
        let metadata_path = metadata_path_for(&path);
        if let Err(err) =
            tokio::fs::write(&metadata_path, serde_json::to_vec_pretty(&metadata)?).await
        {
            let _ = tokio::fs::remove_file(&path).await;
            return Err(err)
                .with_context(|| format!("write media metadata {}", metadata_path.display()));
        }
        self.cleanup_expired().await;
        Ok(path)
    }

    pub(crate) async fn cleanup_expired(&self) {
        self.cleanup_with_limits(self.retention, self.max_store_bytes)
            .await;
    }

    async fn cleanup_with_limits(&self, retention: std::time::Duration, max_bytes: u64) {
        let Ok(mut chats) = tokio::fs::read_dir(&self.dir).await else {
            return;
        };
        let cutoff = std::time::SystemTime::now().checked_sub(retention);
        let mut retained = Vec::new();
        while let Ok(Some(chat)) = chats.next_entry().await {
            let Ok(mut files) = tokio::fs::read_dir(chat.path()).await else {
                continue;
            };
            while let Ok(Some(file)) = files.next_entry().await {
                let path = file.path();
                if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".metadata.json"))
                {
                    if let Some(payload_path) = payload_path_for_metadata(&path)
                        && !payload_path.exists()
                    {
                        let _ = tokio::fs::remove_file(path).await;
                    }
                    continue;
                }
                let Ok(metadata) = file.metadata().await else {
                    continue;
                };
                let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
                let metadata_path = metadata_path_for(&path);
                let metadata_bytes = tokio::fs::metadata(&metadata_path)
                    .await
                    .map_or(0, |metadata| metadata.len());
                if cutoff.is_some_and(|cutoff| modified < cutoff) {
                    let _ = tokio::fs::remove_file(file.path()).await;
                    let _ = tokio::fs::remove_file(metadata_path).await;
                } else {
                    retained.push(MediaArtifact {
                        modified,
                        bytes: metadata.len().saturating_add(metadata_bytes),
                        payload_path: path,
                        metadata_path,
                    });
                }
            }
        }
        let mut total = retained.iter().map(|artifact| artifact.bytes).sum::<u64>();
        retained.sort_by_key(|artifact| artifact.modified);
        for artifact in retained {
            if total <= max_bytes {
                break;
            }
            if tokio::fs::remove_file(&artifact.payload_path).await.is_ok() {
                let _ = tokio::fs::remove_file(&artifact.metadata_path).await;
                total = total.saturating_sub(artifact.bytes);
            }
        }
    }
}

#[derive(Debug)]
struct MediaArtifact {
    modified: std::time::SystemTime,
    bytes: u64,
    payload_path: PathBuf,
    metadata_path: PathBuf,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
struct StoredMediaMetadata {
    source_conversation: String,
    original_name: String,
    mime_type: String,
    size: u64,
    sha256: String,
    received_at_unix: u64,
    expires_at_unix: u64,
}

fn metadata_path_for(payload_path: &Path) -> PathBuf {
    let extension = payload_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("bin");
    payload_path.with_extension(format!("{extension}.metadata.json"))
}

fn payload_path_for_metadata(metadata_path: &Path) -> Option<PathBuf> {
    let file_name = metadata_path.file_name()?.to_str()?;
    let payload_name = file_name.strip_suffix(".metadata.json")?;
    Some(metadata_path.with_file_name(payload_name))
}

fn stable_media_name(unique_id: &str, extension: &str) -> String {
    let digest = format!("{:x}", Sha256::digest(unique_id.as_bytes()));
    format!("{}.{}", &digest[..24], extension.to_ascii_lowercase())
}

/// The largest photo variant Telegram offers for a message. Telegram sends
/// several `PhotoSize`s per photo; do not assume any ordering.
pub fn largest_photo(photos: &[PhotoSize]) -> Option<&PhotoSize> {
    photos
        .iter()
        .max_by_key(|p| u64::from(p.width) * u64::from(p.height))
}

/// Map an image mime type to the file extension we store. Non-image (or
/// unknown) mime types return `None` and stay on the unsupported path.
pub fn image_ext_from_mime(mime: &str) -> Option<&'static str> {
    match mime {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        _ => None,
    }
}

/// What the media layer concluded about one inbound message.
pub enum FetchedImages {
    /// No photo / image document present at all.
    None,
    /// Image(s) present and fetched to disk.
    Images(Vec<PathBuf>),
    /// An image was present but could not be accepted (too large). The string
    /// is a user-facing reason.
    Rejected(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedDocument {
    pub path: PathBuf,
    pub original_name: String,
    pub mime_type: String,
    pub size: u32,
    pub sha256: String,
}

pub enum DocumentFetch {
    None,
    Document(FetchedDocument),
    Rejected(String),
}

pub fn document_mime_allowed(mime: &str) -> bool {
    mime.starts_with("text/")
        || matches!(
            mime,
            "application/json" | "application/pdf" | "application/xml" | "application/yaml"
        )
}

pub async fn fetch_message_document(
    bot: &Bot,
    store: &MediaStore,
    message: &Message,
) -> anyhow::Result<DocumentFetch> {
    let Some(document) = message.document() else {
        return Ok(DocumentFetch::None);
    };
    let mime_type = document
        .mime_type
        .as_ref()
        .map(|mime| mime.essence_str().to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    if image_ext_from_mime(&mime_type).is_some() {
        return Ok(DocumentFetch::None);
    }
    if !document_mime_allowed(&mime_type) {
        return Ok(DocumentFetch::Rejected(format!(
            "I can't safely ingest `{mime_type}` files. Send text, source, JSON, PDF, XML, or YAML instead; archives and executables are rejected."
        )));
    }
    if document.file.size > store.max_attachment_bytes {
        return Ok(DocumentFetch::Rejected(format!(
            "That file is {} MB — Telegram document ingestion is limited to {} MB.",
            document.file.size / (1024 * 1024),
            store.max_attachment_bytes / (1024 * 1024)
        )));
    }

    store.cleanup_expired().await;
    let file = crate::outbound::call_with_policy(
        crate::outbound::CallSafety::Idempotent,
        crate::outbound::MEDIA_API_TIMEOUT,
        "telegram getFile",
        || {
            let bot = bot.clone();
            let file_id = document.file.id.clone();
            async move { bot.get_file(file_id).await }
        },
    )
    .await
    .context("telegram document getFile failed")?;
    let file_path = file.path.clone();
    let bytes = crate::outbound::call_with_policy(
        crate::outbound::CallSafety::Idempotent,
        crate::outbound::MEDIA_API_TIMEOUT,
        "telegram document download",
        || {
            let bot = bot.clone();
            let file_path = file_path.clone();
            async move {
                let mut bytes = Vec::new();
                bot.download_file(&file_path, &mut bytes).await?;
                Ok(bytes)
            }
        },
    )
    .await
    .context("telegram document download failed")?;
    if bytes.len() > store.max_attachment_bytes as usize {
        return Ok(DocumentFetch::Rejected(format!(
            "That file downloaded as more than {} MB, so it was rejected.",
            store.max_attachment_bytes / (1024 * 1024)
        )));
    }
    let original_name = document
        .file_name
        .clone()
        .unwrap_or_else(|| "attachment".to_string());
    let extension = Path::new(&original_name)
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| extension.chars().all(|ch| ch.is_ascii_alphanumeric()))
        .unwrap_or("bin");
    let stored_name = stable_media_name(&document.file.unique_id.0, extension);
    let path = store
        .save(message, &stored_name, &original_name, &mime_type, &bytes)
        .await?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    info!(
        conversation = %ConversationKey::from_message(message).redacted_id(),
        bytes = bytes.len(),
        %sha256,
        "fetched inbound Telegram document"
    );
    Ok(DocumentFetch::Document(FetchedDocument {
        path,
        original_name,
        mime_type,
        size: document.file.size,
        sha256,
    }))
}

/// Identify, size-check and download every image attached to `message`.
pub async fn fetch_message_images(
    bot: &Bot,
    store: &MediaStore,
    message: &Message,
) -> anyhow::Result<FetchedImages> {
    store.cleanup_expired().await;
    // (FileMeta to fetch, extension to store under)
    let mut wanted: Vec<(&FileMeta, &'static str, String, String)> = Vec::new();

    if let Some(photo) = message.photo().and_then(largest_photo) {
        wanted.push((
            &photo.file,
            "jpg",
            "telegram-photo.jpg".to_string(),
            "image/jpeg".to_string(),
        ));
    }
    if let Some(doc) = message.document()
        && let Some(ext) = doc
            .mime_type
            .as_ref()
            .and_then(|m| image_ext_from_mime(m.essence_str()))
    {
        wanted.push((
            &doc.file,
            ext,
            doc.file_name
                .clone()
                .unwrap_or_else(|| format!("telegram-image.{ext}")),
            doc.mime_type
                .as_ref()
                .map(|mime| mime.essence_str().to_string())
                .unwrap_or_else(|| format!("image/{ext}")),
        ));
    }

    if wanted.is_empty() {
        return Ok(FetchedImages::None);
    }
    if let Some((meta, ..)) = wanted
        .iter()
        .find(|(meta, ..)| meta.size > store.max_attachment_bytes)
    {
        return Ok(FetchedImages::Rejected(format!(
            "That image is {} MB — I can only take images up to {} MB through Telegram. \
             Upload it somewhere (GitHub comment works) and send the link instead.",
            meta.size / (1024 * 1024),
            store.max_attachment_bytes / (1024 * 1024),
        )));
    }

    let mut paths = Vec::with_capacity(wanted.len());
    for (meta, ext, original_name, mime_type) in wanted {
        // Both calls are idempotent reads, so they are safe to retry under
        // the outbound policy: 429 honors `retry_after`, 5xx/transport use
        // bounded backoff, and the whole fetch is time-boxed so a stalled
        // download degrades to the caption-note path instead of wedging.
        let file = crate::outbound::call_with_policy(
            crate::outbound::CallSafety::Idempotent,
            crate::outbound::MEDIA_API_TIMEOUT,
            "telegram getFile",
            || {
                let bot = bot.clone();
                let file_id = meta.id.clone();
                async move { bot.get_file(file_id).await }
            },
        )
        .await
        .context("telegram getFile failed")?;
        let file_path = file.path.clone();
        let bytes = crate::outbound::call_with_policy(
            crate::outbound::CallSafety::Idempotent,
            crate::outbound::MEDIA_API_TIMEOUT,
            "telegram file download",
            || {
                let bot = bot.clone();
                let file_path = file_path.clone();
                let mut bytes = Vec::with_capacity(meta.size as usize);
                async move {
                    bot.download_file(&file_path, &mut bytes).await?;
                    Ok(bytes)
                }
            },
        )
        .await
        .context("telegram file download failed")?;
        if bytes.len() > store.max_attachment_bytes as usize {
            return Ok(FetchedImages::Rejected(format!(
                "That image downloaded as more than {} MB, so it was rejected.",
                store.max_attachment_bytes / (1024 * 1024)
            )));
        }
        let name = stable_media_name(&meta.unique_id.0, ext);
        let path = store
            .save(message, &name, &original_name, &mime_type, &bytes)
            .await?;
        info!(
            conversation = %ConversationKey::from_message(message).redacted_id(),
            bytes = bytes.len(),
            "fetched inbound Telegram image"
        );
        paths.push(path);
    }
    Ok(FetchedImages::Images(paths))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn photo(width: u32, height: u32, size: u32, id: &str) -> PhotoSize {
        use teloxide::types::FileId;
        use teloxide::types::FileUniqueId;
        PhotoSize {
            file: FileMeta {
                id: FileId(id.to_string()),
                unique_id: FileUniqueId(format!("u{id}")),
                size,
            },
            width,
            height,
        }
    }

    #[test]
    fn largest_photo_picks_by_pixel_area_not_order() {
        let photos = vec![photo(/*width*/ 1280, /*height*/ 720, /*size*/ 90_000, "b"), photo(/*width*/ 320, /*height*/ 180, /*size*/ 8_000, "a")];
        assert_eq!(largest_photo(&photos).unwrap().width, 1280);
    }

    #[test]
    fn largest_photo_empty_is_none() {
        assert!(largest_photo(&[]).is_none());
    }

    #[test]
    fn image_mimes_map_to_extensions_and_others_do_not() {
        assert_eq!(image_ext_from_mime("image/png"), Some("png"));
        assert_eq!(image_ext_from_mime("image/jpeg"), Some("jpg"));
        assert_eq!(image_ext_from_mime("application/pdf"), None);
        assert_eq!(image_ext_from_mime("video/mp4"), None);
    }

    #[test]
    fn document_allowlist_is_bounded_to_inspectable_formats() {
        assert!(document_mime_allowed("text/plain"));
        assert!(document_mime_allowed("application/json"));
        assert!(document_mime_allowed("application/pdf"));
        assert!(!document_mime_allowed("application/zip"));
        assert!(!document_mime_allowed("application/x-tar"));
        assert!(!document_mime_allowed("application/gzip"));
        assert!(!document_mime_allowed("application/x-msdownload"));
        assert!(!document_mime_allowed("video/mp4"));
    }

    #[test]
    fn stored_names_are_stable_and_cannot_escape_the_media_directory() {
        let first = stable_media_name("../../remote/id", "PDF");
        let second = stable_media_name("../../remote/id", "PDF");
        assert_eq!(first, second);
        assert!(first.ends_with(".pdf"));
        assert!(!first.contains('/'));
        assert!(!first.contains(".."));
    }

    #[test]
    fn metadata_paths_round_trip_to_their_payload() {
        let payload = PathBuf::from("/tmp/file.txt");
        let metadata = metadata_path_for(&payload);
        assert_eq!(
            payload_path_for_metadata(&metadata).as_deref(),
            Some(payload.as_path())
        );
    }

    #[tokio::test]
    async fn byte_cap_removes_payload_and_metadata_as_one_artifact() {
        let root = std::env::temp_dir().join(format!(
            "pfterminal-telegram-media-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = MediaStore::new(&root);
        let chat_dir = store.dir.join("42");
        tokio::fs::create_dir_all(&chat_dir).await.unwrap();
        let payload = chat_dir.join("payload.txt");
        let metadata = metadata_path_for(&payload);
        tokio::fs::write(&payload, b"payload").await.unwrap();
        tokio::fs::write(&metadata, b"metadata").await.unwrap();

        store.cleanup_with_limits(std::time::Duration::MAX, /*max_bytes*/ 0).await;

        assert!(!payload.exists());
        assert!(!metadata.exists());
        tokio::fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn cleanup_removes_orphaned_metadata() {
        let root = std::env::temp_dir().join(format!(
            "pfterminal-telegram-media-orphan-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = MediaStore::new(&root);
        let chat_dir = store.dir.join("42");
        tokio::fs::create_dir_all(&chat_dir).await.unwrap();
        let metadata = chat_dir.join("missing.txt.metadata.json");
        tokio::fs::write(&metadata, b"metadata").await.unwrap();

        store
            .cleanup_with_limits(std::time::Duration::MAX, DEFAULT_MAX_MEDIA_STORE_BYTES)
            .await;

        assert!(!metadata.exists());
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
