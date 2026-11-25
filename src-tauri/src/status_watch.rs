//! Scheduled local reconciliation. Cached reads never manufacture provider evidence.
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub const CHECK_INTERVAL_MS: u64 = 5_000;
pub const STREAM_STALE_AFTER_MS: u64 = 45_000;

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Freshness {
    LocalOnly,
    Fresh,
    Stale,
    Unknown,
    Unsupported,
    Terminal,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceSource {
    LocalDemo,
    LocalCapabilityCheck,
    ProviderStream,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationSnapshot {
    pub freshness: Freshness,
    pub source: EvidenceSource,
    pub last_verified_at: Option<u64>,
    pub last_observed_at: u64,
    pub stale_after_ms: u64,
    pub reconciliation_supported: bool,
}

impl VerificationSnapshot {
    pub fn observe_stream(&mut self, previous: Option<u64>, sequence: Option<u64>, at: u64) {
        if sequence.is_some_and(|next| previous.is_none_or(|old| next > old)) {
            self.last_verified_at = Some(at);
        }
    }
    pub fn local(demo: bool, observed_at: u64) -> Self {
        Self {
            freshness: if demo {
                Freshness::LocalOnly
            } else {
                Freshness::Unsupported
            },
            source: if demo {
                EvidenceSource::LocalDemo
            } else {
                EvidenceSource::LocalCapabilityCheck
            },
            last_verified_at: None,
            last_observed_at: observed_at,
            stale_after_ms: STREAM_STALE_AFTER_MS,
            reconciliation_supported: false,
        }
    }

    /// `verified_at` is set only by an owned, validated provider event, never a snapshot read.
    pub fn stream(
        verified_at: Option<u64>,
        observed_at: u64,
        checked_at: u64,
        terminal: bool,
    ) -> Self {
        let freshness = if terminal {
            Freshness::Terminal
        } else {
            match verified_at {
                None => Freshness::Unsupported,
                Some(at) if at > checked_at => Freshness::Unknown,
                Some(at) if checked_at.saturating_sub(at) >= STREAM_STALE_AFTER_MS => {
                    Freshness::Stale
                }
                Some(_) => Freshness::Fresh,
            }
        };
        Self {
            freshness,
            source: EvidenceSource::ProviderStream,
            last_verified_at: verified_at,
            last_observed_at: observed_at,
            stale_after_ms: STREAM_STALE_AFTER_MS,
            reconciliation_supported: false,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckSchedule {
    pub checked_at: Option<u64>,
    pub next_check_at: u64,
    pub interval_ms: u64,
    pub check_sequence: u64,
}
impl Default for CheckSchedule {
    fn default() -> Self {
        Self::new(now_ms())
    }
}
impl CheckSchedule {
    pub fn new(now: u64) -> Self {
        Self {
            checked_at: None,
            next_check_at: now.saturating_add(CHECK_INTERVAL_MS),
            interval_ms: CHECK_INTERVAL_MS,
            check_sequence: 0,
        }
    }

    /// Single native worker owns the cadence. No catch-up replay after a clock jump/sleep.
    pub fn check(&mut self, now: u64) {
        self.checked_at = Some(now);
        self.next_check_at = now.saturating_add(self.interval_ms);
        self.check_sequence = self.check_sequence.saturating_add(1);
    }
}

/// One sequential local worker: no provider calls, task submission or task transitions.
pub fn start(app: tauri::AppHandle) {
    use tauri::{Emitter, Manager};
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(CHECK_INTERVAL_MS));
        let Some(state) = app.try_state::<crate::AppState>() else {
            break;
        };
        let changed = match state.hub.lock() {
            Ok(mut hub) => {
                hub.check_status(now_ms());
                true
            }
            Err(_) => false,
        };
        if changed {
            let _ = app.emit("hub-changed", ());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cadence_and_sleep_do_not_replay_checks_or_verify_cached_state() {
        let mut schedule = CheckSchedule::new(100);
        assert_eq!(schedule.next_check_at, 5_100);
        schedule.check(5_100);
        schedule.check(86_405_100);
        assert_eq!(schedule.check_sequence, 2);
        assert_eq!(schedule.next_check_at, 86_410_100);
        let local = VerificationSnapshot::local(true, 100);
        assert_eq!(local.last_verified_at, None);
        assert_eq!(local.freshness, Freshness::LocalOnly);
    }
    #[test]
    fn cached_checks_age_evidence_without_changing_its_timestamp() {
        let early = VerificationSnapshot::stream(Some(100), 100, 5_100, false);
        let late = VerificationSnapshot::stream(Some(100), 100, 45_100, false);
        assert_eq!(early.freshness, Freshness::Fresh);
        assert_eq!(late.freshness, Freshness::Stale);
        assert_eq!(early.last_verified_at, late.last_verified_at);
        assert!(!late.reconciliation_supported);
    }
    #[test]
    fn unsupported_clock_regression_and_terminal_outcomes_are_distinct() {
        assert_eq!(
            VerificationSnapshot::stream(None, 100, 100, false).freshness,
            Freshness::Unsupported
        );
        assert_eq!(
            VerificationSnapshot::stream(Some(200), 200, 100, false).freshness,
            Freshness::Unknown
        );
        assert_eq!(
            VerificationSnapshot::stream(Some(100), 100, u64::MAX, true).freshness,
            Freshness::Terminal
        );
        assert_eq!(
            VerificationSnapshot::local(false, 100).last_verified_at,
            None
        );
    }
    #[test]
    fn duplicate_unordered_or_unsequenced_callbacks_cannot_refresh_evidence() {
        let mut evidence = VerificationSnapshot::stream(None, 100, 100, false);
        evidence.observe_stream(None, Some(2), 200);
        assert_eq!(evidence.last_verified_at, Some(200));
        for sequence in [Some(2), Some(1), None] {
            evidence.observe_stream(Some(2), sequence, 50_000);
            assert_eq!(evidence.last_verified_at, Some(200));
        }
        evidence.observe_stream(Some(2), Some(3), 51_000);
        assert_eq!(evidence.last_verified_at, Some(51_000));
    }
}
