//! Deterministic command-boundary tests; fake runtimes never install native hooks.
use crate::library_commands::LibraryService;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
    mpsc,
};
use std::time::Duration;

#[test]
fn start_cannot_publish_between_update_suspension_and_commit() {
    for settings_update in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let service = Arc::new(LibraryService::new(root.path().into()));
        let version = Arc::new(AtomicU32::new(1));
        let (suspended_tx, suspended_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let updating = service.clone();
        let current = version.clone();
        let update = std::thread::spawn(move || {
            updating
                .update_product(
                    || {
                        suspended_tx.send(()).unwrap();
                        release_rx.recv().unwrap();
                        Ok(Some(1))
                    },
                    || current.store(2, Ordering::SeqCst),
                    |_| assert_eq!(current.load(Ordering::SeqCst), 2),
                )
                .unwrap()
        });
        suspended_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let starting = service.clone();
        let start = std::thread::spawn(move || {
            starting
                .coordinated(|| {
                    started_tx.send(version.load(Ordering::SeqCst)).unwrap();
                })
                .unwrap()
        });
        let premature = started_rx.recv_timeout(Duration::from_millis(80)).ok();
        release_tx.send(()).unwrap();
        update.join().unwrap();
        start.join().unwrap();
        assert_eq!(
            premature,
            None,
            "old runtime published during {} update",
            if settings_update {
                "settings"
            } else {
                "content"
            }
        );
        assert_eq!(started_rx.recv_timeout(Duration::from_secs(1)).unwrap(), 2);
    }
}

#[test]
fn opposite_engine_stop_and_start_share_one_serial_boundary() {
    let root = tempfile::tempdir().unwrap();
    let service = Arc::new(LibraryService::new(root.path().into()));
    let active = Arc::new(AtomicU32::new(0));
    let (first_tx, first_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let first = service.clone();
    let state = active.clone();
    let product = std::thread::spawn(move || {
        first
            .coordinated(|| {
                state.store(0, Ordering::SeqCst); // stop opposite runtime
                first_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                state.fetch_or(1, Ordering::SeqCst); // start product
            })
            .unwrap()
    });
    first_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let (second_tx, second_rx) = mpsc::channel();
    let second = service.clone();
    let state = active.clone();
    let probe = std::thread::spawn(move || {
        second
            .coordinated(|| {
                state.store(0, Ordering::SeqCst); // stop opposite runtime
                state.fetch_or(2, Ordering::SeqCst); // start strict probe
                second_tx.send(()).unwrap();
            })
            .unwrap()
    });
    let premature = second_rx.recv_timeout(Duration::from_millis(80)).ok();
    release_tx.send(()).unwrap();
    product.join().unwrap();
    probe.join().unwrap();
    assert_eq!(
        premature, None,
        "second activation entered while first was incomplete"
    );
    assert_eq!(active.load(Ordering::SeqCst), 2);
}
