//! Explicit, read-only qualification helper. Never installs hooks or sends input.
#[cfg(target_os = "windows")]
fn main() -> std::process::ExitCode {
    use std::{thread, time::Duration};
    use texpio_lib::engine::{
        fixtures::fixtures,
        types::{Candidate, ReferenceProfile},
        windows::{context::WindowsContext, current_stamp},
    };

    let profile = match std::env::args().nth(1).as_deref() {
        Some("notepad") => ReferenceProfile::Notepad,
        Some("vscode") => ReferenceProfile::VsCode,
        Some("chrome") => ReferenceProfile::ChromeFixture,
        _ => {
            eprintln!(
                "Usage: cargo run --locked --manifest-path src-tauri/Cargo.toml --example probe_context -- <notepad|vscode|chrome>"
            );
            return std::process::ExitCode::FAILURE;
        }
    };
    println!(
        "Read-only check: focus the disposable reference editor with a collapsed caret after ;hello and one space. Checking once in 5 seconds. No input will be sent."
    );
    thread::sleep(Duration::from_secs(5));
    let result = (|| {
        let origin = current_stamp()?;
        let mut context = WindowsContext::new()?;
        let target = context.capture(profile, &origin)?;
        let fixture = fixtures().remove(0);
        let candidate = Candidate {
            fixture_id: fixture.id.into(),
            trigger: fixture.trigger.into(),
        };
        let edit = context.prepare_automatic(&target, &candidate, &fixture.body)?;
        context.revalidate(&edit)?;
        Ok::<(), texpio_lib::engine::types::RefusalReason>(())
    })();
    match result {
        Ok(()) => {
            println!(
                "Read-only preparation accepted. This does not qualify input dispatch or output verification."
            );
            std::process::ExitCode::SUCCESS
        }
        Err(reason) => {
            println!("Read-only preparation refused: {reason:?}. No mutation occurred.");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("The read-only context probe requires Windows.");
}
