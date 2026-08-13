//! Current public product terminology and canonical distribution locations.
//!
//! Persisted protocol identifiers intentionally do not live here: changing a
//! display name must not silently change stored provider IDs, credential keys,
//! HTTP headers, or wallet signature domains.

pub const PRODUCT_NAME: &str = "Corbanu Terminal";
pub const LEGACY_PRODUCT_NAME: &str = "PFTerminal";
pub const TASK_NODE_NAME: &str = "Post Fiat Task Node";
pub const INFERENCE_NAME: &str = "Ambient Inference";
pub const NEWSLETTER_NAME: &str = "Corbanu Newsletter";
pub const PLAN_NAME: &str = "Corbanu Terminal Plan";

pub const RELATIONSHIP_SENTENCE: &str =
    "Corbanu Terminal integrates Post Fiat Task Node and Ambient Inference.";

pub const GITHUB_REPOSITORY: &str = "CorbanuCore/CorbanuTerminal";
pub const GITHUB_REPOSITORY_URL: &str = "https://github.com/CorbanuCore/CorbanuTerminal";
pub const GITHUB_LATEST_RELEASE_URL: &str =
    "https://github.com/CorbanuCore/CorbanuTerminal/releases/latest";
pub const INSTALL_SH_URL: &str =
    "https://github.com/CorbanuCore/CorbanuTerminal/releases/latest/download/install.sh";
pub const INSTALL_PS1_URL: &str =
    "https://github.com/CorbanuCore/CorbanuTerminal/releases/latest/download/install.ps1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_relationship_keeps_product_boundaries_distinct() {
        assert!(RELATIONSHIP_SENTENCE.contains(PRODUCT_NAME));
        assert!(RELATIONSHIP_SENTENCE.contains(TASK_NODE_NAME));
        assert!(RELATIONSHIP_SENTENCE.contains(INFERENCE_NAME));
        assert!(!RELATIONSHIP_SENTENCE.contains("Corbanu Tasks"));
    }

    #[test]
    fn canonical_distribution_urls_use_the_transferred_repository() {
        assert!(GITHUB_REPOSITORY_URL.ends_with(GITHUB_REPOSITORY));
        assert!(GITHUB_LATEST_RELEASE_URL.starts_with(GITHUB_REPOSITORY_URL));
        assert!(INSTALL_SH_URL.starts_with(GITHUB_REPOSITORY_URL));
        assert!(INSTALL_PS1_URL.starts_with(GITHUB_REPOSITORY_URL));
    }
}
