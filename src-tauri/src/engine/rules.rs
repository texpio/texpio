//! Immutable, native-owned expansion rules shared by observation and effects.

use super::types::RefusalReason;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TargetPolicy {
    Probe(super::types::ReferenceProfile),
    NotepadEditor,
}

#[derive(Clone)]
pub struct Rule {
    pub id: String,
    pub trigger: String,
    pub body: String,
}

pub struct RuleSet {
    revision: u32,
    entries: Vec<Rule>,
}

impl RuleSet {
    pub fn new(revision: u32, entries: Vec<Rule>) -> Result<Self, RefusalReason> {
        let mut ids = HashSet::new();
        let mut triggers = HashSet::new();
        if entries.len() > 10_000
            || entries.iter().any(|rule| {
                let bytes = rule.trigger.as_bytes();
                rule.id.is_empty()
                    || rule.id.len() > 120
                    || !ids.insert(&rule.id)
                    || !triggers.insert(&rule.trigger)
                    || !(2..=32).contains(&bytes.len())
                    || bytes[0] != b';'
                    || !bytes[1..].iter().all(u8::is_ascii_lowercase)
                    || rule.body.is_empty()
                    || rule.body.chars().count() > 16_384
                    || rule
                        .body
                        .chars()
                        .any(|ch| ch.is_control() && !matches!(ch, '\t' | '\n' | '\r'))
            })
        {
            return Err(RefusalReason::ContextUnavailable);
        }
        Ok(Self { revision, entries })
    }

    pub fn fixtures() -> Self {
        Self::new(
            0,
            super::fixtures::fixtures()
                .into_iter()
                .map(|item| Rule {
                    id: item.id.into(),
                    trigger: item.trigger.into(),
                    body: item.body,
                })
                .collect(),
        )
        .expect("fixed probe rule contract")
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn entries(&self) -> &[Rule] {
        &self.entries
    }
    pub fn resolve(&self, id: &str, trigger: &str) -> Option<&Rule> {
        self.entries
            .iter()
            .find(|rule| rule.id == id && rule.trigger == trigger)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, trigger: &str, body: &str) -> Rule {
        Rule {
            id: id.into(),
            trigger: trigger.into(),
            body: body.into(),
        }
    }

    #[test]
    fn owned_snapshot_resolves_only_its_exact_id_and_trigger() {
        let rules = RuleSet::new(42, vec![rule("user-id", ";reply", "Café 👋")]).unwrap();
        assert_eq!(rules.revision(), 42);
        assert_eq!(rules.resolve("user-id", ";reply").unwrap().body, "Café 👋");
        assert!(rules.resolve("other-id", ";reply").is_none());
        assert!(rules.resolve("user-id", ";hello").is_none());
    }

    #[test]
    fn invalid_or_duplicate_rules_are_rejected_before_observation() {
        for rules in [
            vec![rule("same", ";one", "a"), rule("same", ";two", "b")],
            vec![rule("one", ";same", "a"), rule("two", ";same", "b")],
            vec![rule("one", ";Upper", "body")],
            vec![rule("one", ";valid", "bad\u{8}body")],
            vec![rule("one", ";valid", "")],
            vec![rule("one", ";valid", &"x".repeat(16_385))],
        ] {
            assert!(RuleSet::new(1, rules).is_err());
        }
    }

    #[test]
    fn fixture_snapshot_is_independent_and_preserves_all_bodies() {
        let rules = RuleSet::fixtures();
        assert_eq!(rules.entries().len(), 7);
        assert_eq!(
            rules.resolve("emoji", ";emoji").unwrap().body,
            "Hello 👋 e\u{301}"
        );
        assert_eq!(rules.resolve("long", ";long").unwrap().body.len(), 2000);
    }
}
