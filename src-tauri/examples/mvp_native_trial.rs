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
    use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::GetWindowTextW};

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
            "Usage: mvp_native_trial <hello|address|accent|devanagari|emoji|multiline|long|custom>"
        );
        return std::process::ExitCode::FAILURE;
    };
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
        let origin = current_stamp()?;
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
        let target = context.capture_policy(TargetPolicy::NotepadEditor, &origin)?;
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
        Ok::<_, RefusalReason>(result)
    })();
    match result {
        Ok(OperationResult::Observed) => {
            println!("OBSERVED exact synthetic output; no hooks installed.");
            std::process::ExitCode::SUCCESS
        }
        Ok(result) => {
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
