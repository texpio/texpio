//! Explicit synthetic native trial, never run by the ordinary test suite.
//! Requires this task's disposable Notepad document. It installs no input hooks.
#[cfg(target_os = "windows")]
fn main() -> std::process::ExitCode {
    use std::{
        sync::Arc,
        thread,
        time::{Duration, Instant},
    };
    use texpio_lib::engine::{
        control::{DispatchGate, GateSnapshot},
        fixtures::fixtures,
        recognizer::Recognizer,
        rules::{Rule, RuleSet, TargetPolicy},
        runtime::{Backend, DispatchCounts, ObservedCandidate, OperationResult, RuntimeCore},
        types::{Candidate, EngineInput, PhysicalKey, RefusalReason},
        windows::{
            TargetStamp,
            context::{PreparedEdit, TargetToken, WindowsContext},
            current_stamp,
            unicode::{self, InjectionPlan},
        },
    };
    use windows::{
        Win32::{
            Foundation::HWND,
            System::{
                Com::{CLSCTX_INPROC_SERVER, CoCreateInstance},
                Variant::{VARIANT, VT_I4},
            },
            UI::{
                Accessibility::{
                    CUIAutomation8, IUIAutomation, IUIAutomation2, IUIAutomationElement,
                    IUIAutomationTextPattern, TextPatternRangeEndpoint_End,
                    TextPatternRangeEndpoint_Start, TreeScope_Descendants,
                    UIA_ControlTypePropertyId, UIA_DocumentControlTypeId, UIA_EditControlTypeId,
                    UIA_TabItemControlTypeId, UIA_TextPatternId,
                },
                WindowsAndMessaging::GetWindowTextW,
            },
        },
        core::Interface,
    };

    fn prepare_fixture(origin: TargetStamp, expected: &str) -> Result<TargetStamp, RefusalReason> {
        let native = (|| -> windows::core::Result<()> {
            // Explicit test-only setup. Search only the caption-checked window,
            // require one visible strict-profile editor and the exact fixed input.
            let automation2: IUIAutomation2 =
                unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)? };
            unsafe {
                automation2.SetConnectionTimeout(200)?;
                automation2.SetTransactionTimeout(200)?;
            }
            let automation: IUIAutomation = automation2.cast()?;
            let mut kind = VARIANT::default();
            unsafe {
                (*kind.Anonymous.Anonymous).vt = VT_I4;
                (*kind.Anonymous.Anonymous).Anonymous.lVal = UIA_DocumentControlTypeId.0;
            }
            let condition =
                unsafe { automation.CreatePropertyCondition(UIA_ControlTypePropertyId, &kind)? };
            let root =
                unsafe { automation.ElementFromHandle(HWND(origin.foreground_window as *mut _))? };
            let documents = unsafe { root.FindAll(TreeScope_Descendants, &condition)? };
            let count = unsafe { documents.Length()? };
            if !(1..=8).contains(&count) {
                return Err(windows::core::Error::empty());
            }
            let mut selected: Option<IUIAutomationElement> = None;
            for index in 0..count {
                let element = unsafe { documents.GetElement(index)? };
                let matches = unsafe {
                    element.CurrentProcessId()? == origin.process_id as i32
                        && element.CurrentFrameworkId()? == "Win32"
                        && element.CurrentClassName()? == "RichEditD2DPT"
                        && element.CurrentName()? == "Text editor"
                        && !element.CurrentIsOffscreen()?.as_bool()
                };
                if matches && selected.replace(element).is_some() {
                    return Err(windows::core::Error::empty());
                }
            }
            let element = selected.ok_or_else(windows::core::Error::empty)?;
            let pattern: IUIAutomationTextPattern =
                unsafe { element.GetCurrentPatternAs(UIA_TextPatternId)? };
            let range = unsafe { pattern.DocumentRange()? };
            let text = unsafe { range.GetText((expected.encode_utf16().count() + 1) as i32)? };
            if text != expected || current_stamp().ok() != Some(origin) {
                return Err(windows::core::Error::empty());
            }
            // No text is changed. The real pipeline recaptures and validates the
            // resulting focus, caret, suffix, integrity and composition afterward.
            unsafe {
                element.SetFocus()?;
                range.MoveEndpointByRange(
                    TextPatternRangeEndpoint_Start,
                    &range,
                    TextPatternRangeEndpoint_End,
                )?;
                range.Select()?;
            }
            Ok(())
        })();
        native.map_err(|_| RefusalReason::ContextUnavailable)?;
        let fresh = current_stamp()?;
        if fresh.foreground_window != origin.foreground_window
            || fresh.process_id != origin.process_id
        {
            return Err(RefusalReason::StaleContext);
        }
        println!("Fixture setup accepted exact synthetic content and collapsed its caret at end.");
        Ok(fresh)
    }

    fn report_profile_facts(origin: TargetStamp, input: &str, output: &str) {
        let result = (|| -> windows::core::Result<()> {
            // The context owns this thread's COM apartment. Read properties only;
            // never print text, control names, captions or process paths.
            let automation2: IUIAutomation2 =
                unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)? };
            unsafe {
                automation2.SetConnectionTimeout(200)?;
                automation2.SetTransactionTimeout(200)?;
            }
            let automation: IUIAutomation = automation2.cast()?;
            let element = unsafe { automation.GetFocusedElement()? };
            let process = unsafe { element.CurrentProcessId()? };
            let framework = unsafe { element.CurrentFrameworkId()? };
            let class = unsafe { element.CurrentClassName()? };
            let name = unsafe { element.CurrentName()? };
            let kind = unsafe { element.CurrentControlType()? };
            if current_stamp().ok() != Some(origin) {
                println!("Profile diagnostics: target changed; facts discarded.");
                return Ok(());
            }
            println!(
                "Profile facts: same_process={}, win32={}, richedit={}, text_editor_name={}, document={}, edit={}, tab={}",
                process == origin.process_id as i32,
                framework == "Win32",
                class == "RichEditD2DPT",
                name == "Text editor",
                kind == UIA_DocumentControlTypeId,
                kind == UIA_EditControlTypeId,
                kind == UIA_TabItemControlTypeId,
            );
            if process == origin.process_id as i32
                && framework == "Win32"
                && class == "RichEditD2DPT"
                && name == "Text editor"
                && kind == UIA_DocumentControlTypeId
            {
                let pattern: IUIAutomationTextPattern =
                    unsafe { element.GetCurrentPatternAs(UIA_TextPatternId)? };
                let document = unsafe { pattern.DocumentRange()? };
                let limit = input.encode_utf16().count() + output.encode_utf16().count() + 1;
                let text = unsafe { document.GetText(limit as i32)? };
                let observed =
                    String::from_utf16(&text).map_err(|_| windows::core::Error::empty())?;
                if current_stamp().ok() != Some(origin) {
                    println!("Fixture observation: target changed; comparison discarded.");
                    return Ok(());
                }
                println!(
                    "Fixture observation: bounded_units={}, original={}, replacement={}, semicolon_only={}, empty={}, replacement_after_semicolon={}",
                    text.len(),
                    observed == input,
                    observed == output,
                    observed == ";",
                    observed.is_empty(),
                    observed == format!(";{output}"),
                );
                println!(
                    "Fixture case comparison: ascii_case_only={}, lower_product_name={}",
                    observed.eq_ignore_ascii_case(output),
                    observed == output.replace("Texpio", "texpio"),
                );
                let first_difference = observed
                    .encode_utf16()
                    .zip(output.encode_utf16())
                    .position(|(left, right)| left != right);
                println!(
                    "Fixture comparison: first_difference={first_difference:?}, starts_semicolon={}, trailing_space={}, trailing_cr={}, trailing_lf={}, equal_trimmed={}, equal_after_first_unit={}",
                    observed.starts_with(';'),
                    observed.ends_with(' '),
                    observed.ends_with('\r'),
                    observed.ends_with('\n'),
                    observed.trim_end() == output.trim_end(),
                    observed
                        .encode_utf16()
                        .skip(1)
                        .eq(output.encode_utf16().skip(1)),
                );
            }
            Ok(())
        })();
        if result.is_err() {
            println!("Profile diagnostics unavailable; no mutation.");
        }
    }

    struct Edit {
        prepared: PreparedEdit,
        plan: InjectionPlan,
    }
    struct Trial {
        context: WindowsContext,
        target: TargetToken,
        body: String,
    }
    impl Backend for Trial {
        type Origin = TargetStamp;
        type Edit = Edit;
        fn prepare(
            &mut self,
            request: &ObservedCandidate<TargetStamp>,
        ) -> Result<Edit, RefusalReason> {
            if current_stamp()? != request.origin {
                return Err(RefusalReason::StaleContext);
            }
            let prepared =
                self.context
                    .prepare_automatic(&self.target, &request.candidate, &self.body)?;
            let plan = unicode::unicode_plan(&prepared)?;
            Ok(Edit { prepared, plan })
        }
        fn revalidate(&mut self, edit: &Edit) -> Result<(), RefusalReason> {
            self.context.revalidate(&edit.prepared)
        }
        fn dispatch(&mut self, edit: &Edit) -> Result<DispatchCounts, RefusalReason> {
            let counts = unicode::dispatch(&edit.prepared, &edit.plan)?;
            println!(
                "Native submission: expected={}, submitted={}",
                counts.expected, counts.submitted
            );
            Ok(counts)
        }
        fn verify(&mut self, edit: &Edit, _: GateSnapshot) -> Result<bool, RefusalReason> {
            let deadline = Instant::now() + Duration::from_millis(1500);
            while Instant::now() < deadline {
                let observed = self.context.verify(&edit.prepared).inspect_err(|reason| {
                    println!("Native verification refused: {reason:?}");
                })?;
                if observed {
                    return Ok(true);
                }
                thread::sleep(Duration::from_millis(10));
            }
            Ok(false)
        }
    }

    let Some(case) = std::env::args().nth(1) else {
        eprintln!(
            "Usage: mvp_native_trial <hello|address|accent|devanagari|emoji|multiline|long|custom> [--inspect-only|--prepare-fixture]"
        );
        return std::process::ExitCode::FAILURE;
    };
    let mode = std::env::args().nth(2);
    if !matches!(
        mode.as_deref(),
        None | Some("--inspect-only" | "--prepare-fixture")
    ) {
        eprintln!("Optional mode must be --inspect-only or --prepare-fixture.");
        return std::process::ExitCode::FAILURE;
    }
    let inspect_only = mode.as_deref() == Some("--inspect-only");
    let selected = fixtures()
        .into_iter()
        .find(|fixture| fixture.id == case)
        .map(|fixture| Rule {
            id: fixture.id.into(),
            trigger: fixture.trigger.into(),
            body: fixture.body,
        })
        .or_else(|| {
            (case == "custom").then(|| Rule {
                id: "owned-custom".into(),
                trigger: ";reply".into(),
                body: "A custom Texpio snippet — Café 👋".into(),
            })
        });
    let Some(rule) = selected else {
        return std::process::ExitCode::FAILURE;
    };
    let expected_input = format!("{} ", rule.trigger);
    let expected_output = format!("{} ", rule.body);
    let rules = Arc::new(RuleSet::new(1, vec![rule.clone()]).expect("synthetic rule contract"));
    let mut recognizer = Recognizer::from_rules(&rules);
    for byte in rule.trigger.bytes() {
        for down in [true, false] {
            recognizer.push(EngineInput::Key {
                key: PhysicalKey::Ascii(byte),
                down,
                repeat: false,
            });
        }
    }
    recognizer.push(EngineInput::Key {
        key: PhysicalKey::Space,
        down: true,
        repeat: false,
    });
    let candidate: Candidate = recognizer
        .push(EngineInput::Key {
            key: PhysicalKey::Space,
            down: false,
            repeat: false,
        })
        .expect("synthetic recognition");
    println!(
        "Synthetic native trial: {case}; checking only texpio-mvp-test.txt in five seconds. Not physical-input evidence."
    );
    thread::sleep(Duration::from_secs(5));
    let result = (|| {
        let mut origin = current_stamp()?;
        println!("Stage: foreground stamp captured.");
        let mut caption = [0u16; 256];
        // SAFETY: read-only check with a caller-owned bounded caption buffer.
        let count =
            unsafe { GetWindowTextW(HWND(origin.foreground_window as *mut _), &mut caption) };
        if count <= 0
            || String::from_utf16_lossy(&caption[..count as usize]).trim_start_matches('*')
                != "texpio-mvp-test.txt - Notepad"
        {
            println!("Stage: disposable-document caption did not match; no target text read.");
            return Err(RefusalReason::UnsupportedProfile);
        }
        println!("Stage: disposable-document caption accepted.");
        let mut context = WindowsContext::with_rules(rules)?;
        println!("Stage: context initialized.");
        if inspect_only {
            report_profile_facts(origin, &expected_input, &expected_output);
            return Ok(None);
        }
        if mode.as_deref() == Some("--prepare-fixture") {
            origin = prepare_fixture(origin, &expected_input)?;
        }
        let target = context
            .capture_policy(TargetPolicy::NotepadEditor, &origin)
            .inspect_err(|_| report_profile_facts(origin, &expected_input, &expected_output))?;
        println!("Stage: native control accepted.");
        let gate = Arc::new(DispatchGate::new());
        let ticket = gate.arm()?;
        let mut core = RuntimeCore::new(
            gate.clone(),
            Trial {
                context,
                target,
                body: rule.body,
            },
        );
        let request = ObservedCandidate {
            candidate,
            origin,
            ticket,
            published_at: Instant::now(),
        };
        let result = core.process(&request);
        gate.close();
        if matches!(result, OperationResult::Uncertain { .. }) {
            report_profile_facts(origin, &expected_input, &expected_output);
        }
        Ok::<_, RefusalReason>(Some(result))
    })();
    match result {
        Ok(None) => {
            println!("Read-only profile inspection finished; no input submitted.");
            std::process::ExitCode::SUCCESS
        }
        Ok(Some(OperationResult::Observed)) => {
            println!("OBSERVED exact synthetic output; no hooks installed.");
            std::process::ExitCode::SUCCESS
        }
        Ok(Some(result)) => {
            println!("Synthetic result: {result:?}. No repair or retry attempted.");
            std::process::ExitCode::FAILURE
        }
        Err(reason) => {
            println!("Synthetic preflight refused: {reason:?}; no mutation.");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("The synthetic native trial requires Windows.");
}
