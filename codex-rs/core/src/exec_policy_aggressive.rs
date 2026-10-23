//! #626: remembered command approvals under security level Aggressive.
//!
//! A remembered approval ("Yes, and don't ask again for commands that start
//! with …") is an `allow` prefix rule. Rules carried no level, so while
//! Aggressive is in force no `allow` prefix rule approves a command unless a
//! person saved it while Aggressive was in force (Travis, 2026-10-13): such a
//! rule is written to the Corbanu home's `rules/default.rules` with
//! [`APPROVED_UNDER_AGGRESSIVE`] as its justification, and only that file may
//! say so (a project's `.codex/rules` cannot). Every other `allow` prefix rule
//! (also one an administrator or a system layer set) is suspended, not
//! deleted: the person is asked as usual, the prompt says
//! which remembered approval was not applied, and under Permissive the rules
//! apply as before. `prompt` and `forbidden` rules always apply.
//!
//! D52 (#1001, Travis 2026-10-20): network `allow` rules ("allow this host
//! always", which widen a managed network proxy's allowlist) are suspended
//! the same way: only one saved while Aggressive was in force, marked the
//! same way in the same file, applies under Aggressive. `deny` rules apply.
//!
//! The mark means "the Corbanu home's rules file says so": anything that can
//! write that file outside Aggressive (a person, a command run with full
//! access) can add one. That is no weaker than before, when every rule in it
//! applied under Aggressive; under Aggressive the file is read-only to agent
//! commands.

use std::path::Path;

use codex_config::ConfigLayerSource;
use codex_config::ConfigLayerStack;
use codex_config::ConfigLayerStackOrdering;
use codex_execpolicy::Decision;
use codex_execpolicy::Evaluation;
use codex_execpolicy::PatternToken;
use codex_execpolicy::Policy;
use codex_execpolicy::PolicyParser;
use codex_execpolicy::PrefixRule;
use codex_execpolicy::RuleMatch;

/// The justification of a rule a person saved while Aggressive was active.
pub(crate) const APPROVED_UNDER_AGGRESSIVE: &str =
    "Remembered while security level Aggressive was active.";

/// The prefixes (and, D52, the hosts of network `allow` rules) the Corbanu
/// home's `rules/default.rules` records as approved while Aggressive was
/// active (the only file trusted to say so), from one read. A file that is
/// missing or does not parse records none.
pub(crate) async fn load_approved(
    config_stack: &ConfigLayerStack,
) -> (Vec<Vec<String>>, Vec<String>) {
    match read_home_rules(config_stack).await {
        Some((path, contents)) => (
            approved_in(&path, &contents),
            approved_hosts_in(&path, &contents),
        ),
        None => (Vec::new(), Vec::new()),
    }
}

/// The Corbanu home's `rules/default.rules`, when rules are read at all.
async fn read_home_rules(config_stack: &ConfigLayerStack) -> Option<(std::path::PathBuf, String)> {
    if config_stack.ignore_user_and_project_exec_policy_rules() {
        return None;
    }
    let folder = config_stack
        .get_layers(
            ConfigLayerStackOrdering::LowestPrecedenceFirst,
            /*include_disabled*/ false,
        )
        .into_iter()
        .filter(|layer| matches!(layer.name, ConfigLayerSource::User { .. }))
        .find_map(codex_config::ConfigLayerEntry::config_folder)?;
    let path = crate::exec_policy::default_policy_path(folder.as_path());
    let contents = tokio::fs::read_to_string(&path).await.ok()?;
    Some((path, contents))
}

/// The prefixes of `contents`' `allow` prefix rules marked
/// [`APPROVED_UNDER_AGGRESSIVE`].
pub(crate) fn approved_in(path: &Path, contents: &str) -> Vec<Vec<String>> {
    let mut parser = PolicyParser::new();
    if parser
        .parse(&path.to_string_lossy(), contents)
        .inspect_err(|err| tracing::warn!("rules approved under Aggressive not read: {err}"))
        .is_err()
    {
        return Vec::new();
    }
    let policy = parser.build();
    let mut approved: Vec<Vec<String>> = policy
        .rules()
        .iter_all()
        .flat_map(|(_, rules)| rules)
        .filter_map(|rule| rule.as_any().downcast_ref::<PrefixRule>())
        .filter(|rule| marked(rule))
        .filter_map(prefix_of)
        .collect();
    approved.sort();
    approved.dedup();
    approved
}

/// D52: the hosts of `contents`' network `allow` rules marked
/// [`APPROVED_UNDER_AGGRESSIVE`].
pub(crate) fn approved_hosts_in(path: &Path, contents: &str) -> Vec<String> {
    let mut parser = PolicyParser::new();
    if parser
        .parse(&path.to_string_lossy(), contents)
        .inspect_err(|err| {
            tracing::warn!("network rules approved under Aggressive not read: {err}")
        })
        .is_err()
    {
        return Vec::new();
    }
    let mut hosts: Vec<String> = parser
        .build()
        .network_rules()
        .iter()
        .filter(|rule| network_rule_marked(rule.decision, rule.justification.as_deref()))
        .map(|rule| rule.host.clone())
        .collect();
    hosts.sort();
    hosts.dedup();
    hosts
}

fn network_rule_marked(decision: Decision, justification: Option<&str>) -> bool {
    decision == Decision::Allow && justification == Some(APPROVED_UNDER_AGGRESSIVE)
}

/// D52: `policy` without its network `allow` rules, except those marked
/// [`APPROVED_UNDER_AGGRESSIVE`] whose host is in `approved_hosts`.
pub(crate) fn suspend_unapproved_network(policy: &Policy, approved_hosts: &[String]) -> Policy {
    let network_rules = policy
        .network_rules()
        .iter()
        .filter(|rule| {
            rule.decision != Decision::Allow
                || (network_rule_marked(rule.decision, rule.justification.as_deref())
                    && approved_hosts.contains(&rule.host))
        })
        .cloned()
        .collect();
    Policy::from_parts(
        policy.rules().clone(),
        network_rules,
        policy.host_executables().clone(),
    )
}

fn marked(rule: &PrefixRule) -> bool {
    rule.decision == Decision::Allow
        && rule.justification.as_deref() == Some(APPROVED_UNDER_AGGRESSIVE)
}

/// The words of a rule with one word per position (as saved approvals are).
fn prefix_of(rule: &PrefixRule) -> Option<Vec<String>> {
    std::iter::once(Some(rule.pattern.first.to_string()))
        .chain(rule.pattern.rest.iter().map(|token| match token {
            PatternToken::Single(word) => Some(word.clone()),
            PatternToken::Alts(_) => None,
        }))
        .collect()
}

/// `policy` without its `allow` prefix rules, except those marked
/// [`APPROVED_UNDER_AGGRESSIVE`] whose prefix is in `approved`.
pub(crate) fn suspend_unapproved(policy: &Policy, approved: &[Vec<String>]) -> Policy {
    let mut rules = policy.rules().clone();
    rules.retain(|_, rule| match rule.as_any().downcast_ref::<PrefixRule>() {
        Some(rule) if rule.decision == Decision::Allow => {
            marked(rule) && prefix_of(rule).is_some_and(|prefix| approved.contains(&prefix))
        }
        _ => true,
    });
    Policy::from_parts(
        rules,
        policy.network_rules().to_vec(),
        policy.host_executables().clone(),
    )
}

/// The suspended remembered approvals in `full` (the evaluation without
/// suspension) when they would have approved the command unasked, as the
/// prompt names them; those in `approved` applied and are not named.
pub(crate) fn suspended_prefixes(full: &Evaluation, approved: &[Vec<String>]) -> Vec<Vec<String>> {
    if full.decision != Decision::Allow {
        return Vec::new();
    }
    let mut prefixes: Vec<Vec<String>> = full
        .matched_rules
        .iter()
        .filter_map(|rule_match| match rule_match {
            RuleMatch::PrefixRuleMatch {
                matched_prefix,
                decision: Decision::Allow,
                justification,
                ..
            } if !(justification.as_deref() == Some(APPROVED_UNDER_AGGRESSIVE)
                && approved.contains(matched_prefix)) =>
            {
                Some(matched_prefix.clone())
            }
            _ => None,
        })
        .collect();
    prefixes.sort();
    prefixes.dedup();
    prefixes
}

/// What the approval prompt says about `prefixes` (see [`suspended_prefixes`]).
pub(crate) fn suspended_note(prefixes: &[Vec<String>]) -> Option<String> {
    let shown = prefixes
        .iter()
        .map(|prefix| {
            let words = shlex::try_join(prefix.iter().map(String::as_str))
                .unwrap_or_else(|_| prefix.join(" "));
            format!("`{}`", crate::security::protected_reads::shown_path(&words))
        })
        .collect::<Vec<_>>();
    if shown.is_empty() {
        return None;
    }
    let (what, were) = if shown.len() == 1 {
        ("A remembered approval", "was")
    } else {
        ("Remembered approvals", "were")
    };
    Some(format!(
        "{what} (don't ask again for commands that start with {}) {were} not applied because \
         security level Aggressive is on: approvals and allow rules saved outside Aggressive are \
         suspended while it is active, so you are asked instead. They are kept and apply again \
         when you return to a less strict level.",
        shown.join(", ")
    ))
}

#[cfg(test)]
#[path = "exec_policy_aggressive_tests.rs"]
mod tests;
