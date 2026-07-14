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
use chrono::DateTime;
use serde_json::Value;
use std::sync::Arc;

const DEFAULT_REST_BASE: &str = "https://rest.runpod.io/v1";
const DEFAULT_GRAPHQL_URL: &str = "https://api.runpod.io/graphql";

#[derive(Clone)]
pub struct RunpodProvider {
    client: reqwest::Client,
    credentials: Arc<dyn GpuCredentialResolver>,
    rest_base: String,
    graphql_url: String,
}

impl std::fmt::Debug for RunpodProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunpodProvider")
            .field("rest_base", &self.rest_base)
            .field("graphql_url", &self.graphql_url)
            .finish_non_exhaustive()
    }
}

impl RunpodProvider {
    pub fn new(credentials: Arc<dyn GpuCredentialResolver>) -> Self {
        Self::with_endpoints(credentials, DEFAULT_REST_BASE, DEFAULT_GRAPHQL_URL)
    }

    pub fn with_endpoints(
        credentials: Arc<dyn GpuCredentialResolver>,
        rest_base: impl Into<String>,
        graphql_url: impl Into<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            credentials,
            rest_base: rest_base.into().trim_end_matches('/').to_string(),
            graphql_url: graphql_url.into(),
        }
    }

    fn api_key(&self) -> ProviderResult<crate::GpuCredential> {
        self.credentials
            .resolve(&GpuCredentialKind::ProviderApiKey {
                provider: "runpod".to_string(),
            })
            .map_err(credential_error)
    }

    async fn current_offer(&self, request: &SearchOffersRequest) -> ProviderResult<GpuOffer> {
        let key = self.api_key()?;
        let body = serde_json::json!({
            "query": "query PftGpuOffer($id: String!, $count: Int!) { gpuTypes(input: { id: $id }) { id displayName memoryInGb secureCloud lowestPrice(input: { gpuCount: $count, secureCloud: true }) { stockStatus uninterruptablePrice availableGpuCounts } } }",
            "variables": {
                "id": request.hardware.gpu_model,
                "count": request.hardware.gpu_count,
            }
        });
        let response = self
            .client
            .post(self.graphql_url.as_str())
            .bearer_auth(key.secret.expose())
            .json(&body)
            .send()
            .await
            .map_err(|_| transport_error())?;
        let json = decode_json(response).await?;
        if json.get("errors").is_some() {
            return Err(ProviderError::new(
                ProviderErrorKind::Permanent,
                "RunPod rejected the GPU inventory query.",
            ));
        }
        let gpu = json.pointer("/data/gpuTypes/0").ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::OfferUnavailable,
                "RunPod Secure Cloud has no compatible GPU offer.",
            )
        })?;
        if gpu.get("secureCloud").and_then(Value::as_bool) != Some(true) {
            return Err(ProviderError::new(
                ProviderErrorKind::OfferUnavailable,
                "Requested GPU is not available in RunPod Secure Cloud.",
            ));
        }
        let price = gpu
            .pointer("/lowestPrice/uninterruptablePrice")
            .and_then(parse_usd_micros)
            .ok_or_else(|| {
                ProviderError::new(
                    ProviderErrorKind::Permanent,
                    "RunPod did not return an authoritative on-demand price.",
                )
            })?;
        let available_counts = gpu
            .pointer("/lowestPrice/availableGpuCounts")
            .and_then(Value::as_array)
            .is_some_and(|counts| {
                counts
                    .iter()
                    .any(|count| count.as_u64() == u64::try_from(request.hardware.gpu_count).ok())
            });
        if !available_counts {
            return Err(ProviderError::new(
                ProviderErrorKind::OfferUnavailable,
                "RunPod does not have the requested Secure Cloud GPU count.",
            ));
        }
        let now_ms = unix_now_ms();
        Ok(GpuOffer {
            provider: "runpod".to_string(),
            offer_id: format!(
                "secure:{}:{}:{}",
                request.hardware.gpu_model, request.hardware.gpu_count, price
            ),
            gpu_model: request.hardware.gpu_model.clone(),
            gpu_count: request.hardware.gpu_count,
            vram_mib_per_gpu: gpu
                .get("memoryInGb")
                .and_then(Value::as_u64)
                .and_then(|gib| gib.checked_mul(1024))
                .and_then(|mib| u64::try_into(mib).ok())
                .unwrap_or_default(),
            host_ram_mib: request.hardware.minimum_host_ram_mib,
            disk_gib: request.hardware.minimum_disk_gib,
            high_bandwidth_interconnect: false,
            cuda_versions: request.hardware.allowed_cuda_versions.clone(),
            region: "RunPod Secure Cloud".to_string(),
            security_class: "secure".to_string(),
            reliability_millionths: None,
            interruptible: false,
            hourly_microusd: price,
            storage_microusd_per_gib_month: None,
            quoted_at_ms: now_ms,
            expires_at_ms: Some(now_ms.saturating_add(30_000)),
            raw_snapshot: gpu.clone(),
        })
    }

    async fn send_pod_get(&self, resource_id: Option<&str>) -> ProviderResult<reqwest::Response> {
        let key = self.api_key()?;
        let url = match resource_id {
            Some(resource_id) => format!("{}/pods/{resource_id}", self.rest_base),
            None => format!("{}/pods", self.rest_base),
        };
        self.client
            .get(url)
            .bearer_auth(key.secret.expose())
            .send()
            .await
            .map_err(|_| transport_error())
    }
}

impl GpuProvider for RunpodProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            provider: "runpod".to_string(),
            supports_ownership_tags: true,
            supports_inventory: true,
            supports_native_ttl: false,
            supports_native_spend_cap: false,
            security_classes: vec!["secure".to_string()],
        }
    }

    async fn search_offers(&self, request: SearchOffersRequest) -> ProviderResult<Vec<GpuOffer>> {
        match self.current_offer(&request).await {
            Ok(offer) if offer.validate_for(&request, unix_now_ms()).is_ok() => Ok(vec![offer]),
            Ok(_) => Ok(Vec::new()),
            Err(error) if error.kind == ProviderErrorKind::OfferUnavailable => Ok(Vec::new()),
            Err(error) => Err(error),
        }
    }

    async fn create_instance(&self, request: CreateInstanceRequest) -> ProviderResult<GpuInstance> {
        let requirements = SearchOffersRequest {
            hardware: crate::HardwareRequirements {
                gpu_model: request.offer.gpu_model.clone(),
                gpu_count: request.offer.gpu_count,
                minimum_vram_mib_per_gpu: request.offer.vram_mib_per_gpu,
                minimum_host_ram_mib: request.offer.host_ram_mib,
                minimum_disk_gib: request.disk_gib,
                requires_high_bandwidth_interconnect: request.offer.high_bandwidth_interconnect,
                allowed_cuda_versions: request.offer.cuda_versions.clone(),
            },
            allow_interruptible: false,
            require_verified_or_secure: true,
            maximum_hourly_microusd: request.offer.hourly_microusd,
        };
        let current = self.current_offer(&requirements).await?;
        if current.offer_id != request.offer.offer_id
            || current.hourly_microusd != request.offer.hourly_microusd
        {
            return Err(ProviderError::new(
                ProviderErrorKind::PriceDrift,
                "RunPod offer changed after confirmation; creation was not attempted.",
            ));
        }
        let key = self.api_key()?;
        let min_ram_per_gpu_gib = request
            .offer
            .host_ram_mib
            .div_ceil(u64::from(request.offer.gpu_count).max(1))
            .div_ceil(1024);
        let body = serde_json::json!({
            "name": request.ownership_tag,
            "imageName": request.image,
            "cloudType": "SECURE",
            "computeType": "GPU",
            "interruptible": false,
            "gpuTypeIds": [request.offer.gpu_model],
            "gpuTypePriority": "custom",
            "gpuCount": request.offer.gpu_count,
            "allowedCudaVersions": request.offer.cuda_versions,
            "minRAMPerGPU": min_ram_per_gpu_gib,
            "containerDiskInGb": request.disk_gib,
            "volumeInGb": 0,
            "supportPublicIp": true,
            "startSsh": true,
        });
        let response = self
            .client
            .post(format!("{}/pods", self.rest_base))
            .bearer_auth(key.secret.expose())
            .json(&body)
            .send()
            .await
            .map_err(|_| transport_error())?;
        let json = decode_json(response).await?;
        pod_to_instance(&json, Some(request.ownership_tag))
    }

    async fn get_instance(&self, resource_id: String) -> ProviderResult<Option<GpuInstance>> {
        let response = self.send_pod_get(Some(resource_id.as_str())).await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let json = decode_json(response).await?;
        pod_to_instance(&json, None).map(Some)
    }

    async fn list_owned_instances(
        &self,
        query: OwnedInstanceQuery,
    ) -> ProviderResult<Vec<GpuInstance>> {
        let response = self.send_pod_get(None).await?;
        let json = decode_json(response).await?;
        let pods = json.as_array().ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::Permanent,
                "RunPod returned malformed Pod inventory.",
            )
        })?;
        pods.iter()
            .filter(|pod| {
                query
                    .ownership_tag
                    .as_ref()
                    .is_none_or(|tag| pod.get("name").and_then(Value::as_str) == Some(tag))
            })
            .map(|pod| pod_to_instance(pod, None))
            .collect()
    }

    async fn terminate_instance(&self, resource_id: String) -> ProviderResult<()> {
        let key = self.api_key()?;
        let response = self
            .client
            .delete(format!("{}/pods/{resource_id}", self.rest_base))
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

fn pod_to_instance(pod: &Value, fallback_tag: Option<String>) -> ProviderResult<GpuInstance> {
    let resource_id = pod.get("id").and_then(Value::as_str).ok_or_else(|| {
        ProviderError::new(
            ProviderErrorKind::Permanent,
            "RunPod response omitted the Pod id.",
        )
    })?;
    let gpu = pod.get("gpu").unwrap_or(&Value::Null);
    let state = match pod.get("desiredStatus").and_then(Value::as_str) {
        Some("RUNNING") => GpuInstanceState::Running,
        Some("EXITED" | "TERMINATED") => GpuInstanceState::Stopped,
        Some(_) | None => GpuInstanceState::Allocating,
    };
    Ok(GpuInstance {
        provider: "runpod".to_string(),
        resource_id: resource_id.to_string(),
        ownership_tag: pod
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or(fallback_tag)
            .unwrap_or_default(),
        state,
        gpu_model: gpu
            .get("id")
            .or_else(|| gpu.get("displayName"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        gpu_count: gpu
            .get("count")
            .and_then(Value::as_u64)
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default(),
        hourly_microusd: pod
            .get("adjustedCostPerHr")
            .or_else(|| pod.get("costPerHr"))
            .and_then(parse_usd_micros)
            .unwrap_or_default(),
        created_at_ms: pod
            .get("lastStartedAt")
            .and_then(Value::as_str)
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp_millis()),
        public_ip: pod
            .get("publicIp")
            .and_then(Value::as_str)
            .map(str::to_string),
        ssh_port: pod
            .pointer("/portMappings/22")
            .and_then(Value::as_u64)
            .and_then(|value| value.try_into().ok()),
    })
}

fn unix_now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
#[path = "runpod_tests.rs"]
mod tests;
