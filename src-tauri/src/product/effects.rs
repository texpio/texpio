//! Ordered effects with explicit failure boundaries; native implementations never retry text.
use super::{
    matcher::Expansion,
    settings::{InsertionMode, ProductSettings},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ProductError {
    Cancelled,
    TargetChanged,
    ModifiersHeld,
    InputBlocked,
    Clipboard,
    Busy,
    InvalidSettings,
    Unavailable,
    AttachmentMissing,
    ShortcutUnavailable,
}

pub trait EffectBackend {
    fn check(&mut self) -> Result<(), ProductError>;
    fn prepare(&mut self, _text: &str, _mode: InsertionMode) -> Result<(), ProductError> {
        Ok(())
    }
    fn erase(&mut self, characters: usize) -> Result<(), ProductError>;
    fn content(&mut self, text: &str, mode: InsertionMode) -> Result<(), ProductError>;
    fn before_content(&mut self) -> Result<(), ProductError> {
        Ok(())
    }
    fn after_content(&mut self) -> Result<(), ProductError> {
        Ok(())
    }
    fn terminator(&mut self, character: char) -> Result<(), ProductError>;
}

pub fn perform(
    backend: &mut impl EffectBackend,
    expansion: &Expansion,
    settings: &ProductSettings,
) -> Result<(), ProductError> {
    backend.check()?;
    backend.prepare(&expansion.rule.body, settings.insertion)?;
    backend.erase(expansion.rule.trigger.chars().count())?;
    backend.check()?;
    backend.before_content()?;
    if !expansion.rule.body.is_empty() {
        backend.content(&expansion.rule.body, settings.insertion)?;
    }
    backend.check()?;
    backend.after_content()?;
    backend.check()?;
    if settings.preserve_terminator {
        backend.terminator(expansion.terminator)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::matcher::ProductRule;
    #[derive(Default)]
    struct Receiver {
        calls: Vec<String>,
        fail_at: Option<&'static str>,
        checks: usize,
        change_after: Option<usize>,
    }
    impl EffectBackend for Receiver {
        fn check(&mut self) -> Result<(), ProductError> {
            self.checks += 1;
            if self.change_after == Some(self.checks) {
                Err(ProductError::TargetChanged)
            } else {
                Ok(())
            }
        }
        fn erase(&mut self, count: usize) -> Result<(), ProductError> {
            self.calls.push(format!("erase:{count}"));
            if self.fail_at == Some("erase") {
                Err(ProductError::InputBlocked)
            } else {
                Ok(())
            }
        }
        fn content(&mut self, text: &str, _: InsertionMode) -> Result<(), ProductError> {
            self.calls.push(format!("body:{text}"));
            if self.fail_at == Some("content") {
                Err(ProductError::Cancelled)
            } else {
                Ok(())
            }
        }
        fn terminator(&mut self, ch: char) -> Result<(), ProductError> {
            self.calls.push(format!("end:{ch}"));
            Ok(())
        }
    }
    fn expansion() -> Expansion {
        Expansion {
            rule: ProductRule {
                id: "hello".into(),
                trigger: ":hello".into(),
                body: "Café 👋".into(),
            },
            terminator: ' ',
        }
    }
    #[test]
    fn deletion_content_and_terminator_follow_selected_order() {
        let mut receiver = Receiver::default();
        perform(&mut receiver, &expansion(), &ProductSettings::default()).unwrap();
        assert_eq!(receiver.calls, ["erase:6", "body:Café 👋", "end: "]);
    }
    #[test]
    fn failed_partial_deletion_is_never_retried_or_followed_by_text() {
        let mut receiver = Receiver {
            fail_at: Some("erase"),
            ..Default::default()
        };
        assert_eq!(
            perform(&mut receiver, &expansion(), &ProductSettings::default()),
            Err(ProductError::InputBlocked)
        );
        assert_eq!(receiver.calls, ["erase:6"]);
    }
    #[test]
    fn cancelled_body_never_forwards_a_terminator() {
        let mut receiver = Receiver {
            fail_at: Some("content"),
            ..Default::default()
        };
        assert_eq!(
            perform(&mut receiver, &expansion(), &ProductSettings::default()),
            Err(ProductError::Cancelled)
        );
        assert_eq!(receiver.calls, ["erase:6", "body:Café 👋"]);
    }
    #[test]
    fn changed_target_is_checked_before_and_after_deletion() {
        for (change_after, expected) in [(1, Vec::<String>::new()), (2, vec!["erase:6".into()])] {
            let mut receiver = Receiver {
                change_after: Some(change_after),
                ..Default::default()
            };
            assert_eq!(
                perform(&mut receiver, &expansion(), &ProductSettings::default()),
                Err(ProductError::TargetChanged)
            );
            assert_eq!(receiver.calls, expected);
        }
    }
    #[test]
    fn empty_body_still_deletes_and_can_omit_terminator() {
        let mut expansion = expansion();
        expansion.rule.body = "".into();
        let settings = ProductSettings {
            preserve_terminator: false,
            ..Default::default()
        };
        let mut receiver = Receiver::default();
        perform(&mut receiver, &expansion, &settings).unwrap();
        assert_eq!(receiver.calls, ["erase:6"]);
    }
}
