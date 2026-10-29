use texpio_wayland_helper::attestation::*;
fn stat(pid: u32, start: &str) -> String {
    format!(
        "{pid} (helper name with ) brackets) S {} {start} 0 0",
        vec!["0"; 18].join(" ")
    )
}
fn record() -> String {
    r#"{"version":1,"workerPid":42,"sessionUid":1000,"startTime":"123456"}"#.into()
}
#[test]
fn immutable_root_metadata_is_required() {
    assert!(trusted_directory(0, 0o040755, false));
    assert!(trusted_file(0, 0o100444, false, 1));
    for (uid, mode, link) in [
        (1000, 0o040755, false),
        (0, 0o040775, false),
        (0, 0o040757, false),
        (0, 0o040755, true),
        (0, 0o100755, false),
    ] {
        assert!(!trusted_directory(uid, mode, link));
    }
    for (uid, mode, link, count) in [
        (1000, 0o100444, false, 1),
        (0, 0o100644, false, 1),
        (0, 0o100446, false, 1),
        (0, 0o100444, true, 1),
        (0, 0o040444, false, 1),
        (0, 0o100444, false, 2),
    ] {
        assert!(!trusted_file(uid, mode, link, count));
    }
}
#[test]
fn kernel_starttime_is_parsed_after_last_parenthesis() {
    assert_eq!(proc_start_time(&stat(42, "123456"), 42), Some("123456"));
    assert!(proc_start_time(&stat(41, "123456"), 42).is_none());
    assert!(proc_start_time("42 (name) S 0", 42).is_none());
    assert!(proc_start_time(&stat(42, "0003"), 42).is_none());
    assert!(proc_start_time(&"x".repeat(MAX_PROC_STAT + 1), 42).is_none());
}
#[test]
fn attestation_binds_bus_peer_to_current_kernel_process() {
    assert!(validate_record(
        record().as_bytes(),
        42,
        1000,
        &stat(42, "123456")
    ));
    assert!(!validate_record(
        record().as_bytes(),
        43,
        1000,
        &stat(43, "123456")
    ));
    assert!(!validate_record(
        record().as_bytes(),
        42,
        1001,
        &stat(42, "123456")
    ));
    assert!(!validate_record(
        record().as_bytes(),
        42,
        1000,
        &stat(42, "123457")
    ));
    for value in [
        "{}".into(),
        record().replace("\"version\":1", "\"version\":2"),
        record().replace("42", "\"42\""),
        record().replace("\"123456\"", "123456"),
        record().replace("}", ",\"extra\":1}"),
        " ".repeat(MAX_ATTESTATION + 1),
    ] {
        assert!(!validate_record(
            value.as_bytes(),
            42,
            1000,
            &stat(42, "123456")
        ));
    }
}
