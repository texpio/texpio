#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductRule {
    pub id: String,
    pub trigger: String,
    pub body: std::sync::Arc<str>,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum Boundary {
    #[default]
    Word,
    Anywhere,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum Activation {
    #[default]
    Immediate,
    Terminator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expansion {
    pub rule: ProductRule,
    /// Characters already in the target; the matching key has not been delivered.
    pub erase_count: usize,
    pub terminator: Option<char>,
}

const HISTORY_CAPACITY: usize = 512;

pub struct TypedHistory {
    rules: std::collections::HashMap<String, ProductRule>,
    lengths: Vec<usize>,
    history: std::collections::VecDeque<char>,
    boundary: Boundary,
    activation: Activation,
    left_truncated: bool,
}
impl TypedHistory {
    pub fn new(rules: Vec<ProductRule>, boundary: Boundary, activation: Activation) -> Self {
        let rules: std::collections::HashMap<_, _> = rules
            .into_iter()
            .filter(|rule| valid_trigger(&rule.trigger))
            .map(|rule| (rule.trigger.clone(), rule))
            .collect();
        let mut lengths: Vec<_> = rules.keys().map(|key| key.chars().count()).collect();
        lengths.sort_unstable_by(|a, b| b.cmp(a));
        lengths.dedup();
        Self {
            rules,
            lengths,
            history: std::collections::VecDeque::with_capacity(HISTORY_CAPACITY),
            boundary,
            activation,
            left_truncated: false,
        }
    }
    pub fn push(&mut self, character: char) -> Option<Expansion> {
        if self.activation == Activation::Terminator
            && is_terminator(character)
            && let Some(found) = self.match_suffix(0, Some(character))
        {
            return Some(found);
        }
        // Tab and Return may change logical focus without changing the native
        // window stamp. They can terminate a match, but cannot remain editable history.
        if character.is_control() {
            self.reset();
            return None;
        }
        if self.history.len() == HISTORY_CAPACITY {
            self.history.pop_front();
            self.left_truncated = true;
        }
        self.history.push_back(character);
        if self.activation == Activation::Immediate {
            self.match_suffix(-1, None)
        } else {
            None
        }
    }
    pub fn backspace(&mut self) -> Option<Expansion> {
        self.history.pop_back()?;
        if self.activation == Activation::Immediate {
            self.match_suffix(1, None)
        } else {
            None
        }
    }
    fn match_suffix(
        &mut self,
        pending_adjustment: isize,
        terminator: Option<char>,
    ) -> Option<Expansion> {
        for &length in &self.lengths {
            let Some(start) = self.history.len().checked_sub(length) else {
                continue;
            };
            if self.boundary == Boundary::Word
                && if start == 0 {
                    self.left_truncated
                } else {
                    word(self.history[start - 1])
                }
            {
                continue;
            }
            let suffix: String = self.history.iter().skip(start).collect();
            if let Some(rule) = self.rules.get(&suffix) {
                let found = Expansion {
                    rule: rule.clone(),
                    erase_count: length.saturating_add_signed(pending_adjustment),
                    terminator,
                };
                self.reset();
                return Some(found);
            }
        }
        None
    }
    pub fn reset(&mut self) {
        self.history.clear();
        self.left_truncated = false;
    }
}

pub fn valid_trigger(trigger: &str) -> bool {
    !trigger.is_empty()
        && trigger.chars().take(129).count() <= 128
        && !trigger
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}
fn word(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
pub fn is_terminator(ch: char) -> bool {
    !word(ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rule(trigger: &str) -> ProductRule {
        ProductRule {
            id: trigger.into(),
            trigger: trigger.into(),
            body: "Café 👋 नमस्ते\n".into(),
        }
    }
    fn type_text(history: &mut TypedHistory, text: &str) -> Vec<Expansion> {
        text.chars().filter_map(|ch| history.push(ch)).collect()
    }
    #[test]
    fn supports_colon_and_existing_triggers_with_each_terminator() {
        for trigger in [":hello", ";hello", "email"] {
            for end in [' ', '.', '\t', '\n'] {
                let mut history =
                    TypedHistory::new(vec![rule(trigger)], Boundary::Word, Activation::Terminator);
                assert!(type_text(&mut history, trigger).is_empty());
                let found = history.push(end).expect("completed trigger must match");
                assert_eq!(found.rule, rule(trigger));
                assert_eq!(found.terminator, Some(end));
            }
        }
    }
    #[test]
    fn word_boundary_refuses_embedded_matches_but_anywhere_allows_them() {
        let mut word =
            TypedHistory::new(vec![rule(":sig")], Boundary::Word, Activation::Terminator);
        assert!(type_text(&mut word, "something:sig ").is_empty());
        assert_eq!(type_text(&mut word, " :sig ").len(), 1);
        let mut anywhere = TypedHistory::new(
            vec![rule(":sig")],
            Boundary::Anywhere,
            Activation::Terminator,
        );
        assert_eq!(type_text(&mut anywhere, "something:sig ").len(), 1);
    }
    #[test]
    fn longest_matching_suffix_wins() {
        let mut history = TypedHistory::new(
            vec![rule("sig"), rule("mysig")],
            Boundary::Anywhere,
            Activation::Terminator,
        );
        assert_eq!(type_text(&mut history, "mysig ")[0].rule.trigger, "mysig");
    }
    #[test]
    fn unicode_trigger_and_empty_replacement_are_supported() {
        let mut empty = rule(":réponse");
        empty.body = "".into();
        let mut history =
            TypedHistory::new(vec![empty.clone()], Boundary::Word, Activation::Terminator);
        assert_eq!(type_text(&mut history, ":réponse.")[0].rule, empty);
    }
    #[test]
    fn backspace_and_reset_follow_real_editing() {
        let mut history =
            TypedHistory::new(vec![rule(":hello")], Boundary::Word, Activation::Terminator);
        type_text(&mut history, ":hellx");
        history.backspace();
        assert_eq!(type_text(&mut history, "o ").len(), 1);
        type_text(&mut history, ":hel");
        history.reset();
        assert!(type_text(&mut history, "lo ").is_empty());
    }
    #[test]
    fn completed_abbreviation_expands_without_space() {
        let mut history =
            TypedHistory::new(vec![rule(";hello")], Boundary::Word, Activation::Immediate);
        assert!(type_text(&mut history, ";hell").is_empty());
        let found = history
            .push('o')
            .expect("the final o must trigger expansion");
        assert_eq!(found.rule, rule(";hello"));
        assert_eq!(found.erase_count, 5, "the matching o is suppressed");
        assert_eq!(found.terminator, None);
        assert!(history.push(' ').is_none(), "the match is consumed once");
    }
    #[test]
    fn backspace_and_retyping_complete_an_immediate_trigger() {
        for (mistyped, backspaces) in [(";helllo", 2), (";helllo ", 3)] {
            let mut history =
                TypedHistory::new(vec![rule(";hello")], Boundary::Word, Activation::Immediate);
            assert!(type_text(&mut history, mistyped).is_empty());
            for _ in 0..backspaces {
                assert!(history.backspace().is_none());
            }
            let found = history.push('o').expect("the corrected trigger must match");
            assert_eq!(found.rule, rule(";hello"));
            assert_eq!(found.erase_count, 5);
            assert_eq!(found.terminator, None);
        }
    }
    #[test]
    fn unmatched_navigation_controls_cannot_be_backspaced_into_a_cross_field_match() {
        for activation in [Activation::Immediate, Activation::Terminator] {
            for control in ['\t', '\n', '\r', '\u{1b}'] {
                let mut history =
                    TypedHistory::new(vec![rule(";hello")], Boundary::Word, activation);
                assert!(type_text(&mut history, ";helllo").is_empty());
                assert!(history.push(control).is_none());
                for _ in 0..3 {
                    assert!(history.backspace().is_none());
                }
                assert!(
                    type_text(&mut history, "o ").is_empty(),
                    "navigation must discard the previous field's text"
                );
                assert_eq!(type_text(&mut history, ";hello ").len(), 1);
            }
        }
    }
    #[test]
    fn correcting_a_typo_preserves_the_preceding_word_boundary() {
        for activation in [Activation::Immediate, Activation::Terminator] {
            let mut history = TypedHistory::new(vec![rule(";hello")], Boundary::Word, activation);
            assert!(type_text(&mut history, "x;helllo ").is_empty());
            for _ in 0..3 {
                assert!(history.backspace().is_none());
            }
            assert!(type_text(&mut history, "o ").is_empty());
            assert!(history.backspace().is_none());
            assert!(history.push(' ').is_none());
            assert_eq!(type_text(&mut history, ";hello ").len(), 1);
        }
    }
    #[test]
    fn correction_at_truncated_history_start_does_not_invent_a_word_boundary() {
        for activation in [Activation::Immediate, Activation::Terminator] {
            let mut history = TypedHistory::new(vec![rule(";hello")], Boundary::Word, activation);
            let tail_length = HISTORY_CAPACITY - ";hello".chars().count();
            let text = format!("x;hello{}", "q".repeat(tail_length));
            assert!(type_text(&mut history, &text).is_empty());
            for _ in 0..tail_length {
                assert!(history.backspace().is_none());
            }
            // The visible trigger is still preceded by x, which was evicted.
            assert!(history.push(' ').is_none());
            assert!(history.backspace().is_none());
            for _ in ";hello".chars() {
                assert!(history.backspace().is_none());
            }
            // Empty retained history must not turn the unobserved x into a boundary.
            assert!(type_text(&mut history, ";hello ").is_empty());
            // A newly observed separator supplies a real boundary again.
            assert_eq!(type_text(&mut history, ";hello ").len(), 1);
        }
    }
    #[test]
    fn immediate_activation_uses_the_first_complete_prefix() {
        let rules = vec![rule(";he"), rule(";hello")];
        let mut immediate = TypedHistory::new(rules.clone(), Boundary::Word, Activation::Immediate);
        let matches = type_text(&mut immediate, ";hello ");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rule, rule(";he"));
        assert_eq!(matches[0].erase_count, 2);
        assert_eq!(matches[0].terminator, None);

        let mut delimited = TypedHistory::new(rules, Boundary::Word, Activation::Terminator);
        assert!(type_text(&mut delimited, ";hello").is_empty());
        let found = delimited.push(' ').expect("the longer trigger is complete");
        assert_eq!(found.rule, rule(";hello"));
        assert_eq!(found.erase_count, 6);
        assert_eq!(found.terminator, Some(' '));
    }
    #[test]
    fn deletion_counts_exclude_the_suppressed_scalar_only_in_immediate_mode() {
        for (trigger, already_typed) in [
            ("x", 0),
            ("é", 0),
            ("👋", 0),
            (";é", 1),
            (";👋", 1),
            (";é👋", 2),
        ] {
            let mut immediate =
                TypedHistory::new(vec![rule(trigger)], Boundary::Word, Activation::Immediate);
            let matches = type_text(&mut immediate, trigger);
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].erase_count, already_typed);
            assert_eq!(matches[0].terminator, None);

            let mut delimited =
                TypedHistory::new(vec![rule(trigger)], Boundary::Word, Activation::Terminator);
            assert!(type_text(&mut delimited, trigger).is_empty());
            let found = delimited
                .push(' ')
                .expect("the terminator completes the trigger");
            assert_eq!(found.erase_count, already_typed + 1);
            assert_eq!(found.terminator, Some(' '));
        }
    }
    #[test]
    fn reset_prevents_joining_trigger_fragments_in_either_activation_mode() {
        for activation in [Activation::Immediate, Activation::Terminator] {
            let mut history = TypedHistory::new(vec![rule(";hello")], Boundary::Word, activation);
            assert!(type_text(&mut history, ";hel").is_empty());
            history.reset();
            assert!(type_text(&mut history, "lo ").is_empty());
            assert_eq!(type_text(&mut history, ";hello ").len(), 1);
        }
    }
    #[test]
    fn correcting_a_typo_after_a_space_retains_the_trigger_start() {
        let mut history =
            TypedHistory::new(vec![rule(";hello")], Boundary::Word, Activation::Terminator);
        assert!(type_text(&mut history, ";helllo ").is_empty());
        for _ in 0..3 {
            history.backspace();
        }
        assert_eq!(type_text(&mut history, "o ").len(), 1);
    }
    #[test]
    fn old_word_context_is_retained_when_history_is_at_capacity() {
        let mut history =
            TypedHistory::new(vec![rule(":sig")], Boundary::Word, Activation::Terminator);
        assert!(type_text(&mut history, &format!("{}:sig ", "x".repeat(100_000))).is_empty());
        assert_eq!(type_text(&mut history, ":sig ").len(), 1);
    }
}
