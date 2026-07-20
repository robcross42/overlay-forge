use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn reset_shutdown() {
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
}

pub fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

pub fn is_shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

pub fn sleep_until_shutdown(duration: Duration) -> bool {
    let started_at = Instant::now();
    while started_at.elapsed() < duration {
        if is_shutdown_requested() {
            return true;
        }
        let remaining = duration.saturating_sub(started_at.elapsed());
        thread::sleep(remaining.min(Duration::from_millis(100)));
    }
    is_shutdown_requested()
}
