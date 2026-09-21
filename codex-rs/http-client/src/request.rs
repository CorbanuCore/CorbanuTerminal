use bytes::Bytes;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

/// A JSON request body serialized once into reference-counted bytes.
///
/// Clones share the encoded allocation. Internally, the body can also hold the
/// final compressed wire bytes. Request contents are never retained solely for
/// trace logging; prompts and credentials do not belong in diagnostic logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedJsonBody {
    bytes: Bytes,
    prepared: bool,
}

impl EncodedJsonBody {
    /// Serializes `value` into a reusable JSON body.
    pub fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Self, serde_json::Error> {
        serde_json::to_vec(value).map(|bytes| Self {
            bytes: Bytes::from(bytes),
            prepared: false,
        })
    }

    /// Returns the encoded bytes currently stored by this body.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RequestCompression {
    #[default]
    None,
    Zstd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestBody {
    Json(Value),
    EncodedJson(EncodedJsonBody),
    Raw(Bytes),
}

impl RequestBody {
    pub fn json(&self) -> Option<&Value> {
        match self {
            Self::Json(value) => Some(value),
            Self::EncodedJson(_) | Self::Raw(_) => None,
        }
    }

    /// Returns the body as JSON for callers that must inspect routing keys.
    ///
    /// A prepared body holds the final wire bytes, which for this client means
    /// they may already be zstd-compressed; only this module knows that, so the
    /// decoding lives here rather than in every caller. `None` means the body
    /// genuinely cannot be read as JSON, which callers must treat as
    /// uninspectable rather than as an absence of keys.
    pub fn inspectable_json(&self) -> Option<Value> {
        let bytes = match self {
            Self::Json(value) => return Some(value.clone()),
            Self::EncodedJson(body) => body.as_bytes(),
            Self::Raw(bytes) => bytes.as_ref(),
        };
        if let Ok(value) = serde_json::from_slice::<Value>(bytes) {
            return Some(value);
        }
        let plain = zstd::stream::decode_all(std::io::Cursor::new(bytes)).ok()?;
        serde_json::from_slice(&plain).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRequestBody {
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
}

impl PreparedRequestBody {
    pub fn body_bytes(&self) -> Bytes {
        self.body.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<RequestBody>,
    pub compression: RequestCompression,
    pub timeout: Option<Duration>,
}

impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            compression: RequestCompression::None,
            timeout: None,
        }
    }

    /// Every JSON document this request's body carries, for callers that must
    /// inspect routing keys.
    ///
    /// Almost every body is a single JSON document. A multipart body is not
    /// JSON as a whole - realtime call creation builds one, an SDP part beside
    /// a session JSON part - so reading it as one document yields nothing, and
    /// a caller that treats nothing as uninspectable refuses the whole route.
    /// The JSON parts are readable, so return them and let the caller inspect
    /// what is actually there. `None` still means the body could not be read,
    /// which callers must treat as uninspectable rather than as an absence of
    /// keys; an absent body is an empty list, because it carries no keys.
    pub fn inspectable_json_documents(&self) -> Option<Vec<Value>> {
        let Some(body) = self.body.as_ref() else {
            return Some(Vec::new());
        };
        if let Some(value) = body.inspectable_json() {
            return Some(vec![value]);
        }
        let boundary = self.multipart_boundary()?;
        let RequestBody::Raw(bytes) = body else {
            return None;
        };
        let text = std::str::from_utf8(bytes).ok()?;
        let mut documents = Vec::new();
        for part in text.split(&format!("--{boundary}")).skip(1) {
            let Some((headers, content)) = part.split_once("\r\n\r\n") else {
                continue;
            };
            if !headers
                .to_ascii_lowercase()
                .contains("content-type: application/json")
            {
                continue;
            }
            // A part that says it is JSON and is not means this body is not the
            // shape it claims; that is uninspectable, not empty.
            documents.push(serde_json::from_str(content.trim_end_matches("\r\n")).ok()?);
        }
        Some(documents)
    }

    /// The boundary of a multipart body, from the content type this request
    /// declares. `None` for every other content type.
    fn multipart_boundary(&self) -> Option<String> {
        let content_type = self
            .headers
            .get(http::header::CONTENT_TYPE)?
            .to_str()
            .ok()?;
        let (kind, parameters) = content_type.split_once(';')?;
        if !kind.trim().eq_ignore_ascii_case("multipart/form-data") {
            return None;
        }
        parameters.split(';').find_map(|parameter| {
            let (name, value) = parameter.split_once('=')?;
            name.trim()
                .eq_ignore_ascii_case("boundary")
                .then(|| value.trim().trim_matches('"').to_string())
        })
    }

    pub fn with_json<T: Serialize>(mut self, body: &T) -> Self {
        self.body = serde_json::to_value(body).ok().map(RequestBody::Json);
        self
    }

    pub fn with_raw_body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = Some(RequestBody::Raw(body.into()));
        self
    }

    pub fn with_compression(mut self, compression: RequestCompression) -> Self {
        self.compression = compression;
        self
    }

    /// Prepares the body once and stores the exact bytes that will be sent.
    ///
    /// Cloning the returned request shares the body bytes, so retry attempts do
    /// not repeat JSON serialization or compression. Request-signing auth also
    /// sees the same final headers and bytes that the transport will send.
    pub fn into_prepared(mut self) -> Result<Self, String> {
        let is_json = matches!(
            self.body,
            Some(RequestBody::Json(_) | RequestBody::EncodedJson(_))
        );
        let prepared = self.prepare_body_for_send()?;
        self.headers = prepared.headers;
        self.body = match (is_json, prepared.body) {
            (true, Some(bytes)) => Some(RequestBody::EncodedJson(EncodedJsonBody {
                bytes,
                prepared: true,
            })),
            (false, Some(body)) => Some(RequestBody::Raw(body)),
            (_, None) => None,
        };
        self.compression = RequestCompression::None;
        Ok(self)
    }

    /// Convert the request body into the exact bytes that will be sent.
    ///
    /// Auth schemes such as AWS SigV4 need to sign the final body bytes, including
    /// compression and content headers. Calling this method does not mutate the
    /// request.
    pub fn prepare_body_for_send(&self) -> Result<PreparedRequestBody, String> {
        let headers = self.headers.clone();
        match self.body.as_ref() {
            Some(RequestBody::Raw(raw_body)) => {
                if self.compression != RequestCompression::None {
                    return Err("request compression cannot be used with raw bodies".to_string());
                }
                Ok(PreparedRequestBody {
                    headers,
                    body: Some(raw_body.clone()),
                })
            }
            Some(RequestBody::Json(body)) => {
                let body = EncodedJsonBody::encode(body).map_err(|err| err.to_string())?;
                self.prepare_encoded_json(headers, &body)
            }
            Some(RequestBody::EncodedJson(body)) => self.prepare_encoded_json(headers, body),
            None => Ok(PreparedRequestBody {
                headers,
                body: None,
            }),
        }
    }

    fn prepare_encoded_json(
        &self,
        mut headers: HeaderMap,
        body: &EncodedJsonBody,
    ) -> Result<PreparedRequestBody, String> {
        if body.prepared {
            return Ok(PreparedRequestBody {
                headers,
                body: Some(body.bytes.clone()),
            });
        }

        let bytes = if self.compression != RequestCompression::None {
            if headers.contains_key(http::header::CONTENT_ENCODING) {
                return Err(
                    "request compression was requested but content-encoding is already set"
                        .to_string(),
                );
            }

            let pre_compression_bytes = body.bytes.len();
            let compression_start = std::time::Instant::now();
            let (compressed, content_encoding) = match self.compression {
                RequestCompression::None => unreachable!("guarded by compression != None"),
                RequestCompression::Zstd => (
                    zstd::stream::encode_all(std::io::Cursor::new(body.as_bytes()), 3)
                        .map_err(|err| err.to_string())?,
                    HeaderValue::from_static("zstd"),
                ),
            };
            let post_compression_bytes = compressed.len();
            let compression_duration = compression_start.elapsed();

            headers.insert(http::header::CONTENT_ENCODING, content_encoding);

            tracing::debug!(
                pre_compression_bytes,
                post_compression_bytes,
                compression_duration_ms = compression_duration.as_millis(),
                "Compressed request body with zstd"
            );

            Bytes::from(compressed)
        } else {
            body.bytes.clone()
        };

        if !headers.contains_key(http::header::CONTENT_TYPE) {
            headers.insert(
                http::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }

        Ok(PreparedRequestBody {
            headers,
            body: Some(bytes),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn multipart(parts: &str) -> Request {
        let mut request =
            Request::new(Method::POST, "https://example.com/v1/realtime/calls".into())
                .with_raw_body(parts.to_string());
        request.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("multipart/form-data; boundary=\"edge\""),
        );
        request
    }

    #[test]
    fn multipart_body_exposes_its_json_parts_for_inspection() {
        let request = multipart(concat!(
            "--edge\r\nContent-Disposition: form-data; name=\"sdp\"\r\n",
            "Content-Type: application/sdp\r\n\r\nv=offer\r\n",
            "--edge\r\nContent-Disposition: form-data; name=\"session\"\r\n",
            "Content-Type: application/json\r\n\r\n{\"model\":\"gpt-realtime\"}\r\n",
            "--edge--\r\n"
        ));

        assert_eq!(
            request.inspectable_json_documents(),
            Some(vec![json!({"model": "gpt-realtime"})])
        );
    }

    #[test]
    fn multipart_part_that_is_not_the_json_it_claims_is_uninspectable() {
        let request = multipart(concat!(
            "--edge\r\nContent-Disposition: form-data; name=\"session\"\r\n",
            "Content-Type: application/json\r\n\r\nnot json\r\n",
            "--edge--\r\n"
        ));

        assert_eq!(request.inspectable_json_documents(), None);
    }

    #[test]
    fn a_body_that_is_neither_json_nor_multipart_stays_uninspectable() {
        let request = Request::new(Method::POST, "https://example.com/v1/responses".to_string())
            .with_raw_body("v=offer\r\n");

        assert_eq!(request.inspectable_json_documents(), None);
    }

    #[test]
    fn prepare_body_for_send_serializes_json_and_sets_content_type() {
        let request = Request::new(Method::POST, "https://example.com/v1/responses".to_string())
            .with_json(&json!({"model": "test-model"}));

        let prepared = request
            .prepare_body_for_send()
            .expect("body should prepare");

        assert_eq!(
            prepared.body,
            Some(Bytes::from_static(br#"{"model":"test-model"}"#))
        );
        assert_eq!(
            prepared
                .headers
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
        assert_eq!(
            request.body,
            Some(RequestBody::Json(json!({"model": "test-model"})))
        );
        assert_eq!(request.compression, RequestCompression::None);
    }

    #[test]
    fn prepare_body_for_send_rejects_existing_content_encoding_when_compressing() {
        let mut request =
            Request::new(Method::POST, "https://example.com/v1/responses".to_string())
                .with_json(&json!({"model": "test-model"}))
                .with_compression(RequestCompression::Zstd);
        request.headers.insert(
            http::header::CONTENT_ENCODING,
            HeaderValue::from_static("gzip"),
        );

        let err = request
            .prepare_body_for_send()
            .expect_err("conflicting content-encoding should fail");

        assert_eq!(
            err,
            "request compression was requested but content-encoding is already set"
        );
    }

    #[test]
    fn into_prepared_stores_compressed_body_for_reuse() {
        let body =
            EncodedJsonBody::encode(&json!({"model": "test-model"})).expect("JSON should encode");
        let mut request =
            Request::new(Method::POST, "https://example.com/v1/responses".to_string())
                .with_compression(RequestCompression::Zstd);
        request.body = Some(RequestBody::EncodedJson(body));
        let request = request.into_prepared().expect("body should prepare");
        let Some(RequestBody::EncodedJson(body)) = request.body.as_ref() else {
            panic!("expected an encoded JSON body");
        };
        let decompressed = zstd::stream::decode_all(std::io::Cursor::new(body.as_bytes()))
            .expect("body should decompress");

        assert_eq!(decompressed, br#"{"model":"test-model"}"#);
        assert_eq!(request.compression, RequestCompression::None);
        assert_eq!(
            request.headers.get(http::header::CONTENT_ENCODING),
            Some(&HeaderValue::from_static("zstd"))
        );
        assert_eq!(
            request.headers.get(http::header::CONTENT_TYPE),
            Some(&HeaderValue::from_static("application/json"))
        );
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: http::StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}
