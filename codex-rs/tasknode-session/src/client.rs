//! Shared HTTP contract for the asynchronous CLI and blocking TUI adapters.
use crate::ActiveSession;
use reqwest::Method;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;
use std::io::Read;
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;

const RESPONSE_LIMIT: usize = 16 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Task Node origin must be an HTTPS server origin or a local HTTP test server.")]
    InvalidOrigin,
    #[error(
        "This Task Node credential belongs to {saved}. Link an account on {requested} before using that server."
    )]
    OriginMismatch { saved: String, requested: String },
    #[error("Task Node request failed: {0}")]
    Transport(String),
    #[error("Task Node returned an unreadable response (HTTP {0}).")]
    InvalidResponse(u16),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    pub status: u16,
    pub body: Value,
}

impl Response {
    pub fn is_ok(&self) -> bool {
        (200..300).contains(&self.status) && self.body.get("ok") != Some(&Value::Bool(false))
    }

    pub fn message(&self) -> String {
        let message = self
            .body
            .get("message")
            .or_else(|| self.body.get("error"))
            .and_then(Value::as_str)
            .unwrap_or("Task Node could not complete this request.");
        if self.status == 401 {
            format!("{message} Relink Task Node in the current profile.")
        } else {
            message.to_string()
        }
    }
}

pub fn normalize_origin(value: &str) -> Result<String, ClientError> {
    let url = Url::parse(value.trim()).map_err(|_| ClientError::InvalidOrigin)?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if (url.scheme() != "https" && !(url.scheme() == "http" && local))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(ClientError::InvalidOrigin);
    }
    Ok(url.origin().ascii_serialization())
}

#[derive(Clone)]
pub struct Client {
    origin: String,
    token: Option<String>,
    account_id: Option<String>,
}

impl Client {
    pub async fn stream(&self, path: &str, body: &Value) -> Result<reqwest::Response, ClientError> {
        static HTTP: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
        let http = HTTP
            .get_or_init(|| {
                reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .connect_timeout(Duration::from_secs(5))
                    .timeout(Duration::from_secs(600))
                    .build()
                    .map_err(|error| error.to_string())
            })
            .as_ref()
            .map_err(|error| ClientError::Transport(error.clone()))?;
        let mut request = http.post(self.url(path)?).json(body);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        request.send().await.map_err(transport_error)
    }

    pub fn stream_blocking(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<reqwest::blocking::Response, ClientError> {
        static HTTP: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();
        let http = HTTP
            .get_or_init(|| {
                reqwest::blocking::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .connect_timeout(Duration::from_secs(5))
                    .timeout(Duration::from_secs(600))
                    .build()
                    .map_err(|error| error.to_string())
            })
            .as_ref()
            .map_err(|error| ClientError::Transport(error.clone()))?;
        let mut request = http.post(self.url(path)?).json(body);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        request.send().map_err(transport_error)
    }

    pub fn for_session(
        session: &ActiveSession,
        requested_origin: &str,
    ) -> Result<Self, ClientError> {
        let saved = normalize_origin(&session.origin)?;
        let requested = normalize_origin(requested_origin)?;
        if saved != requested {
            return Err(ClientError::OriginMismatch { saved, requested });
        }
        Ok(Self {
            origin: saved,
            token: Some(session.terminal_token.clone()),
            account_id: session.account_id.clone(),
        })
    }

    pub fn anonymous(origin: &str) -> Result<Self, ClientError> {
        Ok(Self {
            origin: normalize_origin(origin)?,
            token: None,
            account_id: None,
        })
    }

    pub fn retry_body(&self, request_id: &str, attempt: u64) -> Value {
        serde_json::json!({"phase": "retry", "requestId": request_id,
            "expectedAttemptCount": attempt, "expectedAccountId": self.account_id})
    }

    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// An opaque response fence, including credential rotation but no raw token.
    pub fn identity(&self) -> String {
        let mut digest = Sha256::new();
        for value in [
            self.origin.as_str(),
            self.account_id.as_deref().unwrap_or_default(),
            self.token.as_deref().unwrap_or_default(),
        ] {
            digest.update(value.len().to_le_bytes());
            digest.update(value.as_bytes());
        }
        format!("{:x}", digest.finalize())
    }

    fn url(&self, path: &str) -> Result<Url, ClientError> {
        if !path.starts_with("/api/") || path.contains('\\') {
            return Err(ClientError::InvalidOrigin);
        }
        let url = Url::parse(&format!("{}{path}", self.origin))
            .map_err(|_| ClientError::InvalidOrigin)?;
        if url.origin().ascii_serialization() != self.origin {
            return Err(ClientError::InvalidOrigin);
        }
        Ok(url)
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Response, ClientError> {
        static HTTP: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
        let http = HTTP
            .get_or_init(|| {
                reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .connect_timeout(Duration::from_secs(5))
                    .timeout(REQUEST_TIMEOUT)
                    .build()
                    .map_err(|error| error.to_string())
            })
            .as_ref()
            .map_err(|error| ClientError::Transport(error.clone()))?;
        let mut request = http.request(method, self.url(path)?);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let mut response = request.send().await.map_err(transport_error)?;
        let status = response.status().as_u16();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
            if bytes.len() + chunk.len() > RESPONSE_LIMIT {
                return Err(ClientError::InvalidResponse(status));
            }
            bytes.extend_from_slice(&chunk);
        }
        decode_response(status, &bytes)
    }

    /// Invoke from a blocking worker, never from an async runtime thread.
    pub fn request_blocking(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Response, ClientError> {
        static HTTP: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();
        let http = HTTP
            .get_or_init(|| {
                reqwest::blocking::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .connect_timeout(Duration::from_secs(5))
                    .timeout(REQUEST_TIMEOUT)
                    .build()
                    .map_err(|error| error.to_string())
            })
            .as_ref()
            .map_err(|error| ClientError::Transport(error.clone()))?;
        let mut request = http.request(method, self.url(path)?);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().map_err(transport_error)?;
        let status = response.status().as_u16();
        let mut bytes = Vec::new();
        response
            .take((RESPONSE_LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| ClientError::Transport(error.to_string()))?;
        if bytes.len() > RESPONSE_LIMIT {
            return Err(ClientError::InvalidResponse(status));
        }
        decode_response(status, &bytes)
    }
}

fn decode_response(status: u16, bytes: &[u8]) -> Result<Response, ClientError> {
    let body: Value =
        serde_json::from_slice(bytes).map_err(|_| ClientError::InvalidResponse(status))?;
    if !body.is_object() {
        return Err(ClientError::InvalidResponse(status));
    }
    Ok(Response { status, body })
}

fn transport_error(error: reqwest::Error) -> ClientError {
    ClientError::Transport(if error.is_timeout() {
        "The response timed out. Your saved request can be recovered with its original key."
            .to_string()
    } else {
        error.without_url().to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_never_follow_an_origin_override_or_redirect() {
        let session = ActiveSession {
            origin: "https://tasknode.example/".to_string(),
            account_id: Some("alice".to_string()),
            github_username: None,
            terminal_token: "fixture".to_string(),
            expires_at: None,
        };
        assert!(matches!(
            Client::for_session(&session, "https://other.example"),
            Err(ClientError::OriginMismatch { .. })
        ));
        let client = Client::for_session(&session, "https://tasknode.example").unwrap();
        assert!(client.url("//other.example/api/tasks").is_err());
        assert!(client.url("/api/\\other.example").is_err());
        for origin in [
            "http://external.example",
            "https://user:secret@tasknode.example",
            "https://tasknode.example/path",
            "https://tasknode.example?token=x",
        ] {
            assert!(normalize_origin(origin).is_err());
        }
        for origin in [
            "http://localhost:8080",
            "http://127.0.0.1:8080",
            "http://[::1]:8080",
        ] {
            assert!(normalize_origin(origin).is_ok());
        }
        let mut rotated = session;
        rotated.terminal_token = "rotated".to_string();
        assert_ne!(
            client.identity(),
            Client::for_session(&rotated, &rotated.origin)
                .unwrap()
                .identity()
        );
    }

    #[test]
    fn blocking_transport_refuses_redirect_and_retains_structured_errors() {
        use std::io::Write;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let count = socket.read(&mut request).unwrap();
            assert!(count > 0);
            let body = r#"{"ok":false,"error":"redirect_denied"}"#;
            write!(socket, "HTTP/1.1 302 Found\r\nLocation: https://other.example/api/private\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        });
        let response = Client::anonymous(&origin)
            .unwrap()
            .request_blocking(Method::GET, "/api/test", None)
            .unwrap();
        assert_eq!(response.status, 302);
        assert!(!response.is_ok());
        assert_eq!(response.message(), "redirect_denied");
        server.join().unwrap();
    }
}
