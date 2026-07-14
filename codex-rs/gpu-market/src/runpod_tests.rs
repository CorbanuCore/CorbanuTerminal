use super::*;
use crate::GpuCredential;
use crate::GpuCredentialError;
use crate::HardwareRequirements;
use crate::SecretValue;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::header;
use wiremock::matchers::method;
use wiremock::matchers::path;

const TEST_KEY: &str = "runpod-test-secret";

#[derive(Debug)]
struct TestCredentials;

impl GpuCredentialResolver for TestCredentials {
    fn resolve(&self, kind: &GpuCredentialKind) -> Result<GpuCredential, GpuCredentialError> {
        Ok(GpuCredential {
            label: kind.canonical_label()?,
            secret: SecretValue::new(TEST_KEY.to_string())?,
        })
    }
}

fn requirements() -> SearchOffersRequest {
    SearchOffersRequest {
        hardware: HardwareRequirements {
            gpu_model: "NVIDIA H200 NVL".to_string(),
            gpu_count: 2,
            minimum_vram_mib_per_gpu: 130_000,
            minimum_host_ram_mib: 128_000,
            minimum_disk_gib: 400,
            requires_high_bandwidth_interconnect: false,
            allowed_cuda_versions: vec!["12.8".to_string()],
        },
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 4_000_000,
    }
}

fn offer_response(price: f64) -> Value {
    serde_json::json!({
        "data": {
            "gpuTypes": [{
                "id": "NVIDIA H200 NVL",
                "displayName": "H200 NVL",
                "memoryInGb": 141,
                "secureCloud": true,
                "lowestPrice": {
                    "stockStatus": "High",
                    "uninterruptablePrice": price,
                    "availableGpuCounts": [1, 2, 4]
                }
            }]
        }
    })
}

fn provider(server: &MockServer) -> RunpodProvider {
    RunpodProvider::with_endpoints(
        Arc::new(TestCredentials),
        format!("{}/v1", server.uri()),
        format!("{}/graphql", server.uri()),
    )
}

#[tokio::test]
async fn secure_offer_is_normalized_and_secret_stays_in_auth_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", format!("Bearer {TEST_KEY}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(offer_response(3.5)))
        .expect(1)
        .mount(&server)
        .await;

    let offers = provider(&server)
        .search_offers(requirements())
        .await
        .expect("search offers");

    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].hourly_microusd, 3_500_000);
    assert_eq!(offers[0].security_class, "secure");
    assert!(!offers[0].interruptible);
    assert!(!serde_json::to_string(&offers).unwrap().contains(TEST_KEY));
}

#[tokio::test]
async fn create_revalidates_price_and_uses_owned_secure_pod() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(offer_response(3.5)))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/pods"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "pod-1",
            "name": "pft-install-lease-1",
            "desiredStatus": "RUNNING",
            "gpu": {"id": "NVIDIA H200 NVL", "count": 2},
            "costPerHr": "3.5"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let provider = provider(&server);
    let offer = provider
        .search_offers(requirements())
        .await
        .unwrap()
        .remove(0);

    let instance = provider
        .create_instance(CreateInstanceRequest {
            offer,
            client_operation_id: "operation-1".to_string(),
            ownership_tag: "pft-install-lease-1".to_string(),
            image: "example.invalid/runtime@sha256:abc".to_string(),
            disk_gib: 400,
        })
        .await
        .expect("create pod");

    assert_eq!(instance.resource_id, "pod-1");
    assert_eq!(instance.ownership_tag, "pft-install-lease-1");
    let requests = server.received_requests().await.expect("requests");
    let create_body = requests
        .iter()
        .find(|request| request.url.path() == "/v1/pods")
        .expect("create request")
        .body_json::<Value>()
        .expect("json body");
    assert_eq!(create_body["cloudType"], "SECURE");
    assert_eq!(create_body["interruptible"], false);
    assert_eq!(create_body["name"], "pft-install-lease-1");
    assert!(!create_body.to_string().contains(TEST_KEY));
}

#[tokio::test]
async fn changed_price_fails_closed_before_create() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(offer_response(3.5)))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    let provider = provider(&server);
    let offer = provider
        .search_offers(requirements())
        .await
        .unwrap()
        .remove(0);
    server.reset().await;
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(offer_response(3.75)))
        .mount(&server)
        .await;

    let error = provider
        .create_instance(CreateInstanceRequest {
            offer,
            client_operation_id: "operation-1".to_string(),
            ownership_tag: "pft-install-lease-1".to_string(),
            image: "runtime@sha256:abc".to_string(),
            disk_gib: 400,
        })
        .await
        .expect_err("price drift must reject");

    assert_eq!(error.kind, ProviderErrorKind::PriceDrift);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn provider_error_never_echoes_response_secret() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(500).set_body_string(TEST_KEY))
        .mount(&server)
        .await;
    let error = provider(&server)
        .search_offers(requirements())
        .await
        .expect_err("provider failure");
    assert!(!format!("{error:?}").contains(TEST_KEY));
    assert!(error.diagnostic_ref.is_some());
}
