use crate::decision::Decision;
use crate::error::Error;
use crate::error::Result;
use crate::executable_name::executable_path_lookup_key;
use crate::rule::NetworkRule;
use crate::rule::NetworkRuleProtocol;
use crate::rule::PatternToken;
use crate::rule::PrefixPattern;
use crate::rule::PrefixRule;
use crate::rule::RuleMatch;
use crate::rule::RuleRef;
use crate::rule::normalize_network_rule_host;
use codex_utils_absolute_path::AbsolutePathBuf;
use multimap::MultiMap;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

type HeuristicsFallback<'a> = Option<&'a dyn Fn(&[String]) -> Decision>;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MatchOptions {
    pub resolve_host_executables: bool,
    /// D692: program names compare without regard to case (macOS and
    /// Windows; see [`PROGRAM_NAMES_FOLD_CASE`]). Only forbidden and prompt
    /// rules match a differently cased name, through
    /// [`Policy::case_folded_matches`] and
    /// [`Policy::strictest_rule_matching_more_args`]; allow rules still need
    /// the exact name, and [`Policy::matches_for_command_with_options`] stays
    /// exact.
    pub fold_program_case: bool,
}

/// D692: whether this platform finds programs without regard to case
/// (case-insensitive APFS on macOS, NTFS on Windows).
pub const PROGRAM_NAMES_FOLD_CASE: bool = cfg!(any(target_os = "macos", windows));

#[derive(Clone, Debug)]
pub struct Policy {
    rules_by_program: MultiMap<String, RuleRef>,
    network_rules: Vec<NetworkRule>,
    host_executables_by_name: HashMap<String, Arc<[AbsolutePathBuf]>>,
    /// D692: the programs of `rules_by_program` by [`fold_program_name`].
    programs_by_folded_name: HashMap<String, Vec<String>>,
}

impl Policy {
    pub fn new(rules_by_program: MultiMap<String, RuleRef>) -> Self {
        Self::from_parts(rules_by_program, Vec::new(), HashMap::new())
    }

    pub fn from_parts(
        rules_by_program: MultiMap<String, RuleRef>,
        network_rules: Vec<NetworkRule>,
        host_executables_by_name: HashMap<String, Arc<[AbsolutePathBuf]>>,
    ) -> Self {
        let mut programs_by_folded_name: HashMap<String, Vec<String>> = HashMap::new();
        for program in rules_by_program.keys() {
            programs_by_folded_name
                .entry(fold_program_name(program))
                .or_default()
                .push(program.clone());
        }
        Self {
            rules_by_program,
            network_rules,
            host_executables_by_name,
            programs_by_folded_name,
        }
    }

    pub fn empty() -> Self {
        Self::new(MultiMap::new())
    }

    pub fn rules(&self) -> &MultiMap<String, RuleRef> {
        &self.rules_by_program
    }

    pub fn network_rules(&self) -> &[NetworkRule] {
        &self.network_rules
    }

    pub fn host_executables(&self) -> &HashMap<String, Arc<[AbsolutePathBuf]>> {
        &self.host_executables_by_name
    }

    pub fn get_allowed_prefixes(&self) -> Vec<Vec<String>> {
        let mut prefixes = Vec::new();

        for (_program, rules) in self.rules_by_program.iter_all() {
            for rule in rules {
                let Some(prefix_rule) = rule.as_any().downcast_ref::<PrefixRule>() else {
                    continue;
                };
                if prefix_rule.decision != Decision::Allow {
                    continue;
                }

                let mut prefix = Vec::with_capacity(prefix_rule.pattern.rest.len() + 1);
                prefix.push(prefix_rule.pattern.first.as_ref().to_string());
                prefix.extend(prefix_rule.pattern.rest.iter().map(render_pattern_token));
                prefixes.push(prefix);
            }
        }

        prefixes.sort();
        prefixes.dedup();
        prefixes
    }

    pub fn add_prefix_rule(&mut self, prefix: &[String], decision: Decision) -> Result<()> {
        self.add_prefix_rule_with_justification(prefix, decision, /*justification*/ None)
    }

    /// [`Self::add_prefix_rule`] with the rule's `justification`.
    pub fn add_prefix_rule_with_justification(
        &mut self,
        prefix: &[String],
        decision: Decision,
        justification: Option<String>,
    ) -> Result<()> {
        let (first_token, rest) = prefix
            .split_first()
            .ok_or_else(|| Error::InvalidPattern("prefix cannot be empty".to_string()))?;

        let rule: RuleRef = Arc::new(PrefixRule {
            pattern: PrefixPattern {
                first: Arc::from(first_token.as_str()),
                rest: rest
                    .iter()
                    .map(|token| PatternToken::Single(token.clone()))
                    .collect::<Vec<_>>()
                    .into(),
            },
            decision,
            justification,
        });

        if !self.rules_by_program.contains_key(first_token) {
            self.programs_by_folded_name
                .entry(fold_program_name(first_token))
                .or_default()
                .push(first_token.clone());
        }
        self.rules_by_program.insert(first_token.clone(), rule);
        Ok(())
    }

    pub fn add_network_rule(
        &mut self,
        host: &str,
        protocol: NetworkRuleProtocol,
        decision: Decision,
        justification: Option<String>,
    ) -> Result<()> {
        let host = normalize_network_rule_host(host)?;
        if let Some(raw) = justification.as_deref()
            && raw.trim().is_empty()
        {
            return Err(Error::InvalidRule(
                "justification cannot be empty".to_string(),
            ));
        }
        self.network_rules.push(NetworkRule {
            host,
            protocol,
            decision,
            justification,
        });
        Ok(())
    }

    pub fn set_host_executable_paths(&mut self, name: String, paths: Vec<AbsolutePathBuf>) {
        self.host_executables_by_name.insert(name, paths.into());
    }

    pub fn merge_overlay(&self, overlay: &Policy) -> Policy {
        let mut combined_rules = self.rules_by_program.clone();
        for (program, rules) in overlay.rules_by_program.iter_all() {
            for rule in rules {
                combined_rules.insert(program.clone(), rule.clone());
            }
        }

        let mut combined_network_rules = self.network_rules.clone();
        combined_network_rules.extend(overlay.network_rules.iter().cloned());

        let mut host_executables_by_name = self.host_executables_by_name.clone();
        host_executables_by_name.extend(
            overlay
                .host_executables_by_name
                .iter()
                .map(|(name, paths)| (name.clone(), paths.clone())),
        );

        Policy::from_parts(
            combined_rules,
            combined_network_rules,
            host_executables_by_name,
        )
    }

    pub fn compiled_network_domains(&self) -> (Vec<String>, Vec<String>) {
        let mut allowed = Vec::new();
        let mut denied = Vec::new();

        for rule in &self.network_rules {
            match rule.decision {
                Decision::Allow => {
                    denied.retain(|entry| entry != &rule.host);
                    upsert_domain(&mut allowed, &rule.host);
                }
                Decision::Forbidden => {
                    allowed.retain(|entry| entry != &rule.host);
                    upsert_domain(&mut denied, &rule.host);
                }
                Decision::Prompt => {}
            }
        }

        (allowed, denied)
    }

    pub fn check<F>(&self, cmd: &[String], heuristics_fallback: &F) -> Evaluation
    where
        F: Fn(&[String]) -> Decision,
    {
        let matched_rules = self.matches_for_command_with_options(
            cmd,
            Some(heuristics_fallback),
            &MatchOptions::default(),
        );
        Evaluation::from_matches(matched_rules)
    }

    pub fn check_with_options<F>(
        &self,
        cmd: &[String],
        heuristics_fallback: &F,
        options: &MatchOptions,
    ) -> Evaluation
    where
        F: Fn(&[String]) -> Decision,
    {
        let matched_rules =
            self.matches_for_command_with_options(cmd, Some(heuristics_fallback), options);
        Evaluation::from_matches(matched_rules)
    }

    /// Checks multiple commands and aggregates the results.
    pub fn check_multiple<Commands, F>(
        &self,
        commands: Commands,
        heuristics_fallback: &F,
    ) -> Evaluation
    where
        Commands: IntoIterator,
        Commands::Item: AsRef<[String]>,
        F: Fn(&[String]) -> Decision,
    {
        self.check_multiple_with_options(commands, heuristics_fallback, &MatchOptions::default())
    }

    pub fn check_multiple_with_options<Commands, F>(
        &self,
        commands: Commands,
        heuristics_fallback: &F,
        options: &MatchOptions,
    ) -> Evaluation
    where
        Commands: IntoIterator,
        Commands::Item: AsRef<[String]>,
        F: Fn(&[String]) -> Decision,
    {
        let matched_rules: Vec<RuleMatch> = commands
            .into_iter()
            .flat_map(|command| {
                self.matches_for_command_with_options(
                    command.as_ref(),
                    Some(heuristics_fallback),
                    options,
                )
            })
            .collect();

        Evaluation::from_matches(matched_rules)
    }

    /// Returns matching rules for the given command. If no rules match and
    /// `heuristics_fallback` is provided, returns a single
    /// `HeuristicsRuleMatch` with the decision rendered by
    /// `heuristics_fallback`.
    ///
    /// If `heuristics_fallback.is_some()`, then the returned vector is
    /// guaranteed to be non-empty.
    pub fn matches_for_command(
        &self,
        cmd: &[String],
        heuristics_fallback: HeuristicsFallback<'_>,
    ) -> Vec<RuleMatch> {
        self.matches_for_command_with_options(cmd, heuristics_fallback, &MatchOptions::default())
    }

    pub fn matches_for_command_with_options(
        &self,
        cmd: &[String],
        heuristics_fallback: HeuristicsFallback<'_>,
        options: &MatchOptions,
    ) -> Vec<RuleMatch> {
        let matched_rules = self
            .match_exact_rules(cmd)
            .filter(|matched_rules| !matched_rules.is_empty())
            .or_else(|| {
                options
                    .resolve_host_executables
                    .then(|| self.match_host_executable_rules(cmd))
                    .filter(|matched_rules| !matched_rules.is_empty())
            })
            .unwrap_or_default();

        if matched_rules.is_empty()
            && let Some(heuristics_fallback) = heuristics_fallback
        {
            vec![RuleMatch::HeuristicsRuleMatch {
                command: cmd.to_vec(),
                decision: heuristics_fallback(cmd),
            }]
        } else {
            matched_rules
        }
    }

    /// D677 (#2455): the strictest decision of the rules that `known` does
    /// not match yet but could once more words follow it (xargs appends
    /// words read from stdin): their pattern extends past `known` and agrees
    /// with it so far. Rules `known` already matches are left to
    /// [`Self::matches_for_command_with_options`]. With no `known` words
    /// every rule counts. An absolute program also counts its basename's
    /// rules, whatever the host executable paths (stricter than matching).
    pub fn strictest_rule_matching_more_args(
        &self,
        known: &[String],
        options: &MatchOptions,
    ) -> Option<Decision> {
        let Some(first) = known.first() else {
            return self
                .rules_by_program
                .iter_all()
                .flat_map(|(_, rules)| rules)
                .map(|rule| rule_decision(rule.as_ref()))
                .max();
        };
        let basename = options
            .resolve_host_executables
            .then(|| AbsolutePathBuf::try_from(first.clone()).ok())
            .flatten()
            .and_then(|program| executable_path_lookup_key(program.as_path()));
        let rest = &known[1..];
        // D692: forbidden and prompt rules for the name in another case.
        let folded = std::iter::once(first)
            .chain(basename.as_ref())
            .filter(|_| options.fold_program_case)
            .flat_map(|program| self.programs_folding_like(program))
            .filter_map(|program| self.rules_by_program.get_vec(program))
            .flatten()
            .filter(|rule| rule_decision(rule.as_ref()) > Decision::Allow);
        std::iter::once(first)
            .chain(basename.as_ref())
            .filter_map(|program| self.rules_by_program.get_vec(program))
            .flatten()
            .chain(folded)
            .filter(|rule| {
                rule.as_any()
                    .downcast_ref::<PrefixRule>()
                    .is_none_or(|rule| {
                        let pattern = &rule.pattern.rest;
                        pattern.len() > rest.len()
                            && pattern
                                .iter()
                                .zip(rest)
                                .all(|(token, word)| token.matches(word))
                    })
            })
            .map(|rule| rule_decision(rule.as_ref()))
            .max()
    }

    /// D692: the matches of forbidden and prompt rules whose program equals
    /// the command's only up to case (see [`fold_program_name`]), when
    /// `options.fold_program_case`: `RM -rf x` for a forbidden `rm` rule,
    /// and with `resolve_host_executables` `/bin/RM` for `rm` (at a listed
    /// host path, compared up to case). Rules for the exact name are left to
    /// [`Self::matches_for_command_with_options`]; callers add these to its
    /// matches and keep its fallback, so they can only make a command
    /// stricter. Allow rules never match here.
    pub fn case_folded_matches(&self, cmd: &[String], options: &MatchOptions) -> Vec<RuleMatch> {
        let mut found = Vec::new();
        let Some(first) = cmd.first().filter(|_| options.fold_program_case) else {
            return found;
        };
        let mut add = |program: &String, resolved: Option<&AbsolutePathBuf>| {
            let Some(rules) = self.rules_by_program.get_vec(program) else {
                return;
            };
            // Only as many words as the longest pattern reads.
            let words = rules
                .iter()
                .map(|rule| {
                    rule.as_any()
                        .downcast_ref::<PrefixRule>()
                        .map_or(usize::MAX, |rule| rule.pattern.rest.len())
                })
                .max()
                .unwrap_or_default();
            let respelled = std::iter::once(program.clone())
                .chain(cmd[1..].iter().take(words).cloned())
                .collect::<Vec<_>>();
            for rule in rules {
                if rule_decision(rule.as_ref()) == Decision::Allow {
                    continue;
                }
                if let Some(rule_match) = rule.matches(&respelled) {
                    let rule_match = match resolved {
                        Some(program) => rule_match.with_resolved_program(program),
                        None => rule_match,
                    };
                    if !found.contains(&rule_match) {
                        found.push(rule_match);
                    }
                }
            }
        };
        for program in self.programs_folding_like(first) {
            add(program, None);
        }
        // A path (`/bin/RM`, `../bin/RM`), read by its basename as the
        // exact host lookup reads it. A bare name was folded above.
        let is_path = first.contains('/') || (cfg!(windows) && first.contains('\\'));
        if options.resolve_host_executables
            && is_path
            && let Ok(path) = AbsolutePathBuf::try_from(first.clone())
            && let Some(basename) = executable_path_lookup_key(path.as_path())
        {
            let folded_path = fold_program_name(&path.as_path().to_string_lossy());
            let programs = self
                .programs_by_folded_name
                .get(&fold_program_name(&basename))
                .into_iter()
                .flatten();
            for program in programs {
                let listed = self.host_executables_by_name.get(program);
                // The exact name at an exactly listed path (or with no list)
                // is matched by `matches_for_command_with_options`.
                if *program == basename && listed.is_none_or(|paths| paths.contains(&path)) {
                    continue;
                }
                if let Some(paths) = listed
                    && !paths.iter().any(|listed| {
                        fold_program_name(&listed.as_path().to_string_lossy()) == folded_path
                    })
                {
                    continue;
                }
                add(program, Some(&path));
            }
        }
        found
    }

    /// The rule programs equal to `name` up to case, other than `name`.
    fn programs_folding_like<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a String> {
        self.programs_by_folded_name
            .get(&fold_program_name(name))
            .into_iter()
            .flatten()
            .filter(move |program| program.as_str() != name)
    }

    fn match_exact_rules(&self, cmd: &[String]) -> Option<Vec<RuleMatch>> {
        let first = cmd.first()?;
        Some(
            self.rules_by_program
                .get_vec(first)
                .map(|rules| rules.iter().filter_map(|rule| rule.matches(cmd)).collect())
                .unwrap_or_default(),
        )
    }

    fn match_host_executable_rules(&self, cmd: &[String]) -> Vec<RuleMatch> {
        let Some(first) = cmd.first() else {
            return Vec::new();
        };
        let Ok(program) = AbsolutePathBuf::try_from(first.clone()) else {
            return Vec::new();
        };
        let Some(basename) = executable_path_lookup_key(program.as_path()) else {
            return Vec::new();
        };
        let Some(rules) = self.rules_by_program.get_vec(&basename) else {
            return Vec::new();
        };
        if let Some(paths) = self.host_executables_by_name.get(&basename)
            && !paths.iter().any(|path| path == &program)
        {
            return Vec::new();
        }

        let basename_command = std::iter::once(basename)
            .chain(cmd.iter().skip(1).cloned())
            .collect::<Vec<_>>();
        rules
            .iter()
            .filter_map(|rule| rule.matches(&basename_command))
            .map(|rule_match| rule_match.with_resolved_program(&program))
            .collect()
    }
}

/// D692: `name` as case-insensitive file systems compare it: Unicode
/// lowercase, plus the folds APFS applies that lowercasing does not (`ſ`,
/// `ß`, the Latin ligatures) and the dotless `ı`, which NTFS uppercases to
/// `I`. Folding more than a file system does only adds forbidden and prompt
/// matches.
pub(crate) fn fold_program_name(name: &str) -> String {
    let mut folded = String::with_capacity(name.len());
    for c in name.chars() {
        match c {
            '\u{17f}' => folded.push('s'),
            '\u{131}' => folded.push('i'),
            '\u{df}' | '\u{1e9e}' => folded.push_str("ss"),
            '\u{fb00}' => folded.push_str("ff"),
            '\u{fb01}' => folded.push_str("fi"),
            '\u{fb02}' => folded.push_str("fl"),
            '\u{fb03}' => folded.push_str("ffi"),
            '\u{fb04}' => folded.push_str("ffl"),
            '\u{fb05}' | '\u{fb06}' => folded.push_str("st"),
            c => folded.extend(c.to_lowercase()),
        }
    }
    folded
}

/// A prefix rule's decision; other kinds count as forbidding (fail closed).
fn rule_decision(rule: &dyn crate::rule::Rule) -> Decision {
    rule.as_any()
        .downcast_ref::<PrefixRule>()
        .map_or(Decision::Forbidden, |rule| rule.decision)
}

fn upsert_domain(entries: &mut Vec<String>, host: &str) {
    entries.retain(|entry| entry != host);
    entries.push(host.to_string());
}

fn render_pattern_token(token: &PatternToken) -> String {
    match token {
        PatternToken::Single(value) => value.clone(),
        PatternToken::Alts(alternatives) => format!("[{}]", alternatives.join("|")),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evaluation {
    pub decision: Decision,
    #[serde(rename = "matchedRules")]
    pub matched_rules: Vec<RuleMatch>,
}

impl Evaluation {
    pub fn is_match(&self) -> bool {
        self.matched_rules
            .iter()
            .any(|rule_match| !matches!(rule_match, RuleMatch::HeuristicsRuleMatch { .. }))
    }

    /// Caller is responsible for ensuring that `matched_rules` is non-empty.
    fn from_matches(matched_rules: Vec<RuleMatch>) -> Self {
        let decision = matched_rules.iter().map(RuleMatch::decision).max();
        #[expect(clippy::expect_used)]
        let decision = decision.expect("invariant failed: matched_rules must be non-empty");

        Self {
            decision,
            matched_rules,
        }
    }
}
