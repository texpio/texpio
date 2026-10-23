use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceProfile {
    Notepad,
    VsCode,
    ChromeFixture,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum InsertionStrategy {
    #[default]
    Unicode,
    Clipboard,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ProbePhase {
    #[default]
    Stopped,
    Running,
    Paused,
    Fault,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum RefusalReason {
    UnsupportedPlatform,
    UnsupportedProfile,
    UnsupportedPasswordSemantics,
    PasswordField,
    ReadOnly,
    DisabledControl,
    Selection,
    UnsupportedLayout,
    Composition,
    Modifier,
    Integrity,
    StaleContext,
    ProviderTimeout,
    ContextUnavailable,
    Boundary,
    SuffixMismatch,
    InjectedInput,
    QueueOverflow,
    ClipboardState,
    ClipboardChanged,
    OutcomeUncertain,
    Busy,
    Stopped,
    Excluded,
    ShortcutConflict,
    PickerUnavailable,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ProbeCounters {
    pub submitted: u64,
    pub observed: u64,
    pub refused: u64,
    pub uncertain: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ProbeStatus {
    pub phase: ProbePhase,
    pub profile: Option<ReferenceProfile>,
    pub strategy: InsertionStrategy,
    pub counters: ProbeCounters,
    pub refusal_reason: Option<RefusalReason>,
    pub last_activity_age_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicalKey {
    Ascii(u8),
    Space,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetReason {
    Focus,
    Pointer,
    Navigation,
    Selection,
    Layout,
    Composition,
    Modifier,
    Injected,
    Pause,
    Stop,
    Fault,
    Overflow,
    Uncertainty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineInput {
    Key {
        key: PhysicalKey,
        down: bool,
        repeat: bool,
    },
    Reset(ResetReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub fixture_id: String,
    pub trigger: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopped_status_contract_excludes_native_input() {
        assert_eq!(
            serde_json::to_value(ProbeStatus::default()).unwrap(),
            serde_json::json!({
                "phase":"stopped", "profile":null, "strategy":"unicode",
                "counters":{"submitted":0,"observed":0,"refused":0,"uncertain":0},
                "refusalReason":null,"lastActivityAgeMs":null
            })
        );
        assert_eq!(
            serde_json::to_value(ReferenceProfile::ChromeFixture).unwrap(),
            "chromeFixture"
        );
    }
}
