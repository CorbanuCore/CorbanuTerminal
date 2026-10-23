//! Permissions granted through `request_permissions`, by environment.
//!
//! D51 (#1001, Travis 2026-10-20): grants are bound to the policy marker
//! (epoch and revocation generation, PF-23-S03) they were granted under, as
//! hosts approved "for session" are (#945). A committed security transition
//! moves the marker, so grants made before it end at it: they are dropped on
//! their next use, and a grant answered before it is not recorded after it.

use std::collections::HashMap;

use codex_protocol::models::AdditionalPermissionProfile;
use codex_sandboxing::policy_transforms::merge_permission_profiles;

/// The policy epoch and revocation generation (PF-23-S03).
pub(crate) type AuthorityMarker = Option<(u64, u64)>;

#[derive(Debug, Default)]
pub(crate) struct GrantedPermissions {
    policy: AuthorityMarker,
    by_environment_id: HashMap<String, AdditionalPermissionProfile>,
}

impl GrantedPermissions {
    /// Drops every grant made under another policy than `current`. Grants
    /// recorded before the session's policy tree was set up (no marker) end
    /// when it is: conservative, and a real session sets it up first.
    fn fence(&mut self, current: AuthorityMarker) {
        if self.policy != current {
            self.by_environment_id.clear();
            self.policy = current;
        }
    }

    /// Records a grant given under `granted_under`, only while that is still
    /// the policy in force (`current`); whether it was recorded.
    pub(crate) fn record(
        &mut self,
        environment_id: &str,
        permissions: AdditionalPermissionProfile,
        granted_under: AuthorityMarker,
        current: AuthorityMarker,
    ) -> bool {
        self.fence(current);
        if granted_under.is_some() && granted_under != current {
            return false;
        }
        if let Some(merged) = merge_permission_profiles(
            self.by_environment_id.get(environment_id),
            Some(&permissions),
        ) {
            self.by_environment_id
                .insert(environment_id.to_string(), merged);
        }
        true
    }

    /// The grants for `environment_id` still in force under `current`.
    pub(crate) fn get(
        &mut self,
        environment_id: &str,
        current: AuthorityMarker,
    ) -> Option<AdditionalPermissionProfile> {
        self.fence(current);
        self.by_environment_id.get(environment_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::models::NetworkPermissions;
    use pretty_assertions::assert_eq;

    fn network() -> AdditionalPermissionProfile {
        AdditionalPermissionProfile {
            network: Some(NetworkPermissions {
                enabled: Some(true),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn issue_1001_grants_follow_the_policy_marker() {
        let (before, after) = (Some((1, 0)), Some((2, 0)));
        let mut grants = GrantedPermissions::default();
        assert!(grants.record("local", network(), before, before));
        assert_eq!(grants.get("local", before), Some(network()));
        // A transition: the grant ends.
        assert_eq!(grants.get("local", after), None);
        assert_eq!(grants.get("local", before), None, "not restored");
        // Answered before it, recorded after it: not kept.
        assert!(!grants.record("local", network(), before, after));
        assert_eq!(grants.get("local", after), None);
        // No marker when the request was shown: kept, as #945 counts it.
        assert!(grants.record("local", network(), /*granted_under*/ None, after));
        assert_eq!(grants.get("local", after), Some(network()));
        assert_eq!(grants.get("remote", after), None);
    }
}
