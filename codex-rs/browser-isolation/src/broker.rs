use crate::BrowserError;
use crate::QuarantinedArtifact;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use codex_network_proxy::NetworkProxyState;
use codex_network_proxy::browser_policy::resolve_browser_destination;
use codex_security_policy::AuthorityEpoch;
use reqwest::header::CONTENT_DISPOSITION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::LOCATION;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncBufReadExt;

pub(crate) const MAX_BODY: usize = 2 * 1024 * 1024;
const MAX_TOTAL: usize = 16 * 1024 * 1024;
const MAX_LINE: usize = 6 * 1024 * 1024;

/// Each client serves one request. Its resolver has no system-DNS fallback,
/// including when a transport normalizes the hostname differently from URL.
struct PinnedAddresses(Vec<SocketAddr>);

impl reqwest::dns::Resolve for PinnedAddresses {
    fn resolve(&self, _name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let addresses = self.0.clone();
        Box::pin(std::future::ready(Ok(
            Box::new(addresses.into_iter()) as reqwest::dns::Addrs
        )))
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WorkerMessage {
    Request { id: u32, url: String },
    Result { url: String, body: String },
    Failed {},
}

#[derive(Serialize)]
pub(crate) struct Reply {
    id: u32,
    denied: bool,
    status: u16,
    headers: BTreeMap<String, String>,
    body: String,
}

impl Reply {
    fn denied(id: u32) -> Self {
        Self {
            id,
            denied: true,
            status: 403,
            headers: BTreeMap::new(),
            body: String::new(),
        }
    }
}

pub(crate) struct Broker<'a> {
    policy: &'a NetworkProxyState,
    epoch: AuthorityEpoch,
    sequence: u32,
    redirects: usize,
    bytes: usize,
    pub artifacts: Vec<QuarantinedArtifact>,
    pub visited: Vec<String>,
}

impl<'a> Broker<'a> {
    pub fn new(policy: &'a NetworkProxyState, epoch: AuthorityEpoch) -> Self {
        Self {
            policy,
            epoch,
            sequence: 0,
            redirects: 0,
            bytes: 0,
            artifacts: Vec::new(),
            visited: Vec::new(),
        }
    }

    pub async fn request(
        &mut self,
        id: u32,
        url: &str,
        check_authority: impl Fn() -> Result<(), BrowserError>,
    ) -> Result<Reply, BrowserError> {
        if self.sequence >= 64 || id != self.sequence + 1 {
            return Err(BrowserError::ResourceLimit);
        }
        self.sequence = id;
        // Invalid/subresource destinations are aborted, not silently fetched by
        // another stack. The container cannot fall back to its own network.
        let Ok(destination) = resolve_browser_destination(url, self.policy).await else {
            return Ok(Reply::denied(id));
        };
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .pool_max_idle_per_host(0)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .dns_resolver(Arc::new(PinnedAddresses(destination.addresses().to_vec())))
            .build()
            .map_err(|_| BrowserError::FetchFailed)?;
        check_authority()?;
        let mut response = client
            .get(destination.url().clone())
            .header("User-Agent", "Corbanu-Public-Web/1")
            .header("Accept-Encoding", "identity")
            .send()
            .await
            .map_err(|_| BrowserError::FetchFailed)?;
        // Defense in depth: never accept a connection to an address other than
        // the exact validated DNS answer. Normal TLS hostname checks stay enabled.
        if !response
            .remote_addr()
            .is_some_and(|remote| destination.addresses().contains(&remote))
        {
            return Err(BrowserError::DestinationDenied);
        }
        let status = response.status().as_u16();
        let mut headers = BTreeMap::from([("cache-control".to_owned(), "no-store".to_owned())]);
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream");
        if content_type.len() > 256 {
            return Err(BrowserError::ResourceLimit);
        }
        headers.insert("content-type".to_owned(), content_type.to_owned());
        if response.status().is_redirection() {
            self.redirects += 1;
            if self.redirects > 8 {
                return Err(BrowserError::ResourceLimit);
            }
            let location = response
                .headers()
                .get(LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or(BrowserError::FetchFailed)?;
            if location.len() > 4096 {
                return Err(BrowserError::ResourceLimit);
            }
            let target = destination
                .url()
                .join(location)
                .map_err(|_| BrowserError::DestinationDenied)?;
            // Validate now, then resolve/bind again when the browser follows it.
            if resolve_browser_destination(target.as_str(), self.policy)
                .await
                .is_err()
            {
                return Ok(Reply::denied(id));
            }
            headers.insert("location".to_owned(), target.into());
        }
        let attachment = !response.status().is_redirection()
            && is_download(
                content_type,
                response
                    .headers()
                    .get(CONTENT_DISPOSITION)
                    .and_then(|v| v.to_str().ok()),
            );
        if response
            .content_length()
            .is_some_and(|length| length > MAX_BODY as u64)
        {
            return Err(BrowserError::ResourceLimit);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| BrowserError::FetchFailed)?
        {
            if bytes.len() + chunk.len() > MAX_BODY || self.bytes + chunk.len() > MAX_TOTAL {
                return Err(BrowserError::ResourceLimit);
            }
            self.bytes += chunk.len();
            bytes.extend_from_slice(&chunk);
        }
        self.visited.push(destination.url().to_string());
        if attachment {
            if self.artifacts.len() >= 8 {
                return Err(BrowserError::ResourceLimit);
            }
            self.artifacts
                .push(QuarantinedArtifact::new(bytes, self.epoch));
            return Ok(Reply::denied(id));
        }
        Ok(Reply {
            id,
            denied: false,
            status,
            headers,
            body: STANDARD.encode(bytes),
        })
    }
}

fn is_download(content_type: &str, disposition: Option<&str>) -> bool {
    if disposition.is_some_and(|v| {
        v.split(';')
            .next()
            .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("attachment"))
    }) {
        return true;
    }
    let media = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    !["text/", "image/", "font/", "audio/", "video/"]
        .iter()
        .any(|prefix| media.starts_with(prefix))
        && !matches!(
            media.as_str(),
            "application/javascript"
                | "application/json"
                | "application/xml"
                | "application/xhtml+xml"
                | "application/wasm"
        )
}

pub(crate) async fn read_message(
    reader: &mut (impl AsyncBufRead + Unpin),
) -> Result<WorkerMessage, BrowserError> {
    let mut line = Vec::new();
    // fill_buf/consume bounds allocation before reading an untrusted line. A
    // read_line then length check permits an attacker to exhaust host memory.
    loop {
        let buffer = reader
            .fill_buf()
            .await
            .map_err(|_| BrowserError::InvalidWorkerResponse)?;
        if buffer.is_empty() {
            return Err(BrowserError::InvalidWorkerResponse);
        }
        let count = buffer
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(buffer.len(), |pos| pos + 1);
        if line.len() + count > MAX_LINE {
            return Err(BrowserError::ResourceLimit);
        }
        let finished = buffer[count - 1] == b'\n';
        line.extend_from_slice(&buffer[..count]);
        reader.consume(count);
        if finished {
            break;
        }
    }
    serde_json::from_slice(&line).map_err(|_| BrowserError::InvalidWorkerResponse)
}

pub(crate) fn decode_body(body: &str) -> Result<Vec<u8>, BrowserError> {
    if body.len() > MAX_BODY.div_ceil(3) * 4 {
        return Err(BrowserError::ResourceLimit);
    }
    let bytes = STANDARD
        .decode(body)
        .map_err(|_| BrowserError::InvalidWorkerResponse)?;
    if bytes.len() > MAX_BODY {
        return Err(BrowserError::ResourceLimit);
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "broker_tests.rs"]
mod tests;
