use super::*;
use crate::GpuCredential;
use crate::GpuCredentialError;
use crate::HardwareRequirements;
use crate::SecretValue;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

const TEST_KEY: &str = "vast-test-secret";

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
            gpu_model: "H200 SXM".to_string(),
            gpu_count: 2,
            minimum_vram_mib_per_gpu: 130_000,
            minimum_host_ram_mib: 128_000,
            minimum_disk_gib: 400,
            requires_high_bandwidth_interconnect: true,
            allowed_cuda_versions: Vec::new(),
        },
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 4_000_000,
    }
}

fn offers_response(price: f64) -> Value {
    serde_json::json!({
        "offers": [{
            "id": 987,
            "gpu_name": "H200 SXM",
            "num_gpus": 2,
            "gpu_ram": 141000,
            "cpu_ram": 256000,
            "disk_space": 800,
            "verification": "verified",
            "reliability2": 0.995,
            "is_bid": false,
            "dph_total": price,
            "storage_cost": 0.15,
            "geolocation": "US",
            "bw_nvlink": 900
        }]
    })
}

fn provider(server: &MockServer) -> VastProvider {
    VastProvider::with_api_base(Arc::new(TestCredentials), server.uri())
}

#[tokio::test]
async fn verified_offer_preserves_total_price_and_raw_snapshot() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v0/bundles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(offers_response(3.25)))
        .mount(&server)
        .await;

    let offers = provider(&server)
        .search_offers(requirements())
        .await
        .expect("search Vast");

    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].offer_id, "987");
    assert_eq!(offers[0].hourly_microusd, 3_250_000);
    assert_eq!(offers[0].security_class, "verified");
    assert!(!offers[0].raw_snapshot.to_string().contains(TEST_KEY));
}

#[tokio::test]
async fn create_is_bound_to_revalidated_ask_and_ownership_label() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v0/bundles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(offers_response(3.25)))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v0/asks/987/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"success": true, "new_contract": 12345})),
        )
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
            ownership_tag: "pft-install-rental-1".to_string(),
            image: "runtime@sha256:abc".to_string(),
            disk_gib: 400,
        })
        .await
        .expect("create Vast instance");

    assert_eq!(instance.resource_id, "12345");
    let requests = server.received_requests().await.unwrap();
    let body = requests
        .iter()
        .find(|request| request.url.path() == "/v0/asks/987/")
        .expect("create request")
        .body_json::<Value>()
        .expect("json body");
    assert_eq!(body["label"], "pft-install-rental-1");
    assert_eq!(body["cancel_unavail"], true);
    assert!(!body.to_string().contains(TEST_KEY));
}

#[tokio::test]
async fn owned_inventory_filters_unrelated_resources() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "instances": [
                {"id": 1, "label": "pft-owned", "actual_status": "running", "gpu_name": "H200 SXM", "num_gpus": 2, "dph_total": 3.0},
                {"id": 2, "label": "human-workload", "actual_status": "running", "gpu_name": "H200 SXM", "num_gpus": 2, "dph_total": 3.0}
            ],
            "next_token": null
        })))
        .mount(&server)
        .await;

    let instances = provider(&server)
        .list_owned_instances(OwnedInstanceQuery {
            installation_id: "install".to_string(),
            ownership_tag: Some("pft-owned".to_string()),
        })
        .await
        .expect("inventory");

    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].resource_id, "1");
}

#[tokio::test]
async fn malformed_response_is_sanitized() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v0/bundles/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(TEST_KEY))
        .mount(&server)
        .await;
    let error = provider(&server)
        .search_offers(requirements())
        .await
        .expect_err("malformed response");
    assert!(!format!("{error:?}").contains(TEST_KEY));
    assert!(error.diagnostic_ref.is_some());
}
