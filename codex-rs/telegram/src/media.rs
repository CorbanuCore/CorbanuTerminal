//! Inbound media support: fetch Telegram photos/image documents to disk so a
//! turn can carry them as `UserInput::LocalImage` items.
//!
//! WHY (2026-07-11): the connector was text-only inbound. That cost real money
//! — a Task Node verifier docked 20% because a requested screenshot could not
//! reach the agent. Screenshots are first-class operator evidence; the agent
//! must be able to see them.
//!
//! Scope: photos and `image/*` documents only. Video, audio, stickers and
//! non-image documents remain unsupported (the caption-with-note path in
//! `bot.rs` still applies to those).

use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use teloxide::Bot;
use teloxide::net::Download;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;
use teloxide::types::FileMeta;
use teloxide::types::Message;
use teloxide::types::PhotoSize;
use tracing::info;

/// Refuse to pull anything bigger than this through the Bot API. Telegram's
/// own `getFile` ceiling is 20 MB; screenshots are far below this.
pub const MAX_IMAGE_BYTES: u32 = 10 * 1024 * 1024;

/// Where fetched images land: `<codex_home>/telegram/media/<chat_id>/…`.
/// Files are kept (not temp) so a resumed/forked session can still serialize
/// the `LocalImage` input and the audit trail can reference the exact bytes.
#[derive(Clone, Debug)]
pub struct MediaStore {
    dir: PathBuf,
}

impl MediaStore {
    pub fn new(codex_home: &Path) -> Self {
        Self {
            dir: codex_home.join("telegram").join("media"),
        }
    }

    async fn save(&self, chat_id: ChatId, name: &str, bytes: &[u8]) -> anyhow::Result<PathBuf> {
        let chat_dir = self.dir.join(chat_id.to_string());
        tokio::fs::create_dir_all(&chat_dir)
            .await
            .with_context(|| format!("create media dir {}", chat_dir.display()))?;
        let path = chat_dir.join(name);
        tokio::fs::write(&path, bytes)
            .await
            .with_context(|| format!("write media file {}", path.display()))?;
        Ok(path)
    }
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

/// Identify, size-check and download every image attached to `message`.
pub async fn fetch_message_images(
    bot: &Bot,
    store: &MediaStore,
    message: &Message,
) -> anyhow::Result<FetchedImages> {
    // (FileMeta to fetch, extension to store under)
    let mut wanted: Vec<(&FileMeta, &'static str)> = Vec::new();

    if let Some(photo) = message.photo().and_then(largest_photo) {
        wanted.push((&photo.file, "jpg"));
    }
    if let Some(doc) = message.document()
        && let Some(ext) = doc
            .mime_type
            .as_ref()
            .and_then(|m| image_ext_from_mime(m.essence_str()))
    {
        wanted.push((&doc.file, ext));
    }

    if wanted.is_empty() {
        return Ok(FetchedImages::None);
    }
    if let Some((meta, _)) = wanted.iter().find(|(meta, _)| meta.size > MAX_IMAGE_BYTES) {
        return Ok(FetchedImages::Rejected(format!(
            "That image is {} MB — I can only take images up to {} MB through Telegram. \
             Upload it somewhere (GitHub comment works) and send the link instead.",
            meta.size / (1024 * 1024),
            MAX_IMAGE_BYTES / (1024 * 1024),
        )));
    }

    let mut paths = Vec::with_capacity(wanted.len());
    for (meta, ext) in wanted {
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
        let name = format!("{}.{ext}", meta.unique_id);
        let path = store.save(message.chat.id, &name, &bytes).await?;
        info!(path = %path.display(), bytes = bytes.len(), "fetched inbound Telegram image");
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
        let photos = vec![photo(1280, 720, 90_000, "b"), photo(320, 180, 8_000, "a")];
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
}
