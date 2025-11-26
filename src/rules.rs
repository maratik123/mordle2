use crate::grapheme_rule::GraphemeRule;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::mem;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Rules<'s> {
    pub rules: HashMap<&'s str, HashSet<GraphemeRule>>,
}

#[derive(Debug, Error)]
pub enum RulesError {
    #[error("Invalid rule {rule:?} at {grapheme}")]
    InvalidRule {
        grapheme: String,
        rule: GraphemeRule,
    },
}

impl<'s> Rules<'s> {
    pub fn add_rule(&mut self, grapheme: &'s str, rule: GraphemeRule) -> Result<(), RulesError> {
        let rules = match self.rules.entry(grapheme) {
            Entry::Vacant(entry) => {
                entry.insert(HashSet::from([rule]));
                return Ok(());
            }
            Entry::Occupied(entry) if entry.get().contains(&rule) => {
                return Ok(());
            }
            Entry::Occupied(entry) => entry.into_mut(),
        };
        if rule == GraphemeRule::NotInWord || rules.contains(&GraphemeRule::NotInWord) {
            return Err(RulesError::InvalidRule {
                grapheme: grapheme.to_string(),
                rule,
            });
        }
        let mut rule_to_add = HashSet::new();
        {
            let rule_to_del = HashSet::<GraphemeRule>::new();
            for rule in rules.iter() {
                match rule {
                    GraphemeRule::ExactPlace { .. } => {}
                    GraphemeRule::ExactCount { .. } => {}
                    GraphemeRule::CountFrom { .. } => {}
                    GraphemeRule::NotInWord => {}
                }
            }
            rules.retain(|rule| !rule_to_del.contains(rule));
        }
        if rule_to_add.len() > rules.len() {
            mem::swap(&mut rule_to_add, rules);
        }
        rules.extend(rule_to_add);
        Ok(())
    }
}
