//! Explicit synthetic clipboard check; not part of ordinary test execution.
#[cfg(all(target_os = "windows", debug_assertions))]
fn main() -> std::process::ExitCode {
    if std::env::args().nth(1).as_deref() == Some("--owned-sequence") {
        match texpio_lib::product::clipboard::diagnostic_owned_sequence() {
            Ok((during, after, owned)) => println!(
                "Owned same-text sequence probe: locked={during}, closed={after}, same_owner={owned}; no input submitted."
            ),
            Err(_) => println!("REFUSED same-text sequence probe; no replay."),
        };
        return std::process::ExitCode::SUCCESS;
    }
    if std::env::args().nth(1).as_deref() == Some("--prepare-owned-text") {
        let matches = texpio_lib::product::clipboard::diagnostic_prepare_owned_text();
        if matches == Ok(true) {
            println!(
                "Exact owned disposable marker verified under clipboard lock and normalized to plain text; no input submitted."
            );
            return std::process::ExitCode::SUCCESS;
        }
        println!(
            "REFUSED disposable marker did not match or clipboard was unavailable; no replay."
        );
        return std::process::ExitCode::FAILURE;
    }
    if std::env::args().nth(1).as_deref() == Some("--inspect-formats") {
        match texpio_lib::product::clipboard::diagnostic_formats_complete() {
            Ok(complete) => println!(
                "Read-only clipboard preflight: complete_snapshot_supported={complete}; no clipboard write or input submitted."
            ),
            Err(_) => println!(
                "Read-only clipboard preflight unavailable; no clipboard write or input submitted."
            ),
        };
        return std::process::ExitCode::SUCCESS;
    }
    if std::env::args().nth(1).as_deref() != Some("--owned-clipboard") {
        eprintln!("Usage: clipboard_ownership_trial --owned-clipboard");
        return std::process::ExitCode::FAILURE;
    }
    match texpio_lib::product::clipboard::diagnostic_ownership_trial() {
        Ok(true) => {
            println!(
                "PASS stale clipboard lease preserved the newer owned copy; complete original formats restored. No input submitted."
            );
            std::process::ExitCode::SUCCESS
        }
        Ok(false) => {
            println!("Clipboard outcome unconfirmed; no replay. Diagnostic ended.");
            std::process::ExitCode::FAILURE
        }
        Err(_) => {
            println!("Clipboard check refused or unavailable; no replay. Diagnostic ended.");
            std::process::ExitCode::FAILURE
        }
    }
}
#[cfg(not(all(target_os = "windows", debug_assertions)))]
fn main() {
    eprintln!("This explicit diagnostic requires a Windows debug build.");
}
