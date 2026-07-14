use crate::GpuOffer;
use crate::GpuProvider;
use crate::GpuRecipe;
use crate::ProviderError;
use crate::ProviderErrorKind;
use crate::RecipeCatalog;
use crate::SearchOffersRequest;
use codex_state::GpuLimitEnforcement;
use codex_state::GpuRental;
use codex_state::GpuRentalCreateParams;
use codex_state::StateRuntime;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RentalAuthorization {
    pub client_operation_id: String,
    pub maximum_hourly_microusd: i64,
    pub maximum_total_microusd: i64,
    pub terminate_at_ms: i64,
    pub acknowledged_local_enforcement: bool,
}

pub struct GpuMarketService {
    state: Arc<StateRuntime>,
    recipes: RecipeCatalog,
    installation_id: String,
}

impl GpuMarketService {
    pub fn new(state: Arc<StateRuntime>, recipes: RecipeCatalog, installation_id: String) -> Self {
        Self {
            state,
            recipes,
            installation_id,
        }
    }

    pub async fn search<P1, P2>(
        &self,
        recipe_id: &str,
        maximum_hourly_microusd: i64,
        first: &P1,
        second: &P2,
    ) -> Result<Vec<GpuOffer>, ProviderError>
    where
        P1: GpuProvider,
        P2: GpuProvider,
    {
        let recipe = self.verified_recipe(recipe_id)?;
        let request = search_request(recipe, maximum_hourly_microusd)?;
        let (first_result, second_result) = tokio::join!(
            first.search_offers(request.clone()),
            second.search_offers(request)
        );
        let mut offers = Vec::new();
        merge_search_result(&mut offers, first_result)?;
        merge_search_result(&mut offers, second_result)?;
        offers.sort_by_key(|offer| {
            (
                offer.hourly_microusd,
                offer.storage_microusd_per_gib_month.unwrap_or(i64::MAX),
                offer.provider.clone(),
                offer.offer_id.clone(),
            )
        });
        Ok(offers)
    }

    pub async fn confirm<P>(
        &self,
        recipe_id: &str,
        selected_offer: &GpuOffer,
        authorization: &RentalAuthorization,
        provider: &P,
        now_ms: i64,
    ) -> Result<GpuRental, ProviderError>
    where
        P: GpuProvider,
    {
        let recipe = self.verified_recipe(recipe_id)?;
        validate_authorization(authorization, now_ms)?;
        let capabilities = provider.capabilities();
        if capabilities.provider != selected_offer.provider {
            return Err(ProviderError::new(
                ProviderErrorKind::InvalidRequest,
                "The selected offer belongs to a different provider.",
            ));
        }
        let local_enforcement =
            !capabilities.supports_native_ttl || !capabilities.supports_native_spend_cap;
        if local_enforcement && !authorization.acknowledged_local_enforcement {
            return Err(ProviderError::new(
                ProviderErrorKind::InvalidRequest,
                "This provider requires acknowledgement that spend and TTL enforcement depend on the local controller.",
            ));
        }

        let request = search_request(recipe, authorization.maximum_hourly_microusd)?;
        let current = provider
            .search_offers(request)
            .await?
            .into_iter()
            .find(|offer| offer.offer_id == selected_offer.offer_id)
            .ok_or_else(|| {
                ProviderError::new(
                    ProviderErrorKind::OfferUnavailable,
                    "The selected GPU offer is no longer available.",
                )
            })?;
        if current.hourly_microusd != selected_offer.hourly_microusd
            || current.hourly_microusd > authorization.maximum_hourly_microusd
        {
            return Err(ProviderError::new(
                ProviderErrorKind::PriceDrift,
                "The selected GPU offer price changed; confirmation is required again.",
            ));
        }
        current.validate_for(
            &search_request(recipe, authorization.maximum_hourly_microusd)?,
            now_ms,
        )?;

        let rental_id = format!("gpu-{}", authorization.client_operation_id);
        let rental = self
            .state
            .create_gpu_rental(
                &GpuRentalCreateParams {
                    rental_id: rental_id.clone(),
                    installation_id: self.installation_id.clone(),
                    client_operation_id: authorization.client_operation_id.clone(),
                    provider: current.provider.clone(),
                    recipe_id: recipe.id.clone(),
                    recipe_revision: recipe.revision.clone(),
                    offer_snapshot_json: serde_json::to_string(&current).map_err(|_| {
                        ProviderError::new(
                            ProviderErrorKind::Permanent,
                            "The verified offer could not be recorded.",
                        )
                    })?,
                    quote_expires_at_ms: current.expires_at_ms,
                    max_hourly_microusd: authorization.maximum_hourly_microusd,
                    max_total_microusd: authorization.maximum_total_microusd,
                    terminate_at_ms: authorization.terminate_at_ms,
                    enforcement_class: if local_enforcement {
                        GpuLimitEnforcement::LocalControllerDependent
                    } else {
                        GpuLimitEnforcement::ProviderGuaranteed
                    },
                    ownership_tag: format!("pft-{}-{rental_id}", self.installation_id),
                },
                now_ms,
            )
            .await
            .map_err(state_error)?;
        self.state
            .request_gpu_rental_creation(rental.rental_id.as_str(), now_ms)
            .await
            .map_err(state_error)?;
        self.state
            .get_gpu_rental(rental.rental_id.as_str())
            .await
            .map_err(state_error)?
            .ok_or_else(|| {
                ProviderError::new(
                    ProviderErrorKind::Permanent,
                    "The authorized rental was not durably recorded.",
                )
            })
    }

    fn verified_recipe(&self, recipe_id: &str) -> Result<&GpuRecipe, ProviderError> {
        let recipe = self.recipes.get(recipe_id).ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::InvalidRequest,
                "GPU recipe was not found.",
            )
        })?;
        if !recipe.manifest_verified {
            return Err(ProviderError::new(
                ProviderErrorKind::InvalidRequest,
                "GPU recipe creation is disabled until its immutable manifest is verified.",
            ));
        }
        Ok(recipe)
    }
}

fn search_request(
    recipe: &GpuRecipe,
    maximum_hourly_microusd: i64,
) -> Result<SearchOffersRequest, ProviderError> {
    if maximum_hourly_microusd <= 0 {
        return Err(ProviderError::new(
            ProviderErrorKind::InvalidRequest,
            "Maximum hourly price must be positive.",
        ));
    }
    Ok(SearchOffersRequest {
        hardware: recipe.hardware.clone(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd,
    })
}

fn validate_authorization(
    authorization: &RentalAuthorization,
    now_ms: i64,
) -> Result<(), ProviderError> {
    if authorization.client_operation_id.trim().is_empty()
        || authorization.client_operation_id.len() > 100
        || !authorization
            .client_operation_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        || authorization.maximum_hourly_microusd <= 0
        || authorization.maximum_total_microusd <= 0
        || authorization.terminate_at_ms <= now_ms
    {
        return Err(ProviderError::new(
            ProviderErrorKind::InvalidRequest,
            "GPU rental authorization is incomplete or expired.",
        ));
    }
    Ok(())
}

fn merge_search_result(
    offers: &mut Vec<GpuOffer>,
    result: Result<Vec<GpuOffer>, ProviderError>,
) -> Result<(), ProviderError> {
    match result {
        Ok(found) => offers.extend(found),
        Err(error)
            if matches!(
                error.kind,
                ProviderErrorKind::OfferUnavailable | ProviderErrorKind::RateLimited
            ) => {}
        Err(error) => return Err(error),
    }
    Ok(())
}

fn state_error(error: anyhow::Error) -> ProviderError {
    tracing::warn!(error = %error, "GPU rental state operation failed");
    ProviderError::new(
        ProviderErrorKind::Permanent,
        "The GPU rental ledger could not record the operation.",
    )
}

#[cfg(test)]
#[path = "market_tests.rs"]
mod tests;
