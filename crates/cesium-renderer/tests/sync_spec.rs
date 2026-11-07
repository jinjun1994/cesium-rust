use cesium_renderer::{Sync, SyncStatus};

#[test]
fn test_creates_and_default_status() {
    let sync = Sync::new();
    assert_eq!(sync.get_status(), SyncStatus::Unsignaled);
    assert!(!sync.is_destroyed());
}

#[test]
fn test_destroys() {
    let mut sync = Sync::new();
    assert!(!sync.is_destroyed());
    assert!(sync.destroy().is_ok());
    assert!(sync.is_destroyed());

    // Throws on double destroy
    assert!(sync.destroy().is_err());
}

#[test]
fn test_wait_for_signal_resolves() {
    let mut sync = Sync::new();
    let mut poll_count = 0;

    let result = sync.wait_for_signal(
        |s, _remaining| {
            poll_count += 1;
            if poll_count >= 3 {
                s.signal();
            }
            true
        },
        Some(10),
    );

    assert!(result.is_ok());
    assert_eq!(sync.get_status(), SyncStatus::Signaled);
    assert_eq!(poll_count, 3);
}

#[test]
fn test_wait_for_signal_throws_on_timeout() {
    let mut sync = Sync::new();
    let mut poll_count = 0;

    let result = sync.wait_for_signal(
        |_s, _remaining| {
            poll_count += 1;
            true // simulate never signaled
        },
        Some(5),
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.message, "Wait for signal timeout");
    assert_eq!(poll_count, 5);
}

#[test]
fn test_wait_for_signal_throws_on_custom_timeout() {
    let mut sync = Sync::new();
    let mut poll_count = 0;

    // Would signal at 6th poll, but TTL is 5
    let result = sync.wait_for_signal(
        |s, _remaining| {
            poll_count += 1;
            if poll_count >= 6 {
                s.signal();
            }
            true
        },
        Some(5),
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.message, "Wait for signal timeout");
}
