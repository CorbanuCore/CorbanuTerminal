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
            "dph_base": price,
            "storage_cost": 0.18,
            "geolocation": "US",
            "bw_nvlink": 900
        }]
    })
}

fn provider(server: &MockServer) -> VastProvider {
    VastProvider::with_api_base(Arc::new(TestCredentials), server.uri())
}

#[tokio::test]
async fn verified_offer_quotes_recipe_disk_in_the_full_hourly_price() {
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
    // $3.25/hr base + ($0.18/GiB-month * 400 GiB / 720 hours) = $3.35/hr.
    assert_eq!(offers[0].hourly_microusd, 3_350_000);
    assert_eq!(
        offers[0].expires_at_ms,
        Some(offers[0].quoted_at_ms + LOCAL_QUOTE_CONFIRMATION_WINDOW_MS)
    );
    assert_eq!(offers[0].security_class, "verified");
    assert!(!offers[0].raw_snapshot.to_string().contains(TEST_KEY));
}

#[tokio::test]
async fn create_is_bound_to_atomic_exact_ask_and_ownership_label() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v0/bundles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(offers_response(3.25)))
        .expect(1)
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
            launch_command: vec!["model-id".to_string(), "argument with space".to_string()],
            inference_port: 8000,
            endpoint_token: SecretValue::new("endpoint-secret".to_string()).unwrap(),
            huggingface_token: Some(SecretValue::new("hf-secret".to_string()).unwrap()),
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
    assert_eq!(body["client_id"], "me");
    assert_eq!(body["cancel_unavail"], true);
    assert_eq!(body["runtype"], "args");
    assert_eq!(body["args"][0], "bash");
    assert_eq!(body["args"][4], "8000");
    assert_eq!(body["args"][5], "model-id");
    assert_eq!(body["env"]["PFT_ENDPOINT_TOKEN"], "endpoint-secret");
    assert_eq!(body["env"]["HF_TOKEN"], "hf-secret");
    assert_eq!(body["env"]["-p 8000:8000"], "1");
    assert!(!body.to_string().contains(TEST_KEY));
}

#[test]
fn provider_numeric_and_name_normalization_preserve_recipe_contract() {
    let offer = vast_offer(
        &serde_json::json!({
            "id": 987,
            "gpu_name": "H200",
            "num_gpus": 2,
            "gpu_ram": 143771.5,
            "cpu_ram": 774007.25,
            "disk_space": 5292.75,
            "verification": "verified",
            "dph_total": 7.5,
            "dph_base": 7.25,
            "storage_cost": 0.45,
            "bw_nvlink": 478.1,
            "cuda_max_good": 13.2
        }),
        &requirements(),
        1,
    )
    .unwrap();

    assert_eq!(offer.gpu_model, "H200 SXM");
    assert_eq!(offer.vram_mib_per_gpu, 143771);
    assert_eq!(offer.host_ram_mib, 774007);
    assert_eq!(offer.disk_gib, 5292);
}

#[test]
fn secure_tunnel_output_accepts_only_the_cloudflare_https_shape() {
    assert_eq!(
        extract_trycloudflare_url("ok https://alpha-bravo-42.trycloudflare.com\n"),
        Some("https://alpha-bravo-42.trycloudflare.com")
    );
    assert_eq!(
        extract_trycloudflare_url("http://alpha.trycloudflare.com"),
        None
    );
    assert_eq!(extract_trycloudflare_url("https://example.com"), None);
    assert_eq!(
        extract_trycloudflare_url(
            "PFTERMINAL_TUNNEL_URL=https://old-tunnel.trycloudflare.com\n\
             PFTERMINAL_TUNNEL_URL=https://current-tunnel.trycloudflare.com\n"
        ),
        Some("https://current-tunnel.trycloudflare.com")
    );
}

#[test]
fn command_results_accept_official_regional_s3_hosts_only() {
    assert!(validate_vast_result_url("https://s3.us-west-2.amazonaws.com/result").is_ok());
    assert!(
        validate_vast_result_url("https://vast-results.s3.eu-west-1.amazonaws.com/result").is_ok()
    );
    assert!(validate_vast_result_url("https://s3.amazonaws.com.evil.example/result").is_err());
}

#[test]
fn signed_vast_log_download_urls_are_accepted() {
    assert!(
        validate_vast_result_url(
            "https://s3.amazonaws.com/public.vast.ai/instance_logs/log?X-Amz-Signature=abc"
        )
        .is_ok()
    );
}

#[tokio::test]
async fn secure_endpoint_discovery_retries_while_filtered_logs_start() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v0/instances/request_logs/12345"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "success": false,
            "msg": "instance log channel is starting"
        })))
        .mount(&server)
        .await;

    let error = provider(&server)
        .discover_tunnel_url("12345")
        .await
        .expect_err("startup rejection must be retried");
    assert_eq!(error.kind, ProviderErrorKind::Retryable);
    assert!(!format!("{error:?}").contains("instance log channel"));
    let requests = server.received_requests().await.unwrap();
    let body = requests[0].body_json::<Value>().unwrap();
    assert_eq!(body["filter"], "PFTERMINAL_TUNNEL_URL=");
    assert_eq!(body["tail"], "1000");
    assert!(body.get("daemon_logs").is_none());
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
async fn null_get_by_id_reconciles_through_complete_inventory() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/instances/12345/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"instances": null})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "instances": [{
                "id": 12345,
                "label": "pft-owned",
                "actual_status": "loading",
                "gpu_name": "H200",
                "num_gpus": 2,
                "cpu_ram": 256000.5,
                "disk_space": 400.0,
                "dph_total": 7.5
            }],
            "next_token": null
        })))
        .mount(&server)
        .await;

    let instance = provider(&server)
        .get_instance("12345".to_string())
        .await
        .unwrap()
        .expect("inventory recovers the transient null");

    assert_eq!(instance.resource_id, "12345");
    assert_eq!(instance.state, GpuInstanceState::Allocating);
    assert_eq!(instance.gpu_model, "NVIDIA H200");
}

#[tokio::test]
async fn null_get_by_id_and_empty_inventory_remain_ambiguous() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/instances/12345/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"instances": null})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "instances": [],
            "next_token": null
        })))
        .mount(&server)
        .await;

    let error = provider(&server)
        .get_instance("12345".to_string())
        .await
        .expect_err("eventually consistent empty views are not confirmed absence");
    assert_eq!(error.kind, ProviderErrorKind::Ambiguous);
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
