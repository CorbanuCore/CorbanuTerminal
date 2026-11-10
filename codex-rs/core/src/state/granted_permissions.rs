//! Permissions granted through `request_permissions`, by environment.
//!
//! D51 (#1001, Travis 2026-10-20): grants are bound to the policy marker
//! (epoch and revocation generation, PF-23-S03) they were granted under, as
//! hosts approved "for session" are (#945). A committed security transition
//! moves the marker, so grants made before it end at it: they are dropped on
//! their next use, and a grant answered before it is not recorded after it.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;

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

/// Where a [`SessionGrantStore`] reads the policy marker in force; `None` when it cannot be
/// read (no local agent backend), which grants nothing.
type AuthorityMarkerSource = Arc<dyn Fn() -> Option<AuthorityMarker> + Send + Sync>;

/// D520 (a), provisional: the session's "for session" grants. Upstream shares one bare map
/// with every turn and captured step; this store checks the policy marker in force on every
/// read and write instead, so grants made before a security transition end at it whichever
/// reader reaches them.
#[derive(Clone)]
pub(crate) struct SessionGrantStore {
    grants: Arc<StdMutex<GrantedPermissions>>,
    authority_marker: AuthorityMarkerSource,
}

impl SessionGrantStore {
    pub(crate) fn new(
        authority_marker: impl Fn() -> Option<AuthorityMarker> + Send + Sync + 'static,
    ) -> Self {
        Self {
            grants: Arc::default(),
            authority_marker: Arc::new(authority_marker),
        }
    }

    /// The policy marker in force; `None` when it cannot be read, which grants nothing.
    pub(crate) fn authority_marker(&self) -> Option<AuthorityMarker> {
        (self.authority_marker)()
    }

    /// Records a grant answered to a request shown under `granted_under`; whether it was
    /// recorded. A poisoned lock records nothing.
    pub(crate) fn record(
        &self,
        environment_id: &str,
        permissions: AdditionalPermissionProfile,
        granted_under: AuthorityMarker,
    ) -> bool {
        let Ok(mut grants) = self.grants.lock() else {
            return false;
        };
        // Read under the lock: a later grant is never recorded under an earlier marker.
        let Some(current) = self.authority_marker() else {
            return false;
        };
        grants.record(environment_id, permissions, granted_under, current)
    }

    /// The grants for `environment_id` still in force. A poisoned lock grants nothing.
    pub(crate) fn get(&self, environment_id: &str) -> Option<AdditionalPermissionProfile> {
        let mut grants = self.grants.lock().ok()?;
        let current = self.authority_marker()?;
        grants.get(environment_id, current)
    }
}

impl std::fmt::Debug for SessionGrantStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionGrantStore")
            .field("grants", &self.grants)
            .finish_non_exhaustive()
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

    #[test]
    fn d520_session_store_follows_the_live_policy_marker() {
        let marker = Arc::new(StdMutex::new(Some((1, 0))));
        let source = Arc::clone(&marker);
        let store = SessionGrantStore::new(move || Some(*source.lock().expect("marker")));
        let shared = store.clone();
        assert!(store.record("local", network(), Some((1, 0))));
        assert_eq!(shared.get("local"), Some(network()), "shared with clones");
        // A transition committed elsewhere: every holder loses the grant.
        *marker.lock().expect("marker") = Some((2, 0));
        assert_eq!(shared.get("local"), None);
        *marker.lock().expect("marker") = Some((1, 0));
        assert_eq!(store.get("local"), None, "not restored");
        // Answered before a transition, recorded after it: not kept.
        *marker.lock().expect("marker") = Some((3, 0));
        assert!(!store.record("local", network(), Some((1, 0))));
        assert_eq!(store.get("local"), None);
    }

    #[test]
    fn d520_session_store_grants_nothing_without_a_readable_marker() {
        let store = SessionGrantStore::new(|| None);
        assert!(!store.record("local", network(), /*granted_under*/ None));
        assert_eq!(store.get("local"), None);
    }
}
