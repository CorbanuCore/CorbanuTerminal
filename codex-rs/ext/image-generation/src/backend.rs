use codex_api::ImageEditRequest;
use codex_api::ImageGenerationRequest;
use codex_api::ImageResponse;
use codex_api::ImagesClient;
use codex_api::ReqwestTransport;
use codex_core::accounting_extensions::Accounted;
use codex_core::accounting_extensions::ExtensionAccounting;
use codex_login::default_client::add_originator_header;
use codex_login::default_client::create_client;
use codex_model_provider::SharedModelProvider;
use http::HeaderMap;
use http::HeaderValue;

const X_CODEX_IMAGE_TURN_ID_HEADER: &str = "x-codex-image-turn-id";

#[derive(Clone)]
pub(crate) struct CodexImagesBackend {
    provider: SharedModelProvider,
    originator: Option<String>,
    /// Image requests are billed model inference. The host hands this over so
    /// they reach the ledger like any other request; without it - an older host
    /// or a session that has gone away - they are sent unrecorded rather than
    /// refused.
    accounting: Option<std::sync::Arc<ExtensionAccounting>>,
}

impl CodexImagesBackend {
    /// Creates a backend that sends image requests through the active model provider.
    pub(crate) fn new(
        provider: SharedModelProvider,
        originator: Option<String>,
        accounting: Option<std::sync::Arc<ExtensionAccounting>>,
    ) -> Self {
        Self {
            provider,
            originator,
            accounting,
        }
    }

    /// Resolves the provider, auth and collection for the current image request.
    async fn client(
        &self,
        model: &str,
        path: &str,
    ) -> Result<ImagesClient<Accounted<ReqwestTransport>>, String> {
        let provider = self
            .provider
            .api_provider()
            .await
            .map_err(|err| err.to_string())?;
        let auth = self
            .provider
            .api_auth()
            .await
            .map_err(|err| err.to_string())?;
        let transport = ReqwestTransport::from_http_client(create_client());
        let transport = match &self.accounting {
            Some(accounting) => accounting.transport(transport, model, path, "image").await,
            None => ExtensionAccounting::unrecorded(transport, model),
        };
        Ok(ImagesClient::new(transport, provider, auth))
    }

    /// Sends a standalone image generation request through the configured Images client.
    pub(crate) async fn generate(
        &self,
        request: ImageGenerationRequest,
        turn_id: &str,
    ) -> Result<ImageResponse, String> {
        self.client(&request.model, "images/generations")
            .await?
            .generate(
                &request,
                image_request_headers(self.originator.as_deref(), turn_id),
            )
            .await
            .map_err(|err| err.to_string())
    }

    /// Sends a standalone image edit request through the configured Images client.
    pub(crate) async fn edit(
        &self,
        request: ImageEditRequest,
        turn_id: &str,
    ) -> Result<ImageResponse, String> {
        self.client(&request.model, "images/edits")
            .await?
            .edit(
                &request,
                image_request_headers(self.originator.as_deref(), turn_id),
            )
            .await
            .map_err(|err| err.to_string())
    }
}

fn image_request_headers(originator: Option<&str>, turn_id: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(turn_id) = HeaderValue::from_str(turn_id) {
        headers.insert(X_CODEX_IMAGE_TURN_ID_HEADER, turn_id);
    }
    if let Some(originator) = originator {
        add_originator_header(&mut headers, originator);
    }
    headers
}
