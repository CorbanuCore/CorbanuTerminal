use codex_api::ImageEditRequest;
use codex_api::ImageGenerationRequest;
use codex_api::ImageRequestError;
use codex_api::ImageResponse;
use codex_api::ImagesClient;
use codex_api::ReqwestTransport;
use codex_api::map_api_error;
use codex_core::X_CODEX_TURN_METADATA_HEADER;
use codex_core::accounting_extensions::Accounted;
use codex_core::accounting_extensions::ExtensionAccounting;
use codex_http_client::ClientRouteClass;
use codex_http_client::HttpClientFactory;
use codex_login::default_client::add_originator_header;
use codex_login::default_client::create_transport_for_routes_async;
use codex_model_provider::SharedModelProvider;
use codex_protocol::error::CodexErr;
use http::HeaderMap;
use http::HeaderValue;

const X_CODEX_IMAGE_TURN_ID_HEADER: &str = "x-codex-image-turn-id";

pub(crate) struct ImageBackendError {
    message: String,
    codex_error: CodexErr,
    imagegen_request_id: Option<String>,
}

impl ImageBackendError {
    fn from_image_request(error: ImageRequestError) -> Self {
        let (error, imagegen_request_id) = error.into_parts();
        let message = error.to_string();
        Self {
            message,
            codex_error: map_api_error(error),
            imagegen_request_id,
        }
    }

    fn from_message(message: String) -> Self {
        Self {
            codex_error: CodexErr::Stream(message.clone()),
            message,
            imagegen_request_id: None,
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn codex_error(&self) -> &CodexErr {
        &self.codex_error
    }

    pub(crate) fn imagegen_request_id(&self) -> Option<&str> {
        self.imagegen_request_id.as_deref()
    }
}

#[derive(Clone)]
pub(crate) struct CodexImagesBackend {
    provider: SharedModelProvider,
    http_client_factory: HttpClientFactory,
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
        http_client_factory: HttpClientFactory,
        originator: Option<String>,
        accounting: Option<std::sync::Arc<ExtensionAccounting>>,
    ) -> Self {
        Self {
            provider,
            http_client_factory,
            originator,
            accounting,
        }
    }

    /// Resolves the provider, auth and collection for the current image request.
    async fn client(
        &self,
        model: &str,
        path: &str,
    ) -> Result<ImagesClient<Accounted<ReqwestTransport>>, ImageBackendError> {
        let (provider, auth) = self
            .provider
            .api_provider_and_auth()
            .await
            .map_err(|err| ImageBackendError::from_message(err.to_string()))?;
        let transport = create_transport_for_routes_async(
            self.http_client_factory.clone(),
            ClientRouteClass::Api,
        )
        .await
        .map_err(|err| ImageBackendError::from_message(err.to_string()))?;
        let transport = match &self.accounting {
            Some(accounting) => {
                accounting
                    .transport(
                        transport,
                        self.provider.info(),
                        &provider.base_url,
                        model,
                        path,
                        "image",
                    )
                    .await
            }
            None => ExtensionAccounting::unrecorded(transport, model),
        };
        Ok(ImagesClient::new(transport, provider, auth))
    }

    /// Sends a standalone image generation request through the configured Images client.
    pub(crate) async fn generate(
        &self,
        request: ImageGenerationRequest,
        turn_id: &str,
        codex_turn_metadata: Option<&str>,
    ) -> Result<(ImageResponse, Option<String>), ImageBackendError> {
        self.client(&request.model, "images/generations")
            .await?
            .generate(
                &request,
                image_request_headers(self.originator.as_deref(), turn_id, codex_turn_metadata),
            )
            .await
            .map_err(ImageBackendError::from_image_request)
    }

    /// Sends a standalone image edit request through the configured Images client.
    pub(crate) async fn edit(
        &self,
        request: ImageEditRequest,
        turn_id: &str,
        codex_turn_metadata: Option<&str>,
    ) -> Result<(ImageResponse, Option<String>), ImageBackendError> {
        self.client(&request.model, "images/edits")
            .await?
            .edit(
                &request,
                image_request_headers(self.originator.as_deref(), turn_id, codex_turn_metadata),
            )
            .await
            .map_err(ImageBackendError::from_image_request)
    }
}

fn image_request_headers(
    originator: Option<&str>,
    turn_id: &str,
    codex_turn_metadata: Option<&str>,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(turn_id) = HeaderValue::from_str(turn_id) {
        headers.insert(X_CODEX_IMAGE_TURN_ID_HEADER, turn_id);
    }
    if let Some(originator) = originator {
        add_originator_header(&mut headers, originator);
    }

    if let Some(metadata) = codex_turn_metadata
        && let Ok(metadata) = HeaderValue::from_str(metadata)
    {
        headers.insert(X_CODEX_TURN_METADATA_HEADER, metadata);
    }

    headers
}
