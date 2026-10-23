use super::types::RefusalReason;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplicitBool {
    Known(bool),
    Unsupported,
}

pub struct GuardEvidence {
    pub password: ExplicitBool,
    pub enabled: ExplicitBool,
    pub read_only: ExplicitBool,
    pub profile_matches: bool,
    pub integrity_safe: bool,
    pub layout_supported: bool,
    pub modifiers_held: bool,
    pub composition_active: bool,
    pub collapsed_selection: bool,
    pub same_control: bool,
    pub same_caret: bool,
    pub within_deadline: bool,
    pub before: String,
    pub range_starts_document: bool,
}

pub fn validate_base(evidence: &GuardEvidence) -> Result<(), RefusalReason> {
    if !evidence.within_deadline {
        return Err(RefusalReason::ProviderTimeout);
    }
    if !evidence.profile_matches {
        return Err(RefusalReason::UnsupportedProfile);
    }
    if !evidence.integrity_safe {
        return Err(RefusalReason::Integrity);
    }
    if !evidence.layout_supported {
        return Err(RefusalReason::UnsupportedLayout);
    }
    if evidence.modifiers_held {
        return Err(RefusalReason::Modifier);
    }
    if evidence.composition_active {
        return Err(RefusalReason::Composition);
    }
    match evidence.password {
        ExplicitBool::Known(false) => {}
        ExplicitBool::Known(true) => return Err(RefusalReason::PasswordField),
        ExplicitBool::Unsupported => return Err(RefusalReason::UnsupportedPasswordSemantics),
    }
    if evidence.enabled != ExplicitBool::Known(true) {
        return Err(RefusalReason::DisabledControl);
    }
    if evidence.read_only != ExplicitBool::Known(false) {
        return Err(RefusalReason::ReadOnly);
    }
    if !evidence.collapsed_selection {
        return Err(RefusalReason::Selection);
    }
    if !evidence.same_control || !evidence.same_caret {
        return Err(RefusalReason::StaleContext);
    }
    Ok(())
}

pub fn validate_automatic(evidence: &GuardEvidence, trigger: &str) -> Result<usize, RefusalReason> {
    validate_base(evidence)?;
    let bytes = trigger.as_bytes();
    if !(2..=32).contains(&bytes.len())
        || bytes[0] != b';'
        || !bytes[1..].iter().all(u8::is_ascii_lowercase)
        || evidence.before.encode_utf16().count() > 35
    {
        return Err(RefusalReason::ContextUnavailable);
    }
    let suffix = format!("{trigger} ");
    let preceding = evidence
        .before
        .strip_suffix(&suffix)
        .ok_or(RefusalReason::SuffixMismatch)?;
    let has_boundary = if preceding.is_empty() {
        evidence.range_starts_document
    } else {
        preceding == "\r\n"
            || (preceding.chars().count() == 1 && preceding.chars().all(char::is_whitespace))
    };
    if !has_boundary {
        return Err(RefusalReason::Boundary);
    }
    Ok(suffix.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe() -> GuardEvidence {
        GuardEvidence {
            password: ExplicitBool::Known(false),
            enabled: ExplicitBool::Known(true),
            read_only: ExplicitBool::Known(false),
            profile_matches: true,
            integrity_safe: true,
            layout_supported: true,
            modifiers_held: false,
            composition_active: false,
            collapsed_selection: true,
            same_control: true,
            same_caret: true,
            within_deadline: true,
            before: ";hello ".into(),
            range_starts_document: true,
        }
    }

    #[test]
    fn exact_suffix_at_document_start_is_eligible() {
        assert_eq!(validate_automatic(&safe(), ";hello"), Ok(7));
    }

    #[test]
    fn whitespace_boundary_is_required_and_missing_context_is_not_start() {
        for (before, start, want) in [
            (" ;hello ", false, Ok(7)),
            ("\r\n;hello ", false, Ok(7)),
            ("x;hello ", false, Err(RefusalReason::Boundary)),
            (";hello ", false, Err(RefusalReason::Boundary)),
            (";Hello ", true, Err(RefusalReason::SuffixMismatch)),
            (";hello  ", true, Err(RefusalReason::SuffixMismatch)),
        ] {
            let mut evidence = safe();
            evidence.before = before.into();
            evidence.range_starts_document = start;
            assert_eq!(validate_automatic(&evidence, ";hello"), want, "{before:?}");
        }
    }

    #[test]
    fn unsupported_property_does_not_become_explicit_false() {
        let mut evidence = safe();
        evidence.password = ExplicitBool::Unsupported;
        assert_eq!(
            validate_automatic(&evidence, ";hello"),
            Err(RefusalReason::UnsupportedPasswordSemantics)
        );
        evidence.password = ExplicitBool::Known(true);
        assert_eq!(
            validate_automatic(&evidence, ";hello"),
            Err(RefusalReason::PasswordField)
        );
        evidence.password = ExplicitBool::Known(false);
        evidence.read_only = ExplicitBool::Unsupported;
        assert_eq!(
            validate_automatic(&evidence, ";hello"),
            Err(RefusalReason::ReadOnly)
        );
        evidence.read_only = ExplicitBool::Known(false);
        evidence.enabled = ExplicitBool::Unsupported;
        assert_eq!(
            validate_automatic(&evidence, ";hello"),
            Err(RefusalReason::DisabledControl)
        );
    }

    #[test]
    fn changed_control_and_caret_refuse_even_in_same_process() {
        let mut evidence = safe();
        evidence.same_control = false;
        assert_eq!(
            validate_automatic(&evidence, ";hello"),
            Err(RefusalReason::StaleContext)
        );
        evidence.same_control = true;
        evidence.same_caret = false;
        assert_eq!(
            validate_automatic(&evidence, ";hello"),
            Err(RefusalReason::StaleContext)
        );
    }

    #[test]
    fn unsafe_states_refuse_before_text_edit_preparation() {
        type RefusalCase = (fn(&mut GuardEvidence), RefusalReason);
        let setters: Vec<RefusalCase> = vec![
            (
                |e| e.profile_matches = false,
                RefusalReason::UnsupportedProfile,
            ),
            (|e| e.integrity_safe = false, RefusalReason::Integrity),
            (
                |e| e.layout_supported = false,
                RefusalReason::UnsupportedLayout,
            ),
            (|e| e.modifiers_held = true, RefusalReason::Modifier),
            (|e| e.composition_active = true, RefusalReason::Composition),
            (|e| e.collapsed_selection = false, RefusalReason::Selection),
            (
                |e| e.within_deadline = false,
                RefusalReason::ProviderTimeout,
            ),
        ];
        for (set, reason) in setters {
            let mut evidence = safe();
            set(&mut evidence);
            assert_eq!(validate_automatic(&evidence, ";hello"), Err(reason));
        }
    }

    #[test]
    fn overbroad_provider_range_and_invalid_trigger_are_refused() {
        let mut evidence = safe();
        evidence.before = format!("{};hello ", " ".repeat(80));
        assert_eq!(
            validate_automatic(&evidence, ";hello"),
            Err(RefusalReason::ContextUnavailable)
        );
        assert_eq!(
            validate_automatic(&safe(), ""),
            Err(RefusalReason::ContextUnavailable)
        );
    }
}
