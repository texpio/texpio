use super::fixtures::Fixture;
use super::types::{Candidate, EngineInput, PhysicalKey};
use std::collections::HashMap;

pub struct Recognizer {
    matches: HashMap<Vec<u8>, Candidate>,
    suffix: Vec<u8>,
    held: [bool; 129],
    pending: Option<Candidate>,
}

impl Recognizer {
    pub fn new(fixtures: &[Fixture]) -> Self {
        Self {
            matches: fixtures
                .iter()
                .filter(|fixture| {
                    let bytes = fixture.trigger.as_bytes();
                    (2..=32).contains(&bytes.len())
                        && bytes[0] == b';'
                        && bytes[1..].iter().all(u8::is_ascii_lowercase)
                })
                .map(|fixture| Candidate {
                    fixture_id: fixture.id.into(),
                    trigger: fixture.trigger.into(),
                })
                .map(|candidate| (candidate.trigger.as_bytes().to_vec(), candidate))
                .collect(),
            suffix: Vec::with_capacity(33),
            held: [false; 129],
            pending: None,
        }
    }

    pub fn from_rules(rules: &super::rules::RuleSet) -> Self {
        Self {
            matches: rules
                .entries()
                .iter()
                .map(|rule| {
                    (
                        rule.trigger.as_bytes().to_vec(),
                        Candidate {
                            fixture_id: rule.id.clone(),
                            trigger: rule.trigger.clone(),
                        },
                    )
                })
                .collect(),
            suffix: Vec::with_capacity(33),
            held: [false; 129],
            pending: None,
        }
    }

    pub fn push(&mut self, input: EngineInput) -> Option<Candidate> {
        let EngineInput::Key { key, down, repeat } = input else {
            self.clear();
            return None;
        };
        let index = match key {
            PhysicalKey::Ascii(byte) if byte.is_ascii() => byte as usize,
            PhysicalKey::Space => 128,
            _ => {
                self.clear();
                return None;
            }
        };
        if down {
            if repeat || self.held[index] {
                self.clear();
                return None;
            }
            self.held[index] = true;
            self.pending = None;
            match key {
                PhysicalKey::Space => {
                    self.pending = self.matches.get(&self.suffix).cloned();
                }
                PhysicalKey::Ascii(b';') => {
                    self.suffix.clear();
                    self.suffix.push(b';');
                }
                PhysicalKey::Ascii(byte)
                    if byte.is_ascii_lowercase()
                        && !self.suffix.is_empty()
                        && self.suffix.len() < 32 =>
                {
                    self.suffix.push(byte)
                }
                _ => self.suffix.clear(),
            }
            None
        } else {
            if !self.held[index] {
                self.clear();
                return None;
            }
            self.held[index] = false;
            if key != PhysicalKey::Space {
                return None;
            }
            let candidate = if self.held.iter().any(|held| *held) {
                None
            } else {
                self.pending.take()
            };
            self.clear();
            candidate
        }
    }

    pub fn clear(&mut self) {
        self.suffix.clear();
        self.held.fill(false);
        self.pending = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::fixtures::{Fixture, fixtures};
    use crate::engine::types::ResetReason;

    fn event(key: PhysicalKey, down: bool) -> EngineInput {
        EngineInput::Key {
            key,
            down,
            repeat: false,
        }
    }

    fn type_ascii(recognizer: &mut Recognizer, value: &str) {
        for byte in value.bytes() {
            assert!(
                recognizer
                    .push(event(PhysicalKey::Ascii(byte), true))
                    .is_none()
            );
            assert!(
                recognizer
                    .push(event(PhysicalKey::Ascii(byte), false))
                    .is_none()
            );
        }
    }

    fn space(recognizer: &mut Recognizer) -> Option<Candidate> {
        assert!(recognizer.push(event(PhysicalKey::Space, true)).is_none());
        recognizer.push(event(PhysicalKey::Space, false))
    }

    #[test]
    fn space_release_emits_once() {
        let mut recognizer = Recognizer::new(&fixtures());
        type_ascii(&mut recognizer, ";hello");
        assert_eq!(
            space(&mut recognizer),
            Some(Candidate {
                fixture_id: "hello".into(),
                trigger: ";hello".into()
            })
        );
        assert!(recognizer.push(event(PhysicalKey::Space, false)).is_none());
        assert!(space(&mut recognizer).is_none());
    }

    #[test]
    fn case_and_prefix_are_literal() {
        for value in [";Hello", "hello", ";hel", ":hello", ";helloo"] {
            let mut recognizer = Recognizer::new(&fixtures());
            type_ascii(&mut recognizer, value);
            assert_eq!(space(&mut recognizer), None, "{value}");
        }
    }

    #[test]
    fn reset_before_delimiter_release_cancels() {
        for reason in [
            ResetReason::Focus,
            ResetReason::Pointer,
            ResetReason::Navigation,
            ResetReason::Selection,
            ResetReason::Layout,
            ResetReason::Composition,
            ResetReason::Modifier,
            ResetReason::Injected,
            ResetReason::Pause,
            ResetReason::Stop,
            ResetReason::Fault,
            ResetReason::Overflow,
            ResetReason::Uncertainty,
        ] {
            let mut recognizer = Recognizer::new(&fixtures());
            type_ascii(&mut recognizer, ";hello");
            recognizer.push(event(PhysicalKey::Space, true));
            recognizer.push(EngineInput::Reset(reason));
            assert_eq!(recognizer.push(event(PhysicalKey::Space, false)), None);
        }
    }

    #[test]
    fn repeat_intervening_key_and_unmatched_release_cancel() {
        for interruption in [
            EngineInput::Key {
                key: PhysicalKey::Space,
                down: true,
                repeat: true,
            },
            event(PhysicalKey::Ascii(b'x'), true),
            event(PhysicalKey::Ascii(b'z'), false),
            event(PhysicalKey::Other, true),
        ] {
            let mut recognizer = Recognizer::new(&fixtures());
            type_ascii(&mut recognizer, ";hello");
            recognizer.push(event(PhysicalKey::Space, true));
            recognizer.push(interruption);
            assert_eq!(recognizer.push(event(PhysicalKey::Space, false)), None);
        }
    }

    #[test]
    fn rolled_over_trigger_release_is_allowed_before_space_release() {
        let mut recognizer = Recognizer::new(&fixtures());
        type_ascii(&mut recognizer, ";hell");
        recognizer.push(event(PhysicalKey::Ascii(b'o'), true));
        recognizer.push(event(PhysicalKey::Space, true));
        recognizer.push(event(PhysicalKey::Ascii(b'o'), false));
        assert_eq!(
            recognizer
                .push(event(PhysicalKey::Space, false))
                .unwrap()
                .fixture_id,
            "hello"
        );
    }

    #[test]
    fn held_trigger_key_prevents_dispatch() {
        let mut recognizer = Recognizer::new(&fixtures());
        type_ascii(&mut recognizer, ";hell");
        recognizer.push(event(PhysicalKey::Ascii(b'o'), true));
        assert_eq!(space(&mut recognizer), None);
    }

    #[test]
    fn bound_is_33_and_overflow_does_not_match_a_suffix() {
        let maximum = format!(";{}", "a".repeat(31));
        let items = vec![Fixture {
            id: "maximum",
            title: "Maximum",
            trigger: Box::leak(maximum.clone().into_boxed_str()),
            body: "body".into(),
        }];
        let mut recognizer = Recognizer::new(&items);
        type_ascii(&mut recognizer, &maximum);
        assert_eq!(space(&mut recognizer).unwrap().fixture_id, "maximum");
        type_ascii(&mut recognizer, &format!("{maximum}a"));
        assert_eq!(space(&mut recognizer), None);
        type_ascii(&mut recognizer, &"a".repeat(10_000));
        assert_eq!(space(&mut recognizer), None);
        type_ascii(&mut recognizer, &maximum);
        assert_eq!(space(&mut recognizer).unwrap().fixture_id, "maximum");
    }

    #[test]
    fn explicit_clear_discards_partial_input() {
        let mut recognizer = Recognizer::new(&fixtures());
        type_ascii(&mut recognizer, ";hel");
        recognizer.clear();
        type_ascii(&mut recognizer, "lo");
        assert_eq!(space(&mut recognizer), None);
    }

    #[test]
    fn user_rules_match_without_leaking_into_fixed_probe_rules() {
        use crate::engine::rules::{Rule, RuleSet};
        let rules = RuleSet::new(
            12,
            vec![Rule {
                id: "owned-id".into(),
                trigger: ";reply".into(),
                body: "My custom response 👋".into(),
            }],
        )
        .unwrap();
        let mut recognizer = Recognizer::from_rules(&rules);
        type_ascii(&mut recognizer, ";reply");
        assert_eq!(space(&mut recognizer).unwrap().fixture_id, "owned-id");
        type_ascii(&mut recognizer, ";hello");
        assert_eq!(space(&mut recognizer), None);
        let mut probe = Recognizer::new(&fixtures());
        type_ascii(&mut probe, ";reply");
        assert_eq!(space(&mut probe), None);
    }
}
