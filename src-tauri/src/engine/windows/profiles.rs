use super::super::types::ReferenceProfile;
use crate::engine::rules::TargetPolicy;

pub fn qualifies_policy(policy: TargetPolicy, facts: &ControlFacts<'_>) -> bool {
    match policy {
        TargetPolicy::Probe(profile) => qualifies(profile, facts),
        TargetPolicy::NotepadEditor => {
            facts.process.eq_ignore_ascii_case("notepad.exe")
                && facts.framework == "Win32"
                && facts.class == "RichEditD2DPT"
                && facts.name == "Text editor"
                && facts.kind == ControlKind::Document
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ControlKind {
    Edit,
    Document,
    Other,
}

pub struct ControlFacts<'a> {
    pub process: &'a str,
    pub caption: &'a str,
    pub framework: &'a str,
    pub class: &'a str,
    pub automation_id: &'a str,
    pub name: &'a str,
    pub kind: ControlKind,
}

/// Candidate profiles for disposable documents. Native qualification is separate.
/// Chrome also requires document-origin evidence in the context adapter.
pub fn qualifies(profile: ReferenceProfile, facts: &ControlFacts<'_>) -> bool {
    match profile {
        ReferenceProfile::Notepad => {
            facts.process.eq_ignore_ascii_case("notepad.exe")
                && facts.caption.trim_start_matches('*') == "texpio-p0-notepad.txt - Notepad"
                && facts.framework == "Win32"
                && facts.class == "RichEditD2DPT"
                && facts.name == "Text editor"
                && facts.kind == ControlKind::Document
        }
        ReferenceProfile::VsCode => {
            facts.process.eq_ignore_ascii_case("Code.exe")
                && matches!(
                    facts.caption,
                    "texpio-p0-vscode.txt - Visual Studio Code"
                        | "● texpio-p0-vscode.txt - Visual Studio Code"
                )
                && facts.framework == "Chrome"
                && facts.name == "texpio-p0-vscode.txt editor content"
                && facts.kind == ControlKind::Edit
        }
        ReferenceProfile::ChromeFixture => {
            facts.process.eq_ignore_ascii_case("chrome.exe")
                && facts.caption == "Texpio P0 Fixture - Google Chrome"
                && facts.framework == "Chrome"
                && facts.class == "Chrome_RenderWidgetHostHWND"
                && facts.kind == ControlKind::Edit
                && matches!(
                    (facts.automation_id, facts.name),
                    ("probe-text", "Texpio P0 textarea") | ("probe-input", "Texpio P0 input")
                )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_requires_the_local_fixture_and_identified_positive_field() {
        let mut facts = ControlFacts {
            process: "chrome.exe",
            caption: "Texpio P0 Fixture - Google Chrome",
            framework: "Chrome",
            class: "Chrome_RenderWidgetHostHWND",
            automation_id: "probe-text",
            name: "Texpio P0 textarea",
            kind: ControlKind::Edit,
        };
        assert!(qualifies(ReferenceProfile::ChromeFixture, &facts));
        facts.name = "Texpio P0 second field";
        facts.automation_id = "probe-other";
        assert!(!qualifies(ReferenceProfile::ChromeFixture, &facts));
        facts.name = "Texpio P0 textarea";
        facts.automation_id = "probe-text";
        facts.caption = "A remote page - Google Chrome";
        assert!(!qualifies(ReferenceProfile::ChromeFixture, &facts));
    }

    #[test]
    fn editor_profiles_exclude_other_documents_and_search_controls() {
        let mut facts = ControlFacts {
            process: "notepad.exe",
            caption: "texpio-p0-notepad.txt - Notepad",
            framework: "Win32",
            class: "RichEditD2DPT",
            automation_id: "",
            name: "Text editor",
            kind: ControlKind::Document,
        };
        assert!(qualifies(ReferenceProfile::Notepad, &facts));
        facts.caption = "Another document - Notepad";
        assert!(!qualifies(ReferenceProfile::Notepad, &facts));
        facts = ControlFacts {
            process: "Code.exe",
            caption: "texpio-p0-vscode.txt - Visual Studio Code",
            framework: "Chrome",
            class: "",
            automation_id: "",
            name: "texpio-p0-vscode.txt editor content",
            kind: ControlKind::Edit,
        };
        assert!(qualifies(ReferenceProfile::VsCode, &facts));
        facts.name = "Find";
        assert!(!qualifies(ReferenceProfile::VsCode, &facts));
    }

    #[test]
    fn product_notepad_accepts_documents_but_never_widens_probe_or_search() {
        use crate::engine::rules::TargetPolicy;
        let mut facts = ControlFacts {
            process: "notepad.exe",
            caption: "My document - Notepad",
            framework: "Win32",
            class: "RichEditD2DPT",
            automation_id: "",
            name: "Text editor",
            kind: ControlKind::Document,
        };
        assert!(qualifies_policy(TargetPolicy::NotepadEditor, &facts));
        assert!(!qualifies(ReferenceProfile::Notepad, &facts));
        facts.name = "Find";
        assert!(!qualifies_policy(TargetPolicy::NotepadEditor, &facts));
        facts.name = "Text editor";
        facts.process = "other.exe";
        assert!(!qualifies_policy(TargetPolicy::NotepadEditor, &facts));
    }
}
