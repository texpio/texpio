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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expansion {
    pub rule: ProductRule,
    pub terminator: char,
}

pub struct TypedHistory {
    rules: std::collections::HashMap<String, ProductRule>,
    lengths: Vec<usize>,
    history: std::collections::VecDeque<char>,
    capacity: usize,
    boundary: Boundary,
}
impl TypedHistory {
    pub fn new(rules: Vec<ProductRule>, boundary: Boundary) -> Self {
        let rules: std::collections::HashMap<_, _> = rules
            .into_iter()
            .filter(|rule| valid_trigger(&rule.trigger))
            .map(|rule| (rule.trigger.clone(), rule))
            .collect();
        let mut lengths: Vec<_> = rules.keys().map(|key| key.chars().count()).collect();
        lengths.sort_unstable_by(|a, b| b.cmp(a));
        lengths.dedup();
        let capacity = lengths.first().copied().unwrap_or(0) + 1;
        Self {
            rules,
            lengths,
            history: std::collections::VecDeque::with_capacity(capacity),
            capacity,
            boundary,
        }
    }
    pub fn push(&mut self, character: char) -> Option<Expansion> {
        if is_terminator(character) {
            let previous: Vec<_> = self.history.iter().copied().collect();
            for length in &self.lengths {
                let Some(start) = previous.len().checked_sub(*length) else {
                    continue;
                };
                if self.boundary == Boundary::Word && start > 0 && word(previous[start - 1]) {
                    continue;
                }
                let suffix: String = previous[start..].iter().collect();
                if let Some(rule) = self.rules.get(&suffix) {
                    let found = Expansion {
                        rule: rule.clone(),
                        terminator: character,
                    };
                    self.reset();
                    return Some(found);
                }
            }
        }
        if self.history.len() == self.capacity {
            self.history.pop_front();
        }
        self.history.push_back(character);
        None
    }
    pub fn backspace(&mut self) {
        self.history.pop_back();
    }
    pub fn reset(&mut self) {
        self.history.clear();
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
                let mut history = TypedHistory::new(vec![rule(trigger)], Boundary::Word);
                assert!(type_text(&mut history, trigger).is_empty());
                let found = history.push(end).expect("completed trigger must match");
                assert_eq!(found.rule, rule(trigger));
                assert_eq!(found.terminator, end);
            }
        }
    }
    #[test]
    fn word_boundary_refuses_embedded_matches_but_anywhere_allows_them() {
        let mut word = TypedHistory::new(vec![rule(":sig")], Boundary::Word);
        assert!(type_text(&mut word, "something:sig ").is_empty());
        assert_eq!(type_text(&mut word, " :sig ").len(), 1);
        let mut anywhere = TypedHistory::new(vec![rule(":sig")], Boundary::Anywhere);
        assert_eq!(type_text(&mut anywhere, "something:sig ").len(), 1);
    }
    #[test]
    fn longest_matching_suffix_wins() {
        let mut history = TypedHistory::new(vec![rule("sig"), rule("mysig")], Boundary::Anywhere);
        assert_eq!(type_text(&mut history, "mysig ")[0].rule.trigger, "mysig");
    }
    #[test]
    fn unicode_trigger_and_empty_replacement_are_supported() {
        let mut empty = rule(":réponse");
        empty.body = "".into();
        let mut history = TypedHistory::new(vec![empty.clone()], Boundary::Word);
        assert_eq!(type_text(&mut history, ":réponse.")[0].rule, empty);
    }
    #[test]
    fn backspace_and_reset_follow_real_editing() {
        let mut history = TypedHistory::new(vec![rule(":hello")], Boundary::Word);
        type_text(&mut history, ":hellx");
        history.backspace();
        assert_eq!(type_text(&mut history, "o ").len(), 1);
        type_text(&mut history, ":hel");
        history.reset();
        assert!(type_text(&mut history, "lo ").is_empty());
    }
    #[test]
    fn old_word_context_is_retained_when_history_is_at_capacity() {
        let mut history = TypedHistory::new(vec![rule(":sig")], Boundary::Word);
        assert!(type_text(&mut history, &format!("{}:sig ", "x".repeat(100_000))).is_empty());
        assert_eq!(type_text(&mut history, ":sig ").len(), 1);
    }
}
