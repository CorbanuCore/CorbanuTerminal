use super::*;
use crate::GpuCredential;
use crate::GpuCredentialError;
use crate::HardwareRequirements;
use crate::SecretValue;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::body_partial_json;
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

async fn mount_funded_account(server: &MockServer, expected_requests: u64) {
    Mock::given(method("GET"))
        .and(path("/v0/users/current/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "balance": 25.0,
            "credit": 0.0
        })))
        .expect(expected_requests)
        .mount(server)
        .await;
}

fn create_request() -> CreateInstanceRequest {
    let request = requirements();
    let raw = offers_response(/*price*/ 3.25)["offers"][0].clone();
    CreateInstanceRequest {
        offer: vast_offer(&raw, &request, unix_now_ms()).expect("valid test offer"),
        client_operation_id: "operation-1".to_string(),
        ownership_tag: "pft-install-rental-1".to_string(),
        image: "runtime@sha256:abc".to_string(),
        disk_gib: 400,
        launch_command: vec!["model-id".to_string(), "argument with space".to_string()],
        inference_port: 8000,
        endpoint_token: SecretValue::new("endpoint-secret".to_string()).unwrap(),
        huggingface_token: Some(SecretValue::new("hf-secret".to_string()).unwrap()),
    }
}

#[tokio::test]
async fn verified_offer_quotes_recipe_disk_in_the_full_hourly_price() {
    let server = MockServer::start().await;
    mount_funded_account(&server, /*expected_requests*/ 1).await;
    Mock::given(method("POST"))
        .and(path("/v0/bundles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(offers_response(/*price*/ 3.25)))
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
    mount_funded_account(&server, /*expected_requests*/ 2).await;
    Mock::given(method("POST"))
        .and(path("/v0/bundles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(offers_response(/*price*/ 3.25)))
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
    assert_eq!(body["runtype"], "ssh_direct");
    assert_eq!(
        body["onstart"],
        "nohup 'model-id' 'argument with space' >/tmp/pfterminal-runtime.log 2>&1 &"
    );
    assert!(body.get("args").is_none());
    assert_eq!(body["env"]["PFT_ENDPOINT_TOKEN"], "endpoint-secret");
    assert_eq!(body["env"]["HF_TOKEN"], "hf-secret");
    assert!(body["env"].get("-p 8000:8000").is_none());
    assert!(!body.to_string().contains("cloudflare"));
    assert!(!body.to_string().contains(TEST_KEY));
}

#[tokio::test]
async fn rejected_create_with_disappeared_offer_is_reported_as_capacity_race() {
    let server = MockServer::start().await;
    mount_funded_account(&server, /*expected_requests*/ 2).await;
    Mock::given(method("PUT"))
        .and(path("/v0/asks/987/"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "success": false,
            "msg": "offer unavailable"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v0/bundles/"))
        .and(body_partial_json(serde_json::json!({"id": {"eq": 987}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "offers": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let error = provider(&server)
        .create_instance(create_request())
        .await
        .expect_err("vanished exact offer must not look like a malformed request");

    assert_eq!(error.kind, ProviderErrorKind::OfferUnavailable);
    assert!(error.safe_message.contains("claimed before allocation"));
    assert!(error.safe_message.contains("/gpu"));
}

#[tokio::test]
async fn rejected_create_with_still_rentable_offer_preserves_provider_rejection() {
    let server = MockServer::start().await;
    mount_funded_account(&server, /*expected_requests*/ 2).await;
    Mock::given(method("PUT"))
        .and(path("/v0/asks/987/"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "success": false,
            "msg": "invalid image configuration"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v0/bundles/"))
        .and(body_partial_json(serde_json::json!({"id": {"eq": 987}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(offers_response(/*price*/ 3.25)))
        .expect(1)
        .mount(&server)
        .await;

    let error = provider(&server)
        .create_instance(create_request())
        .await
        .expect_err("non-capacity rejection must retain its original classification");

    assert_eq!(error.kind, ProviderErrorKind::InvalidRequest);
    assert_eq!(error.safe_message, "GPU provider rejected the request.");
}

#[tokio::test]
async fn unfunded_account_is_rejected_before_offer_inventory_or_create() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/users/current/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "balance": -0.80,
            "credit": 0.0
        })))
        .expect(1)
        .mount(&server)
        .await;

    let error = provider(&server)
        .search_offers(requirements())
        .await
        .expect_err("unfunded account must not advertise unusable inventory");

    assert_eq!(error.kind, ProviderErrorKind::InsufficientFunds);
    assert!(error.safe_message.contains("Add funds"));
    assert!(error.safe_message.contains("no rental was created"));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
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
        /*now_ms*/ 1,
    )
    .unwrap();

    assert_eq!(offer.gpu_model, "H200 SXM");
    assert_eq!(offer.vram_mib_per_gpu, 143771);
    assert_eq!(offer.host_ram_mib, 774007);
    assert_eq!(offer.disk_gib, 5292);
}

#[test]
fn instance_uses_vasts_authoritative_ssh_host_and_port_pair() {
    let instance = vast_instance(&serde_json::json!({
        "id": 123,
        "actual_status": "running",
        "ssh_host": "ssh.example.vast.ai",
        "public_ipaddr": "192.0.2.10",
        "ssh_port": 22022
    }))
    .expect("valid instance");

    assert_eq!(instance.public_ip.as_deref(), Some("ssh.example.vast.ai"));
    assert_eq!(instance.ssh_port, Some(22022));
}

#[test]
fn onstart_preserves_arguments_without_a_shell_injection_or_public_tunnel() {
    let command = vast_onstart_command(&[
        "python3".to_string(),
        "argument with spaces".to_string(),
        "a'b; touch /tmp/not-run".to_string(),
    ]);
    assert_eq!(
        command,
        "nohup 'python3' 'argument with spaces' 'a'\\''b; touch /tmp/not-run' >/tmp/pfterminal-runtime.log 2>&1 &"
    );
    assert!(!command.contains("cloudflare"));
    assert!(!command.contains("trycloudflare"));
}

#[tokio::test]
async fn instance_ssh_key_is_attached_once_by_key_identity() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/instances/12345/ssh/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ssh_keys": [{"public_key": "ssh-ed25519 AAAAALREADY another-comment"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    provider(&server)
        .ensure_instance_ssh_key("12345", "ssh-ed25519 AAAAALREADY pfterminal")
        .await
        .expect("existing identity is accepted");
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn missing_instance_ssh_key_is_attached_without_exposing_credentials() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/instances/12345/ssh/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ssh_keys": "[]"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v0/instances/12345/ssh/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    provider(&server)
        .ensure_instance_ssh_key("12345", "ssh-ed25519 AAAANEW pfterminal")
        .await
        .expect("missing identity is attached");
    let requests = server.received_requests().await.unwrap();
    let body = requests
        .iter()
        .find(|request| request.method.as_str() == "POST")
        .unwrap()
        .body_json::<Value>()
        .unwrap();
    assert_eq!(body["ssh_key"], "ssh-ed25519 AAAANEW pfterminal");
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
    mount_funded_account(&server, /*expected_requests*/ 1).await;
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
