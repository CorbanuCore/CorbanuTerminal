use super::ExecPolicyManager;
use codex_execpolicy::Decision;
use codex_execpolicy::Policy;
use codex_execpolicy::PrefixRule;
use codex_execpolicy::RequirementsExecPolicy;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AllowPrefixRules {
    Honor,
    IgnoreForCyberModel,
}

impl ExecPolicyManager {
    pub(crate) fn current_for_environment(
        &self,
        environment_policy: Option<&RequirementsExecPolicy>,
        allow_prefix_rules: AllowPrefixRules,
    ) -> Arc<Policy> {
        self.current_for_environment_at(
            environment_policy,
            allow_prefix_rules,
            /*live_aggressive*/ false,
        )
    }

    /// [`Self::current_for_environment`] on the policy an agent command is
    /// checked against at the session's live level (Corbanu #443/#626, D52:
    /// see [`ExecPolicyManager::current_for`]).
    pub(crate) fn current_for_environment_at(
        &self,
        environment_policy: Option<&RequirementsExecPolicy>,
        allow_prefix_rules: AllowPrefixRules,
        live_aggressive: bool,
    ) -> Arc<Policy> {
        let policy =
            without_allow_prefix_rules(self.current_for(live_aggressive), allow_prefix_rules);
        match environment_policy {
            Some(environment_policy) => Arc::new(policy.merge_overlay(environment_policy.as_ref())),
            None => policy,
        }
    }

    pub(crate) fn current_for_prefix_rules(
        &self,
        allow_prefix_rules: AllowPrefixRules,
    ) -> Arc<Policy> {
        without_allow_prefix_rules(self.current(), allow_prefix_rules)
    }
}

fn without_allow_prefix_rules(
    policy: Arc<Policy>,
    allow_prefix_rules: AllowPrefixRules,
) -> Arc<Policy> {
    if allow_prefix_rules == AllowPrefixRules::Honor {
        return policy;
    }

    let rules = policy
        .rules()
        .iter_all()
        .flat_map(|(program, rules)| {
            rules.iter().filter_map(move |rule| {
                let is_allow_prefix = rule
                    .as_any()
                    .downcast_ref::<PrefixRule>()
                    .is_some_and(|prefix| prefix.decision == Decision::Allow);
                (!is_allow_prefix).then(|| (program.clone(), Arc::clone(rule)))
            })
        })
        .collect();

    Arc::new(Policy::from_parts(
        rules,
        policy.network_rules().to_vec(),
        policy.host_executables().clone(),
    ))
}

#[cfg(test)]
#[path = "model_policy_tests.rs"]
mod tests;
