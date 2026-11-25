//! Evidence model only: this module opens no capture stream, input hook, or OS handle.
//! Task completion and visible indicator state are independent of resource-release evidence.
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CaptureState {
    NotStarted,
    Active,
    ReleaseRequested,
    Released,
    Unknown,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IndicatorState {
    NotShown,
    Visible,
    Cleared,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CleanupReason {
    Completed,
    Cancelled,
    Failed,
    Stop,
    Quit,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputerSession {
    pub capture_state: CaptureState,
    pub indicator_state: IndicatorState,
    pub cleanup_reason: Option<CleanupReason>,
    pub release_requested: bool,
    pub remaining_resources: Option<u32>,
    pub generation: u64,
    pub sequence: u64,
    pub last_confirmed_at: Option<u64>,
    pub last_cleanup_attempt_at: Option<u64>,
    pub demo: bool,
}
impl Default for ComputerSession {
    fn default() -> Self {
        Self {
            capture_state: CaptureState::NotStarted,
            indicator_state: IndicatorState::NotShown,
            cleanup_reason: None,
            release_requested: false,
            remaining_resources: Some(0),
            generation: 0,
            sequence: 0,
            last_confirmed_at: None,
            last_cleanup_attempt_at: None,
            demo: false,
        }
    }
}
impl ComputerSession {
    pub fn demo_active(&mut self, at: u64) -> Result<(), String> {
        self.demo_orphan(at)?;
        self.capture_state = CaptureState::Active;
        self.remaining_resources = Some(1); // Synthetic metadata only, never an actual handle.
        Ok(())
    }
    pub fn safe_to_forget(&self) -> bool {
        matches!(
            self.capture_state,
            CaptureState::NotStarted | CaptureState::Released
        ) && self.indicator_state != IndicatorState::Visible
            && self.remaining_resources == Some(0)
    }
    // A synthetic orphan scenario is deliberately the only session-producing operation in this slice.
    // It creates metadata, never a real capture or an OS resource.
    pub fn demo_orphan(&mut self, at: u64) -> Result<(), String> {
        if !self.safe_to_forget() {
            return Err("Resolve the existing computer-session uncertainty first.".into());
        }
        self.generation += 1;
        self.sequence = 1;
        self.capture_state = CaptureState::Unknown;
        self.indicator_state = IndicatorState::Visible;
        self.remaining_resources = None;
        self.last_confirmed_at = Some(at);
        self.last_cleanup_attempt_at = None;
        self.cleanup_reason = None;
        self.release_requested = false;
        self.demo = true;
        Ok(())
    }
    pub fn request_cleanup(&mut self, reason: CleanupReason, at: u64) {
        if self.safe_to_forget() {
            return;
        }
        self.release_requested = true;
        self.cleanup_reason = Some(reason);
        self.last_cleanup_attempt_at = Some(at);
        self.capture_state = CaptureState::ReleaseRequested;
        self.sequence += 1;
    }
    pub fn release_unverified(&mut self) {
        if !self.safe_to_forget() {
            self.capture_state = CaptureState::Unknown;
            self.sequence += 1;
        }
    }
    // Native adapter observations must identify their lease generation and strictly ordered evidence.
    // An indicator disappearing is never sufficient to set resource count to zero.
    fn observe(
        &mut self,
        generation: u64,
        sequence: u64,
        resources: Option<u32>,
        indicator: IndicatorState,
        at: u64,
    ) -> Result<(), String> {
        if generation != self.generation || sequence <= self.sequence {
            return Err("Stale computer-session evidence ignored.".into());
        }
        self.sequence = sequence;
        self.indicator_state = indicator;
        self.remaining_resources = resources;
        self.capture_state = if resources == Some(0) {
            CaptureState::Released
        } else {
            CaptureState::Unknown
        };
        self.last_confirmed_at = Some(at);
        Ok(())
    }
    pub fn demo_verify_resources(&mut self, at: u64) -> Result<(), String> {
        if !self.demo || !self.release_requested {
            return Err("Request Stop/cleanup before confirming simulated release.".into());
        }
        self.observe(
            self.generation,
            self.sequence + 1,
            Some(0),
            self.indicator_state,
            at,
        )
    }
    pub fn demo_clear_indicator(&mut self, at: u64) -> Result<(), String> {
        if !self.demo {
            return Err("No simulated computer session exists.".into());
        }
        self.observe(
            self.generation,
            self.sequence + 1,
            self.remaining_resources,
            IndicatorState::Cleared,
            at,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_marker_is_not_release_proof() {
        let mut session = ComputerSession::default();
        session.demo_orphan(1).unwrap();
        session.request_cleanup(CleanupReason::Completed, 2);
        session.release_unverified();
        assert_eq!(session.capture_state, CaptureState::Unknown);
        assert_eq!(session.indicator_state, IndicatorState::Visible);
        assert_eq!(session.last_confirmed_at, Some(1));
        assert!(!session.safe_to_forget());
    }
    #[test]
    fn hidden_indicator_does_not_prove_resources_released() {
        let mut session = ComputerSession::default();
        session.demo_orphan(1).unwrap();
        session.demo_clear_indicator(2).unwrap();
        assert_eq!(session.capture_state, CaptureState::Unknown);
        assert!(!session.safe_to_forget());
    }
    #[test]
    fn released_resources_do_not_clear_a_stale_indicator() {
        let mut session = ComputerSession::default();
        session.demo_orphan(1).unwrap();
        session.request_cleanup(CleanupReason::Stop, 2);
        session.demo_verify_resources(3).unwrap();
        assert_eq!(session.capture_state, CaptureState::Released);
        assert_eq!(session.indicator_state, IndicatorState::Visible);
        assert!(!session.safe_to_forget());
        session.demo_clear_indicator(4).unwrap();
        assert!(session.safe_to_forget());
    }
    #[test]
    fn old_generation_and_out_of_order_evidence_cannot_clear_an_orphan() {
        let mut session = ComputerSession::default();
        session.demo_orphan(1).unwrap();
        assert!(session
            .observe(0, 100, Some(0), IndicatorState::Cleared, 2)
            .is_err());
        assert!(session
            .observe(1, 1, Some(0), IndicatorState::Cleared, 2)
            .is_err());
        assert_eq!(session.capture_state, CaptureState::Unknown);
        assert!(!session.safe_to_forget());
    }
}
