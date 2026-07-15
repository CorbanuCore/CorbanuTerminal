use crate::BillingState;
use crate::CreateInstanceRequest;
use crate::GpuCredentialKind;
use crate::GpuCredentialResolver;
use crate::GpuInstance;
use crate::GpuInstanceState;
use crate::GpuOffer;
use crate::GpuProvider;
use crate::OwnedInstanceQuery;
use crate::ProviderCapabilities;
use crate::ProviderError;
use crate::ProviderErrorKind;
use crate::ProviderResult;
use crate::SearchOffersRequest;
use crate::provider_http::credential_error;
use crate::provider_http::decode_json;
use crate::provider_http::parse_usd_micros;
use crate::provider_http::transport_error;
use serde_json::Value;
use std::sync::Arc;

const DEFAULT_API_BASE: &str = "https://console.vast.ai/api";
const CLOUDFLARED_URL: &str =
    "https://github.com/cloudflare/cloudflared/releases/download/2026.7.1/cloudflared-linux-amd64";
const CLOUDFLARED_SHA256: &str = "79a0ade7fc854f62c1aaef48424d9d979e8c2fcd039189d24db82b84cd146be1";
const TUNNEL_URL_PATH: &str = "/tmp/pfterminal-cloudflared-url";
const TUNNEL_LOG_PATH: &str = "/tmp/pfterminal-cloudflared.log";
const LOCAL_QUOTE_CONFIRMATION_WINDOW_MS: i64 = 5 * 60 * 1000;

#[derive(Clone)]
pub struct VastProvider {
    client: reqwest::Client,
    credentials: Arc<dyn GpuCredentialResolver>,
    api_base: String,
}

impl std::fmt::Debug for VastProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VastProvider")
            .field("api_base", &self.api_base)
            .finish_non_exhaustive()
    }
}

impl VastProvider {
    pub fn new(credentials: Arc<dyn GpuCredentialResolver>) -> Self {
        Self {
            client: reqwest::Client::new(),
            credentials,
            api_base: DEFAULT_API_BASE.to_string(),
        }
    }

    pub fn with_api_base(
        credentials: Arc<dyn GpuCredentialResolver>,
        api_base: impl Into<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            credentials,
            api_base: api_base.into().trim_end_matches('/').to_string(),
        }
    }

    fn api_key(&self) -> ProviderResult<crate::GpuCredential> {
        self.credentials
            .resolve(&GpuCredentialKind::ProviderApiKey {
                provider: "vast".to_string(),
            })
            .map_err(credential_error)
    }

    async fn offers(&self, request: &SearchOffersRequest) -> ProviderResult<Vec<GpuOffer>> {
        let key = self.api_key()?;
        let mut filters = serde_json::json!({
            "verified": {"eq": true},
            "rentable": {"eq": true},
            "gpu_name": {"eq": vast_gpu_name(request.hardware.gpu_model.as_str())},
            "num_gpus": {"eq": request.hardware.gpu_count},
            "gpu_ram": {"gte": request.hardware.minimum_vram_mib_per_gpu},
            "disk_space": {"gte": request.hardware.minimum_disk_gib},
            "direct_port_count": {"gte": 1},
            "type": "on-demand",
            "order": [["dph_total", "asc"]],
            "limit": 25
        });
        if request.hardware.requires_high_bandwidth_interconnect {
            filters["bw_nvlink"] = serde_json::json!({"gt": 0});
        }
        let response = self
            .client
            .post(format!("{}/v0/bundles/", self.api_base))
            .bearer_auth(key.secret.expose())
            .json(&filters)
            .send()
            .await
            .map_err(|_| transport_error())?;
        let json = decode_json(response).await?;
        let raw_offers = json
            .get("offers")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ProviderError::new(
                    ProviderErrorKind::Permanent,
                    "Vast returned malformed offer inventory.",
                )
            })?;
        let now_ms = unix_now_ms();
        raw_offers
            .iter()
            .map(|raw| vast_offer(raw, request, now_ms))
            .filter_map(|result| match result {
                Ok(offer) if offer.validate_for(request, now_ms).is_ok() => Some(Ok(offer)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .collect()
    }

    async fn get_raw_instance(&self, resource_id: &str) -> ProviderResult<Option<Value>> {
        let key = self.api_key()?;
        let response = self
            .client
            .get(format!("{}/v0/instances/{resource_id}/", self.api_base))
            .bearer_auth(key.secret.expose())
            .send()
            .await
            .map_err(|_| transport_error())?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let json = decode_json(response).await?;
        let instance = json
            .get("instances")
            .and_then(|instances| {
                if instances.is_array() {
                    instances.as_array().and_then(|values| values.first())
                } else if instances.is_null() {
                    None
                } else {
                    Some(instances)
                }
            })
            .cloned();
        match instance {
            Some(instance) => Ok(Some(instance)),
            None => {
                if let Some(instance) = self.find_raw_instance_in_inventory(resource_id).await? {
                    return Ok(Some(instance));
                }
                Err(ProviderError::new(
                    ProviderErrorKind::Ambiguous,
                    "Vast temporarily omitted a known instance from both instance views.",
                ))
            }
        }
    }

    async fn find_raw_instance_in_inventory(
        &self,
        resource_id: &str,
    ) -> ProviderResult<Option<Value>> {
        let key = self.api_key()?;
        let mut after_token: Option<String> = None;
        for _ in 0..100 {
            let mut request = self
                .client
                .get(format!("{}/v1/instances/", self.api_base))
                .bearer_auth(key.secret.expose())
                .query(&[("limit", "25")]);
            if let Some(token) = after_token.as_deref() {
                request = request.query(&[("after_token", token)]);
            }
            let response = request.send().await.map_err(|_| transport_error())?;
            let json = decode_json(response).await?;
            let page = json
                .get("instances")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ProviderError::new(
                        ProviderErrorKind::Retryable,
                        "Vast returned incomplete instance inventory.",
                    )
                })?;
            if let Some(instance) = page
                .iter()
                .find(|raw| value_id(raw.get("id")).as_deref() == Some(resource_id))
            {
                return Ok(Some(instance.clone()));
            }
            after_token = json
                .get("next_token")
                .and_then(Value::as_str)
                .map(str::to_string);
            if after_token.is_none() {
                return Ok(None);
            }
        }
        Err(ProviderError::new(
            ProviderErrorKind::Permanent,
            "Vast instance inventory exceeded the reconciliation page bound.",
        ))
    }

    async fn discover_tunnel_url(&self, resource_id: &str) -> ProviderResult<String> {
        let key = self.api_key()?;
        let response = self
            .client
            .put(format!(
                "{}/v0/instances/request_logs/{resource_id}",
                self.api_base
            ))
            .bearer_auth(key.secret.expose())
            .json(&serde_json::json!({
                "tail": "1000",
                "filter": "PFTERMINAL_TUNNEL_URL="
            }))
            .send()
            .await
            .map_err(|_| transport_error())?;
        let json = match decode_json(response).await {
            Ok(json) => json,
            Err(error)
                if matches!(
                    error.kind,
                    ProviderErrorKind::InvalidRequest | ProviderErrorKind::OfferUnavailable
                ) =>
            {
                return Err(ProviderError {
                    kind: ProviderErrorKind::Retryable,
                    safe_message:
                        "Vast filtered container logs are not ready for the starting instance."
                            .to_string(),
                    retry_after_ms: error.retry_after_ms,
                    diagnostic_ref: error.diagnostic_ref,
                });
            }
            Err(error) => return Err(error),
        };
        // Vast returns an unsigned archival URL and, for private/new logs, a
        // short-lived signed download URL. Prefer the signed URL when present;
        // the archival URL can legitimately answer 403 during startup.
        let result_url = json
            .get("temp_download_url")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .or_else(|| json.get("result_url").and_then(Value::as_str))
            .ok_or_else(|| {
                ProviderError::new(
                    ProviderErrorKind::Retryable,
                    "Vast secure tunnel discovery is not ready yet.",
                )
            })?;
        validate_vast_result_url(result_url)?;

        for _ in 0..6 {
            let response = self
                .client
                .get(result_url)
                .send()
                .await
                .map_err(|_| transport_error())?;
            if response.status().is_success() {
                let output = response.text().await.map_err(|_| {
                    ProviderError::new(
                        ProviderErrorKind::Retryable,
                        "Vast secure tunnel discovery returned unreadable output.",
                    )
                })?;
                if let Some(url) = extract_trycloudflare_url(output.as_str()) {
                    return Ok(format!("{url}/v1"));
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        Err(ProviderError::new(
            ProviderErrorKind::Retryable,
            "Vast secure tunnel discovery is not ready yet.",
        ))
    }
}

impl GpuProvider for VastProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            provider: "vast".to_string(),
            supports_ownership_tags: true,
            supports_inventory: true,
            supports_secure_endpoint_transport: true,
            supports_native_ttl: false,
            supports_native_spend_cap: false,
            security_classes: vec!["verified".to_string()],
        }
    }

    fn create_revalidates_exact_offer_atomically(&self) -> bool {
        true
    }

    async fn secure_endpoint_base_url(
        &self,
        instance: &GpuInstance,
        inference_port: u16,
    ) -> ProviderResult<String> {
        if instance.resource_id.is_empty() || inference_port == 0 {
            return Err(ProviderError::new(
                ProviderErrorKind::Permanent,
                "Vast endpoint identity is incomplete.",
            ));
        }
        self.discover_tunnel_url(instance.resource_id.as_str())
            .await
    }

    async fn search_offers(&self, request: SearchOffersRequest) -> ProviderResult<Vec<GpuOffer>> {
        self.offers(&request).await
    }

    async fn create_instance(&self, request: CreateInstanceRequest) -> ProviderResult<GpuInstance> {
        let requirements = SearchOffersRequest {
            hardware: crate::HardwareRequirements {
                gpu_model: request.offer.gpu_model.clone(),
                gpu_count: request.offer.gpu_count,
                minimum_vram_mib_per_gpu: request.offer.vram_mib_per_gpu,
                minimum_host_ram_mib: request.offer.host_ram_mib,
                minimum_disk_gib: request.disk_gib,
                requires_high_bandwidth_interconnect: request.offer.high_bandwidth_interconnect
                    || request.offer.runtime_topology_verification,
                allowed_cuda_versions: request.offer.cuda_versions.clone(),
            },
            allow_interruptible: false,
            require_verified_or_secure: true,
            maximum_hourly_microusd: request.offer.hourly_microusd,
        };
        request.offer.validate_for(&requirements, unix_now_ms())?;

        let key = self.api_key()?;
        let mut environment = serde_json::Map::from_iter([
            (
                "PFT_ENDPOINT_TOKEN".to_string(),
                Value::String(request.endpoint_token.expose().to_string()),
            ),
            (
                format!("-p {0}:{0}", request.inference_port),
                Value::String("1".to_string()),
            ),
        ]);
        if let Some(token) = request.huggingface_token.as_ref() {
            environment.insert(
                "HF_TOKEN".to_string(),
                Value::String(token.expose().to_string()),
            );
        }
        let launch_command =
            vast_secure_launch_command(&request.launch_command, request.inference_port);
        let body = serde_json::json!({
            "client_id": "me",
            "image": request.image,
            "disk": request.disk_gib,
            "label": request.ownership_tag,
            "cancel_unavail": true,
            "runtype": "args",
            "args": launch_command,
            "env": environment,
        });
        let response = self
            .client
            .put(format!(
                "{}/v0/asks/{}/",
                self.api_base, request.offer.offer_id
            ))
            .bearer_auth(key.secret.expose())
            .json(&body)
            .send()
            .await
            .map_err(|_| transport_error())?;
        let json = decode_json(response).await?;
        let resource_id = json
            .get("new_contract")
            .and_then(|value| {
                value
                    .as_u64()
                    .map(|id| id.to_string())
                    .or_else(|| value.as_str().map(str::to_string))
            })
            .ok_or_else(|| {
                ProviderError::new(
                    ProviderErrorKind::Ambiguous,
                    "Vast create response did not identify the new instance.",
                )
            })?;
        Ok(GpuInstance {
            provider: "vast".to_string(),
            resource_id,
            ownership_tag: request.ownership_tag,
            state: GpuInstanceState::Allocating,
            gpu_model: request.offer.gpu_model,
            gpu_count: request.offer.gpu_count,
            host_ram_mib: Some(request.offer.host_ram_mib),
            disk_gib: Some(request.disk_gib),
            high_bandwidth_interconnect: Some(request.offer.high_bandwidth_interconnect),
            hourly_microusd: request.offer.hourly_microusd,
            created_at_ms: Some(unix_now_ms()),
            public_ip: None,
            ssh_port: None,
        })
    }

    async fn get_instance(&self, resource_id: String) -> ProviderResult<Option<GpuInstance>> {
        self.get_raw_instance(resource_id.as_str())
            .await?
            .map(|raw| vast_instance(&raw))
            .transpose()
    }

    async fn list_owned_instances(
        &self,
        query: OwnedInstanceQuery,
    ) -> ProviderResult<Vec<GpuInstance>> {
        let key = self.api_key()?;
        let mut after_token: Option<String> = None;
        let mut instances = Vec::new();
        for _ in 0..100 {
            let mut request = self
                .client
                .get(format!("{}/v1/instances/", self.api_base))
                .bearer_auth(key.secret.expose())
                .query(&[("limit", "25")]);
            if let Some(token) = after_token.as_deref() {
                request = request.query(&[("after_token", token)]);
            }
            let response = request.send().await.map_err(|_| transport_error())?;
            let json = decode_json(response).await?;
            let page = json
                .get("instances")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ProviderError::new(
                        ProviderErrorKind::Permanent,
                        "Vast returned malformed instance inventory.",
                    )
                })?;
            for raw in page {
                if query
                    .ownership_tag
                    .as_ref()
                    .is_none_or(|tag| raw.get("label").and_then(Value::as_str) == Some(tag))
                {
                    instances.push(vast_instance(raw)?);
                }
            }
            after_token = json
                .get("next_token")
                .and_then(Value::as_str)
                .map(str::to_string);
            if after_token.is_none() {
                return Ok(instances);
            }
        }
        Err(ProviderError::new(
            ProviderErrorKind::Permanent,
            "Vast instance inventory exceeded the reconciliation page bound.",
        ))
    }

    async fn terminate_instance(&self, resource_id: String) -> ProviderResult<()> {
        let key = self.api_key()?;
        let response = self
            .client
            .delete(format!("{}/v0/instances/{resource_id}/", self.api_base))
            .bearer_auth(key.secret.expose())
            .send()
            .await
            .map_err(|_| transport_error())?;
        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(())
        } else {
            decode_json(response).await.map(|_| ())
        }
    }

    async fn billing_state(&self, resource_id: String) -> ProviderResult<BillingState> {
        let Some(instance) = self.get_instance(resource_id.clone()).await? else {
            return Ok(BillingState {
                resource_id,
                estimated_accrued_microusd: 0,
                provider_reported_cost_microusd: None,
                still_billable: false,
            });
        };
        let elapsed_ms = instance
            .created_at_ms
            .map(|started| unix_now_ms().saturating_sub(started))
            .unwrap_or_default();
        Ok(BillingState {
            resource_id,
            estimated_accrued_microusd: instance.hourly_microusd.saturating_mul(elapsed_ms)
                / 3_600_000,
            provider_reported_cost_microusd: None,
            still_billable: instance.state != GpuInstanceState::Stopped,
        })
    }
}

fn vast_offer(raw: &Value, request: &SearchOffersRequest, now_ms: i64) -> ProviderResult<GpuOffer> {
    let id = value_id(raw.get("id")).ok_or_else(|| {
        ProviderError::new(
            ProviderErrorKind::Permanent,
            "Vast offer omitted its ask id.",
        )
    })?;
    // Vast's search `dph_total` includes only the ask's tiny default disk allocation. Creating
    // an instance with the recipe's explicit disk size changes the billable total. Quote and
    // enforce the same full amount Vast will bill: base GPU hourly price plus the requested
    // disk's monthly storage price amortized over Vast's 30-day billing month.
    let base_hourly_microusd = raw
        .get("dph_base")
        .and_then(parse_usd_micros)
        .ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::Permanent,
                "Vast offer omitted authoritative base hourly price.",
            )
        })?;
    let storage_microusd_per_gib_month = raw
        .get("storage_cost")
        .and_then(parse_usd_micros)
        .ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::Permanent,
                "Vast offer omitted authoritative storage price.",
            )
        })?;
    let storage_hourly_microusd = i128::from(storage_microusd_per_gib_month)
        .checked_mul(i128::from(request.hardware.minimum_disk_gib))
        .and_then(|value| value.checked_add(719))
        .map(|value| value / 720)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::Permanent,
                "Vast full hourly price exceeded the supported billing range.",
            )
        })?;
    let hourly_microusd = base_hourly_microusd
        .checked_add(storage_hourly_microusd)
        .ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::Permanent,
                "Vast full hourly price exceeded the supported billing range.",
            )
        })?;
    Ok(GpuOffer {
        provider: "vast".to_string(),
        offer_id: id,
        gpu_model: request.hardware.gpu_model.clone(),
        gpu_count: raw
            .get("num_gpus")
            .and_then(Value::as_u64)
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default(),
        vram_mib_per_gpu: value_u64_floor(raw.get("gpu_ram")).unwrap_or_default(),
        host_ram_mib: value_u64_floor(raw.get("cpu_ram")).unwrap_or_default(),
        disk_gib: value_u64_floor(raw.get("disk_space")).unwrap_or_default(),
        high_bandwidth_interconnect: raw
            .get("bw_nvlink")
            .and_then(Value::as_f64)
            .is_some_and(|bandwidth| bandwidth > 0.0),
        runtime_topology_verification: false,
        cuda_versions: vast_cuda_versions(raw, request),
        region: raw
            .get("geolocation")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        security_class: raw
            .get("verification")
            .and_then(Value::as_str)
            .unwrap_or("unverified")
            .to_string(),
        reliability_millionths: raw
            .get("reliability2")
            .and_then(Value::as_f64)
            .map(|value| (value.clamp(0.0, 1.0) * 1_000_000.0).round() as u32),
        interruptible: raw.get("is_bid").and_then(Value::as_bool).unwrap_or(false),
        hourly_microusd,
        storage_microusd_per_gib_month: Some(storage_microusd_per_gib_month),
        quoted_at_ms: now_ms,
        // Vast asks do not carry a provider-guaranteed quote TTL. This local window only
        // bounds stale UI confirmation; confirmation still re-fetches this exact ask and
        // rejects disappearance or price drift before any billable create request.
        expires_at_ms: Some(now_ms.saturating_add(LOCAL_QUOTE_CONFIRMATION_WINDOW_MS)),
        raw_snapshot: raw.clone(),
    })
}

fn vast_instance(raw: &Value) -> ProviderResult<GpuInstance> {
    let resource_id = value_id(raw.get("id")).ok_or_else(|| {
        ProviderError::new(
            ProviderErrorKind::Permanent,
            "Vast instance omitted its id.",
        )
    })?;
    let status = raw
        .get("actual_status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let state = match status {
        "running" => GpuInstanceState::Running,
        "exited" | "stopped" | "destroyed" => GpuInstanceState::Stopped,
        "error" | "failed" => GpuInstanceState::Failed,
        _ => GpuInstanceState::Allocating,
    };
    Ok(GpuInstance {
        provider: "vast".to_string(),
        resource_id,
        ownership_tag: raw
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        state,
        gpu_model: raw
            .get("gpu_name")
            .and_then(Value::as_str)
            .map(canonical_vast_gpu_name)
            .unwrap_or_else(|| "unknown".to_string()),
        gpu_count: raw
            .get("num_gpus")
            .and_then(Value::as_u64)
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default(),
        host_ram_mib: value_u64_floor(raw.get("cpu_ram")),
        disk_gib: value_u64_floor(raw.get("disk_space")),
        high_bandwidth_interconnect: raw
            .get("bw_nvlink")
            .and_then(Value::as_f64)
            .map(|bandwidth| bandwidth > 0.0),
        hourly_microusd: raw
            .get("dph_total")
            .and_then(parse_usd_micros)
            .unwrap_or_default(),
        created_at_ms: raw
            .get("start_date")
            .and_then(Value::as_f64)
            .map(|seconds| (seconds * 1_000.0) as i64),
        public_ip: raw
            .get("public_ipaddr")
            .and_then(Value::as_str)
            .map(str::to_string),
        ssh_port: raw
            .get("ssh_port")
            .and_then(Value::as_u64)
            .and_then(|value| value.try_into().ok()),
    })
}

fn vast_gpu_name(canonical: &str) -> &str {
    canonical.strip_prefix("NVIDIA ").unwrap_or(canonical)
}

fn canonical_vast_gpu_name(provider_name: &str) -> String {
    if provider_name.starts_with("NVIDIA ") {
        provider_name.to_string()
    } else {
        format!("NVIDIA {provider_name}")
    }
}

fn value_u64_floor(value: Option<&Value>) -> Option<u64> {
    value.and_then(|value| {
        value.as_u64().or_else(|| {
            value
                .as_f64()
                .filter(|number| *number >= 0.0)
                .map(|number| number.floor() as u64)
        })
    })
}

fn vast_cuda_versions(raw: &Value, request: &SearchOffersRequest) -> Vec<String> {
    let Some(maximum) = raw.get("cuda_max_good").and_then(Value::as_f64) else {
        return Vec::new();
    };
    request
        .hardware
        .allowed_cuda_versions
        .iter()
        .filter(|version| {
            version
                .parse::<f64>()
                .is_ok_and(|required| maximum >= required)
        })
        .cloned()
        .collect()
}

fn vast_secure_launch_command(command: &[String], inference_port: u16) -> Vec<String> {
    let bootstrap = format!(
        concat!(
            "set -euo pipefail; port=$1; shift; ",
            "python3 -c 'import hashlib,sys,urllib.request; ",
            "d=urllib.request.urlopen(sys.argv[1], timeout=120).read(); ",
            "assert hashlib.sha256(d).hexdigest()==sys.argv[2]; ",
            "open(sys.argv[3],\"wb\").write(d)' ",
            "'{}' '{}' /tmp/pfterminal-cloudflared; ",
            "chmod 700 /tmp/pfterminal-cloudflared; ",
            "rm -f '{}' '{}'; ",
            "/tmp/pfterminal-cloudflared tunnel --no-autoupdate ",
            "--url \"http://127.0.0.1:$port\" --logfile '{}' --loglevel info ",
            ">/tmp/pfterminal-cloudflared.stdout 2>&1 & ",
            "for i in $(seq 1 120); do ",
            "grep -Eo 'https://[a-z0-9-]+\\.trycloudflare\\.com' '{}' | head -1 > '{}' || true; ",
            "test -s '{}' && break; sleep 1; done; ",
            "test -s '{}'; printf 'PFTERMINAL_TUNNEL_URL=%s\\n' \"$(cat '{}')\"; ",
            "exec \"$@\""
        ),
        CLOUDFLARED_URL,
        CLOUDFLARED_SHA256,
        TUNNEL_URL_PATH,
        TUNNEL_LOG_PATH,
        TUNNEL_LOG_PATH,
        TUNNEL_LOG_PATH,
        TUNNEL_URL_PATH,
        TUNNEL_URL_PATH,
        TUNNEL_URL_PATH,
        TUNNEL_URL_PATH,
    );
    let mut wrapped = vec![
        "bash".to_string(),
        "-lc".to_string(),
        bootstrap,
        "pfterminal-vast-bootstrap".to_string(),
        inference_port.to_string(),
    ];
    wrapped.extend_from_slice(command);
    wrapped
}

fn validate_vast_result_url(result_url: &str) -> ProviderResult<()> {
    let url = reqwest::Url::parse(result_url).map_err(|_| {
        ProviderError::new(
            ProviderErrorKind::Permanent,
            "Vast command result URL is invalid.",
        )
    })?;
    let trusted_host = url.host_str().is_some_and(|host| {
        let regional_s3 = host.ends_with(".amazonaws.com")
            && host.strip_suffix(".amazonaws.com").is_some_and(|prefix| {
                prefix == "s3"
                    || prefix.starts_with("s3.")
                    || prefix.starts_with("s3-")
                    || prefix.contains(".s3.")
                    || prefix.contains(".s3-")
            });
        host == "s3.amazonaws.com" || host.ends_with(".s3.amazonaws.com") || regional_s3
    });
    if url.scheme() != "https" || !trusted_host {
        return Err(ProviderError::new(
            ProviderErrorKind::Permanent,
            "Vast command result URL is not a trusted HTTPS endpoint.",
        ));
    }
    Ok(())
}

fn extract_trycloudflare_url(output: &str) -> Option<&str> {
    output
        .split_whitespace()
        .filter_map(|value| {
            let start = value.find("https://")?;
            Some(value[start..].trim_matches(|character: char| {
                !character.is_ascii_alphanumeric() && !matches!(character, ':' | '/' | '.' | '-')
            }))
        })
        .rfind(|value| {
            value.starts_with("https://")
                && value.ends_with(".trycloudflare.com")
                && value.strip_prefix("https://").is_some_and(|host| {
                    host.bytes().all(|byte| {
                        byte.is_ascii_lowercase()
                            || byte.is_ascii_digit()
                            || matches!(byte, b'-' | b'.')
                    })
                })
        })
}

fn value_id(value: Option<&Value>) -> Option<String> {
    value.and_then(|value| {
        value
            .as_str()
            .map(str::to_string)
            .or_else(|| value.as_u64().map(|id| id.to_string()))
    })
}

fn unix_now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
#[path = "vast_tests.rs"]
mod tests;
