use super::authoritative_state::AuthoritativeStateAnchor;
use super::authoritative_state::AuthoritativeStateAnchorError;
use super::authoritative_state::AuthoritativeStateAnchorStore;
use super::authoritative_state_anchor::NativeAuthoritativeStateAnchor;
use codex_config::AuthoritativeStateOwner;
use codex_protected_state::PolicyCheckpoint;
use codex_protected_state::PolicyRootStore;
use codex_protected_state::RootError;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Default)]
struct NativeContract(Mutex<Option<PolicyCheckpoint>>);
impl PolicyRootStore for NativeContract {
    fn load_policy(&self) -> Result<Option<PolicyCheckpoint>, RootError> { Ok(self.0.lock().unwrap().clone()) }
    fn compare_policy(&self, expected: Option<&PolicyCheckpoint>, next: &PolicyCheckpoint) -> Result<(), RootError> {
        let mut value = self.0.lock().unwrap();
        if value.as_ref() != expected { return Err(RootError::Conflict); }
        *value = Some(next.clone()); Ok(())
    }
}

#[test]
fn pf20_s03_core_adapter_preserves_exact_anchor_and_compare() {
    let storage = Arc::new(NativeContract::default());
    let adapter = NativeAuthoritativeStateAnchor(storage.clone());
    let anchor = AuthoritativeStateAnchor { schema_version: 1, revision: 1, owner: AuthoritativeStateOwner::new("a".repeat(64), "controller", 1).unwrap(), state_sha256: "b".repeat(64), commit_sha256: "c".repeat(64) };
    adapter.compare_and_store_anchor(None, &anchor).unwrap();
    assert_eq!(adapter.load_anchor().unwrap(), Some(anchor.clone()));
    assert_eq!(storage.load_policy().unwrap(), Some(PolicyCheckpoint::from(&anchor)));
    assert_eq!(adapter.compare_and_store_anchor(None, &anchor), Err(AuthoritativeStateAnchorError::Conflict));
}
