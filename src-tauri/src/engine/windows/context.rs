//! Read-only UIA preparation. All COM objects stay on their creating worker.
use super::{
    TargetStamp, current_stamp,
    native::{self, ProcessIdentity},
    profiles::{self, ControlFacts, ControlKind},
    properties::{OwnedVariant, explicit_bool},
};
use crate::engine::{
    fixtures::fixtures,
    guard::{self, ExplicitBool, GuardEvidence},
    types::{Candidate, ReferenceProfile, RefusalReason},
};
use std::{
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::HWND,
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize,
            },
            Threading::GetExitCodeProcess,
        },
        UI::Accessibility::*,
    },
    core::{BSTR, Interface},
};

const PREPARE_BUDGET: Duration = Duration::from_millis(250);
const OBSERVE_BUDGET: Duration = Duration::from_secs(2);
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: Rc makes this worker-local and balances successful initialization.
        unsafe {
            CoUninitialize();
        }
    }
}

#[derive(Clone, Copy)]
struct Budget(Instant);
impl Budget {
    fn new(duration: Duration) -> Self {
        Self(Instant::now() + duration)
    }
    fn check(self) -> Result<(), RefusalReason> {
        if Instant::now() >= self.0 {
            Err(RefusalReason::ProviderTimeout)
        } else {
            Ok(())
        }
    }
    fn call<T>(self, call: impl FnOnce() -> windows::core::Result<T>) -> Result<T, RefusalReason> {
        self.check()?;
        let result = call();
        self.check()?;
        result.map_err(|_| RefusalReason::ContextUnavailable)
    }
}

pub struct WindowsContext {
    automation: IUIAutomation,
    session: u64,
    apartment: Rc<Apartment>,
}

#[derive(Clone)]
pub struct TargetToken {
    session: u64,
    stamp: TargetStamp,
    profile: ReferenceProfile,
    element: IUIAutomationElement,
    process: Rc<ProcessIdentity>,
    _apartment: Rc<Apartment>,
}

pub struct PreparedEdit {
    caret: IUIAutomationTextRange,
    anchor: IUIAutomationTextRange,
    before: String,
    starts_document: bool,
    trigger: String,
    expected: String,
    output: String,
    delete_count: usize,
    deadline: Budget,
    observation_deadline: Budget,
    // Drop ranges before the token's last apartment reference can uninitialize COM.
    target: TargetToken,
}

impl PreparedEdit {
    pub fn delete_count(&self) -> usize {
        self.delete_count
    }
    pub fn stamp(&self) -> TargetStamp {
        self.target.stamp
    }
    pub fn output(&self) -> &str {
        &self.output
    }
}

impl WindowsContext {
    pub fn recheck_target(&self, target: &TargetToken) -> Result<(), RefusalReason> {
        self.inspect(target, Budget::new(PREPARE_BUDGET))
            .map(|_| ())
    }

    pub fn new() -> Result<Self, RefusalReason> {
        // SAFETY: UIA has a dedicated MTA worker. No message-loop HWND is created here.
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(|_| RefusalReason::ContextUnavailable)?;
            let apartment = Rc::new(Apartment);
            let automation2: IUIAutomation2 =
                CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)
                    .map_err(|_| RefusalReason::ContextUnavailable)?;
            automation2
                .SetConnectionTimeout(200)
                .map_err(|_| RefusalReason::ContextUnavailable)?;
            automation2
                .SetTransactionTimeout(200)
                .map_err(|_| RefusalReason::ContextUnavailable)?;
            let automation = automation2
                .cast()
                .map_err(|_| RefusalReason::ContextUnavailable)?;
            Ok(Self {
                automation,
                session: NEXT_SESSION.fetch_add(1, Ordering::Relaxed),
                apartment,
            })
        }
    }

    pub fn capture(
        &mut self,
        profile: ReferenceProfile,
        origin: &TargetStamp,
    ) -> Result<TargetToken, RefusalReason> {
        let budget = Budget::new(PREPARE_BUDGET);
        native::check_input_state(origin)?;
        let process = Rc::new(native::process_identity(origin.process_id)?);
        budget.check()?;
        // SAFETY: COM is initialized and all values are worker-local.
        let element = budget.call(|| unsafe { self.automation.GetFocusedElement() })?;
        let token = TargetToken {
            session: self.session,
            stamp: *origin,
            profile,
            element,
            process,
            _apartment: self.apartment.clone(),
        };
        self.inspect(&token, budget)?;
        self.same_target(&token, budget)?;
        Ok(token)
    }

    pub fn prepare_automatic(
        &mut self,
        target: &TargetToken,
        candidate: &Candidate,
        body: &str,
    ) -> Result<PreparedEdit, RefusalReason> {
        let budget = Budget::new(PREPARE_BUDGET);
        if !fixtures().iter().any(|fixture| {
            fixture.id == candidate.fixture_id
                && fixture.trigger == candidate.trigger
                && fixture.body == body
        }) {
            return Err(RefusalReason::ContextUnavailable);
        }
        let (pattern, caret) = self.inspect(target, budget)?;
        let (range, before, starts_document) =
            self.adjacent(&pattern, &caret, candidate.trigger.len() + 2, budget)?;
        let evidence = safe_evidence(before.clone(), starts_document);
        let delete_count = guard::validate_automatic(&evidence, &candidate.trigger)?;
        let boundary = before
            .strip_suffix(&format!("{} ", candidate.trigger))
            .ok_or(RefusalReason::SuffixMismatch)?;
        // The left boundary is never deleted. It anchors post-insertion observation.
        let anchor = budget.call(|| unsafe { range.Clone() })?;
        budget.call(|| unsafe {
            anchor.MoveEndpointByRange(
                TextPatternRangeEndpoint_End,
                &anchor,
                TextPatternRangeEndpoint_Start,
            )
        })?;
        self.same_target(target, budget)?;
        let fresh = self.collapsed_caret(&pattern, budget)?;
        if !same_range(&caret, &fresh, budget)? {
            return Err(RefusalReason::StaleContext);
        }
        budget.check()?;
        Ok(PreparedEdit {
            target: target.clone(),
            caret,
            anchor,
            before: before.clone(),
            starts_document,
            trigger: candidate.trigger.clone(),
            expected: format!("{boundary}{body} "),
            output: format!("{body} "),
            delete_count,
            deadline: budget,
            observation_deadline: Budget::new(OBSERVE_BUDGET),
        })
    }

    pub fn revalidate(&mut self, edit: &PreparedEdit) -> Result<(), RefusalReason> {
        let budget = edit.deadline;
        let (pattern, caret) = self.inspect(&edit.target, budget)?;
        if !same_range(&edit.caret, &caret, budget)? {
            return Err(RefusalReason::StaleContext);
        }
        let (_, before, starts_document) =
            self.adjacent(&pattern, &caret, edit.trigger.len() + 2, budget)?;
        guard::validate_automatic(
            &safe_evidence(before.clone(), starts_document),
            &edit.trigger,
        )?;
        if before != edit.before || starts_document != edit.starts_document {
            return Err(RefusalReason::StaleContext);
        }
        self.same_target(&edit.target, budget)?;
        budget.check()
    }

    pub fn verify(&mut self, edit: &PreparedEdit) -> Result<bool, RefusalReason> {
        let budget = edit.observation_deadline;
        let (pattern, caret) = self.inspect(&edit.target, budget)?;
        let range = if edit.starts_document {
            let document = budget.call(|| unsafe { pattern.DocumentRange() })?;
            budget.call(|| unsafe {
                document.MoveEndpointByRange(
                    TextPatternRangeEndpoint_End,
                    &caret,
                    TextPatternRangeEndpoint_End,
                )
            })?;
            document
        } else {
            let range = budget.call(|| unsafe { edit.anchor.Clone() })?;
            // Anchor reversal is uncertainty, never an invitation to repair with backspaces.
            if budget.call(|| unsafe {
                range.CompareEndpoints(
                    TextPatternRangeEndpoint_Start,
                    &caret,
                    TextPatternRangeEndpoint_End,
                )
            })? > 0
            {
                return Err(RefusalReason::OutcomeUncertain);
            }
            budget.call(|| unsafe {
                range.MoveEndpointByRange(
                    TextPatternRangeEndpoint_End,
                    &caret,
                    TextPatternRangeEndpoint_End,
                )
            })?;
            range
        };
        let observed = bounded_text(&range, edit.expected.encode_utf16().count(), budget)?;
        if budget.call(|| unsafe {
            range.CompareEndpoints(
                TextPatternRangeEndpoint_End,
                &caret,
                TextPatternRangeEndpoint_End,
            )
        })? != 0
        {
            return Err(RefusalReason::OutcomeUncertain);
        }
        if !edit.starts_document
            && budget.call(|| unsafe {
                range.CompareEndpoints(
                    TextPatternRangeEndpoint_Start,
                    &edit.anchor,
                    TextPatternRangeEndpoint_Start,
                )
            })? != 0
        {
            return Err(RefusalReason::OutcomeUncertain);
        }
        self.same_target(&edit.target, budget)?;
        // Exact Unicode and line endings. Profile-specific EOL conversion needs qualification.
        Ok(observed == edit.expected)
    }

    fn same_target(&self, target: &TargetToken, budget: Budget) -> Result<(), RefusalReason> {
        budget.check()?;
        if target.session != self.session || current_stamp()? != target.stamp {
            return Err(RefusalReason::StaleContext);
        }
        let mut exit_code = 0;
        budget.call(|| unsafe { GetExitCodeProcess(target.process.handle.0, &mut exit_code) })?;
        if exit_code != 259 {
            return Err(RefusalReason::StaleContext);
        }
        native::check_input_state(&target.stamp)?;
        let focused = budget.call(|| unsafe { self.automation.GetFocusedElement() })?;
        if !budget
            .call(|| unsafe { self.automation.CompareElements(&target.element, &focused) })?
            .as_bool()
        {
            return Err(RefusalReason::StaleContext);
        }
        budget.check()
    }

    fn property(
        &self,
        element: &IUIAutomationElement,
        property: UIA_PROPERTY_ID,
        budget: Budget,
    ) -> Result<ExplicitBool, RefusalReason> {
        let value = OwnedVariant(
            budget.call(|| unsafe { element.GetCurrentPropertyValueEx(property, true) })?,
        );
        let unsupported = budget
            .call(|| unsafe { self.automation.CheckNotSupported(&value.0) })?
            .as_bool();
        Ok(explicit_bool(&value.0, unsupported))
    }

    fn inspect(
        &self,
        target: &TargetToken,
        budget: Budget,
    ) -> Result<(IUIAutomationTextPattern, IUIAutomationTextRange), RefusalReason> {
        self.same_target(target, budget)?;
        // Password semantics are explicit and checked BEFORE any text/range access.
        match self.property(&target.element, UIA_IsPasswordPropertyId, budget)? {
            ExplicitBool::Known(false) => {}
            ExplicitBool::Known(true) => return Err(RefusalReason::PasswordField),
            ExplicitBool::Unsupported => return Err(RefusalReason::UnsupportedPasswordSemantics),
        }
        if self.property(&target.element, UIA_IsEnabledPropertyId, budget)?
            != ExplicitBool::Known(true)
        {
            return Err(RefusalReason::DisabledControl);
        }
        self.profile(target, budget)?;
        let pattern: IUIAutomationTextPattern =
            budget.call(|| unsafe { target.element.GetCurrentPatternAs(UIA_TextPatternId) })?;
        let caret = self.collapsed_caret(&pattern, budget)?;
        let read_only = OwnedVariant(
            budget.call(|| unsafe { caret.GetAttributeValue(UIA_IsReadOnlyAttributeId) })?,
        );
        let unsupported = budget
            .call(|| unsafe { self.automation.CheckNotSupported(&read_only.0) })?
            .as_bool();
        if explicit_bool(&read_only.0, unsupported) != ExplicitBool::Known(false) {
            return Err(RefusalReason::ReadOnly);
        }
        self.same_target(target, budget)?;
        Ok((pattern, caret))
    }

    fn collapsed_caret(
        &self,
        pattern: &IUIAutomationTextPattern,
        budget: Budget,
    ) -> Result<IUIAutomationTextRange, RefusalReason> {
        let selection = budget.call(|| unsafe { pattern.GetSelection() })?;
        if budget.call(|| unsafe { selection.Length() })? != 1 {
            return Err(RefusalReason::Selection);
        }
        let caret = budget.call(|| unsafe { selection.GetElement(0) })?;
        if budget.call(|| unsafe {
            caret.CompareEndpoints(
                TextPatternRangeEndpoint_Start,
                &caret,
                TextPatternRangeEndpoint_End,
            )
        })? != 0
        {
            return Err(RefusalReason::Selection);
        }
        // An actual provider range copy, not a second COM reference to the same range.
        budget.call(|| unsafe { caret.Clone() })
    }

    fn adjacent(
        &self,
        pattern: &IUIAutomationTextPattern,
        caret: &IUIAutomationTextRange,
        characters: usize,
        budget: Budget,
    ) -> Result<(IUIAutomationTextRange, String, bool), RefusalReason> {
        if characters > 34 {
            return Err(RefusalReason::ContextUnavailable);
        }
        let range = budget.call(|| unsafe { caret.Clone() })?;
        budget.call(|| unsafe {
            range.MoveEndpointByUnit(
                TextPatternRangeEndpoint_Start,
                TextUnit_Character,
                -(characters as i32),
            )
        })?;
        // UIA may promote Character to a larger unit. Length/content, not move count,
        // determines eligibility. One extra UTF-16 unit detects truncated providers.
        let before = bounded_text(&range, 35, budget)?;
        let document = budget.call(|| unsafe { pattern.DocumentRange() })?;
        let starts_document = budget.call(|| unsafe {
            range.CompareEndpoints(
                TextPatternRangeEndpoint_Start,
                &document,
                TextPatternRangeEndpoint_Start,
            )
        })? == 0;
        Ok((range, before, starts_document))
    }

    fn profile(&self, target: &TargetToken, budget: Budget) -> Result<(), RefusalReason> {
        let element = &target.element;
        if budget.call(|| unsafe { element.CurrentProcessId() })? != target.stamp.process_id as i32
        {
            return Err(RefusalReason::UnsupportedProfile);
        }
        let name = bounded_label(budget.call(|| unsafe { element.CurrentName() })?)?;
        let framework = bounded_label(budget.call(|| unsafe { element.CurrentFrameworkId() })?)?;
        let class = bounded_label(budget.call(|| unsafe { element.CurrentClassName() })?)?;
        let automation_id =
            bounded_label(budget.call(|| unsafe { element.CurrentAutomationId() })?)?;
        let control_type = budget.call(|| unsafe { element.CurrentControlType() })?;
        let kind = if control_type == UIA_EditControlTypeId {
            ControlKind::Edit
        } else if control_type == UIA_DocumentControlTypeId {
            ControlKind::Document
        } else {
            ControlKind::Other
        };
        let caption = native::caption(&target.stamp)?;
        if !profiles::qualifies(
            target.profile,
            &ControlFacts {
                process: &target.process.executable,
                caption: &caption,
                framework: &framework,
                class: &class,
                automation_id: &automation_id,
                name: &name,
                kind,
            },
        ) {
            return Err(RefusalReason::UnsupportedProfile);
        }
        let root = budget.call(|| unsafe {
            self.automation
                .ElementFromHandle(HWND(target.stamp.foreground_window as *mut _))
        })?;
        let walker = budget.call(|| unsafe { self.automation.RawViewWalker() })?;
        let mut ancestor = element.clone();
        let mut local_document = target.profile != ReferenceProfile::ChromeFixture;
        for _ in 0..24 {
            if budget
                .call(|| unsafe { self.automation.CompareElements(&root, &ancestor) })?
                .as_bool()
            {
                return if local_document {
                    Ok(())
                } else {
                    Err(RefusalReason::UnsupportedProfile)
                };
            }
            if !local_document
                && budget.call(|| unsafe { ancestor.CurrentControlType() })?
                    == UIA_DocumentControlTypeId
            {
                // A page title is not origin evidence. Require the document's own URL.
                let value: IUIAutomationValuePattern =
                    budget.call(|| unsafe { ancestor.GetCurrentPatternAs(UIA_ValuePatternId) })?;
                let url = bounded_label(budget.call(|| unsafe { value.CurrentValue() })?)?;
                local_document = matches!(
                    url.as_str(),
                    "http://127.0.0.1:4175/" | "http://127.0.0.1:4175/index.html"
                );
            }
            ancestor = budget.call(|| unsafe { walker.GetParentElement(&ancestor) })?;
        }
        Err(RefusalReason::UnsupportedProfile)
    }
}

fn bounded_label(value: BSTR) -> Result<String, RefusalReason> {
    if value.len() > 512 {
        return Err(RefusalReason::UnsupportedProfile);
    }
    String::from_utf16(&value).map_err(|_| RefusalReason::UnsupportedProfile)
}

fn bounded_text(
    range: &IUIAutomationTextRange,
    limit: usize,
    budget: Budget,
) -> Result<String, RefusalReason> {
    let value = budget.call(|| unsafe { range.GetText((limit + 1) as i32) })?;
    if value.len() > limit {
        return Err(RefusalReason::ContextUnavailable);
    }
    String::from_utf16(&value).map_err(|_| RefusalReason::ContextUnavailable)
}

fn same_range(
    a: &IUIAutomationTextRange,
    b: &IUIAutomationTextRange,
    budget: Budget,
) -> Result<bool, RefusalReason> {
    let start = budget.call(|| unsafe {
        a.CompareEndpoints(
            TextPatternRangeEndpoint_Start,
            b,
            TextPatternRangeEndpoint_Start,
        )
    })?;
    let end = budget.call(|| unsafe {
        a.CompareEndpoints(
            TextPatternRangeEndpoint_End,
            b,
            TextPatternRangeEndpoint_End,
        )
    })?;
    Ok(start == 0 && end == 0)
}

// Successful native extraction constructs policy evidence only after all refusals.
fn safe_evidence(before: String, starts_document: bool) -> GuardEvidence {
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
        before,
        range_starts_document: starts_document,
    }
}
