fn main() {
    #[cfg(target_os = "linux")]
    if texpio_wayland_helper::linux::serve().is_err() {
        // Never include protocol, input, target, snippet, or clipboard values.
        eprintln!("Texpio Wayland helper stopped; check service prerequisites");
        std::process::exit(1);
    }
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("The Wayland device helper requires Linux");
        std::process::exit(1);
    }
}
