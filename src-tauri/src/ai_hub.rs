//! Local request registry. No provider transport, secret, billing, or account access exists here.
pub use overlay_forge_chatgpt_connection::{ChatMessage, ChatRole};
use crate::computer_session::{CleanupReason, ComputerSession};
use crate::status_watch::{EvidenceSource, VerificationSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AiProvider {
    Chatgpt,
    Codex,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PendingSide {
    User,
    Assistant,
    Neither,
    Unknown,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Queued,
    Connecting,
    InProgress,
    Thinking,
    Tool,
    Streaming,
    Approval,
    Login,
    Input,
    Resuming,
    Stopping,
    Complete,
    Cancelled,
    Failed,
    Unknown,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RequestAction {
    Advance,
    Approve,
    Deny,
    Stop,
    AcknowledgeStop,
    LoseConnection,
    Reconcile,
    Fail,
    DemoOrphan,
    DemoActive,
    DemoVerifyResources,
    DemoClearIndicator,
    Cleanup,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BlockerKind {
    AccessApproval,
    SignIn,
    RequiredInput,
    AdapterUnavailable,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestRecord {
    pub id: u64,
    pub conversation_id: String,
    pub provider: AiProvider,
    pub pending_side: PendingSide,
    pub stage: Stage,
    pub blocker_kind: Option<BlockerKind>,
    pub requested_action: Option<String>,
    pub event_source: &'static str,
    pub sequence: u64,
    pub state_since: u64,
    pub last_confirmed_at: u64,
    pub terminal_outcome: Option<Stage>,
    pub demo: bool,
    pub message: String,
    pub response: String,
    pub detail: Option<String>,
    pub computer: ComputerSession,
    pub verification: VerificationSnapshot,
    #[serde(skip)]
    provider_sequence: Option<u64>,
    #[serde(skip)]
    chatgpt_registration: Option<String>,
    #[serde(skip)]
    saved_locally: bool,
    previous_stage: Stage,
    resume_target: Stage,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
impl Stage {
    fn terminal(self) -> bool {
        matches!(self, Self::Complete | Self::Cancelled | Self::Failed)
    }
    fn pending(self) -> PendingSide {
        match self {
            Self::Approval | Self::Login | Self::Input => PendingSide::User,
            Self::Complete | Self::Cancelled | Self::Failed => PendingSide::Neither,
            Self::Unknown => PendingSide::Unknown,
            _ => PendingSide::Assistant,
        }
    }
}
impl RequestRecord {
    fn new(id: u64, provider: AiProvider, mode: RequestMode<'_>, message: String) -> Self {
        let demo = mode == RequestMode::Demo;
        let created_at = now();
        let (conversation_id, chatgpt_registration) = match mode {
            RequestMode::LiveChatgpt {
                conversation_id,
                registration,
            } => (conversation_id.to_owned(), Some(registration.to_owned())),
            _ => (format!("{provider:?}-{id}"), None),
        };
        let mut request = Self {
            id,
            conversation_id,
            provider,
            message,
            demo,
            chatgpt_registration,
            pending_side: PendingSide::Neither,
            stage: Stage::Queued,
            blocker_kind: None,
            requested_action: None,
            event_source: match mode {
                RequestMode::Demo => "localDemo",
                RequestMode::Unavailable => "localCapabilityCheck",
                RequestMode::LiveChatgpt { .. } => "providerStream",
            },
            sequence: 0,
            state_since: created_at,
            last_confirmed_at: created_at,
            terminal_outcome: None,
            response: String::new(),
            detail: None,
            computer: ComputerSession::default(),
            verification: if matches!(mode, RequestMode::LiveChatgpt { .. }) {
                VerificationSnapshot::stream(None, created_at, created_at, false)
            } else {
                VerificationSnapshot::local(demo, created_at)
            },
            provider_sequence: None,
            saved_locally: false,
            previous_stage: Stage::Queued,
            resume_target: Stage::InProgress,
        };
        request.set_stage(match mode {
            RequestMode::Demo => Stage::Queued,
            RequestMode::Unavailable => Stage::Failed,
            RequestMode::LiveChatgpt { .. } => Stage::Connecting,
        });
        request
    }

    fn is_live(&self) -> bool {
        matches!(self.verification.source, EvidenceSource::ProviderStream)
    }

    fn set_stage(&mut self, stage: Stage) {
        self.stage = stage;
        self.pending_side = stage.pending();
        self.blocker_kind = match stage {
            Stage::Approval => Some(BlockerKind::AccessApproval),
            Stage::Login => Some(BlockerKind::SignIn),
            Stage::Input => Some(BlockerKind::RequiredInput),
            _ => None,
        };
        self.requested_action = match stage {
            Stage::Approval => Some(
                "Approve the local demonstration step, or deny it. No external action occurs."
                    .into(),
            ),
            Stage::Login => Some(
                "Demo only: simulate completed sign-in. No browser or authorization opens.".into(),
            ),
            Stage::Input => Some("Demo only: acknowledge the required-input step.".into()),
            _ => None,
        };
        self.sequence += 1;
        self.state_since = now();
        // Losing a connection is not a confirmed provider update.
        if stage != Stage::Unknown {
            self.last_confirmed_at = self.state_since;
            if !self.is_live() {
                self.verification = VerificationSnapshot::local(self.demo, self.state_since);
            }
        }
        self.terminal_outcome = stage.terminal().then_some(stage);
        let reason = match stage {
            Stage::Complete => Some(CleanupReason::Completed),
            Stage::Cancelled => Some(CleanupReason::Cancelled),
            Stage::Failed => Some(CleanupReason::Failed),
            Stage::Stopping => Some(CleanupReason::Stop),
            _ => None,
        };
        if let Some(reason) = reason {
            self.cleanup_computer(reason);
        }
    }
    fn cleanup_computer(&mut self, reason: CleanupReason) {
        self.computer.request_cleanup(reason, now());
        // No capture adapter exists: a local cleanup request is not proof of release.
        self.computer.release_unverified();
    }
    fn transition(&mut self, action: RequestAction, sequence: u64) -> Result<(), String> {
        if !self.demo {
            return Err(
                "This adapter is unavailable; start a new attempt after setup is implemented."
                    .into(),
            );
        }
        if self.sequence != sequence {
            return Err("State changed. Review the latest request before acting.".into());
        }
        // Cleanup remains available after task completion; never regress its terminal outcome.
        match action {
            RequestAction::DemoActive => {
                if self.stage.terminal() {
                    return Err(
                        "A finished task cannot start even a simulated active session.".into(),
                    );
                }
                self.computer.demo_active(now())?;
            }
            RequestAction::DemoOrphan => {
                self.computer.demo_orphan(now())?;
                match self.stage {
                    Stage::Complete => self.cleanup_computer(CleanupReason::Completed),
                    Stage::Cancelled => self.cleanup_computer(CleanupReason::Cancelled),
                    Stage::Failed => self.cleanup_computer(CleanupReason::Failed),
                    _ => (),
                }
            }
            RequestAction::DemoVerifyResources => self.computer.demo_verify_resources(now())?,
            RequestAction::DemoClearIndicator => self.computer.demo_clear_indicator(now())?,
            RequestAction::Cleanup => self.cleanup_computer(CleanupReason::Stop),
            _ => return self.transition_task(action),
        }
        self.sequence += 1;
        Ok(())
    }
    fn transition_task(&mut self, action: RequestAction) -> Result<(), String> {
        if self.stage.terminal() {
            return Err("This request is finished. A retry must create a new request.".into());
        }
        let next = match (self.stage, action) {
            (_, RequestAction::Stop) => Stage::Stopping,
            (_, RequestAction::Fail) => Stage::Failed,
            (Stage::Unknown, RequestAction::Reconcile) => self.previous_stage,
            (Stage::Unknown, _) => {
                return Err(
                    "Reconcile the demo's authoritative state before taking another action.".into(),
                )
            }
            (_, RequestAction::LoseConnection) => {
                self.previous_stage = self.stage;
                Stage::Unknown
            }
            (Stage::Stopping, RequestAction::AcknowledgeStop) => Stage::Cancelled,
            (Stage::Stopping, _) => {
                return Err("Stop requested; cancellation is not yet acknowledged.".into())
            }
            (Stage::Approval, RequestAction::Approve) => Stage::Resuming,
            (Stage::Approval, RequestAction::Deny) => Stage::Cancelled,
            (Stage::Queued, RequestAction::Advance) => Stage::Connecting,
            (Stage::Connecting, RequestAction::Advance) => Stage::Login,
            (Stage::Login, RequestAction::Advance) => Stage::Resuming,
            (Stage::Resuming, RequestAction::Advance) => self.resume_target,
            (Stage::InProgress, RequestAction::Advance) => Stage::Thinking,
            (Stage::Thinking, RequestAction::Advance) => Stage::Input,
            (Stage::Tool, RequestAction::Advance) => Stage::Approval,
            (Stage::Streaming, RequestAction::Advance) => Stage::Complete,
            (Stage::Input, RequestAction::Advance) => Stage::Resuming,
            _ => return Err("This action does not apply to the current request state.".into()),
        };
        // Continuation is domain state, never inferred from reply text.
        match (self.stage, action) {
            (Stage::Login, RequestAction::Advance) => self.resume_target = Stage::InProgress,
            (Stage::Input, RequestAction::Advance) => self.resume_target = Stage::Tool,
            (Stage::Approval, RequestAction::Approve) => self.resume_target = Stage::Streaming,
            _ => (),
        }
        if action == RequestAction::Approve {
            self.response = "Approval recorded in local demo.".into();
        }
        if action == RequestAction::Fail {
            self.response =
                "Local demo task failed. Computer-session cleanup is tracked separately below."
                    .into();
        }
        if next == Stage::Streaming {
            self.response = "Local demo reply. No model was called, no account was accessed, and no charges were incurred.".into();
        }
        self.set_stage(next);
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RequestMode<'a> {
    Demo,
    Unavailable,
    LiveChatgpt {
        conversation_id: &'a str,
        registration: &'a str,
    },
}

pub struct PendingChat {
    pub request_id: u64,
    pub conversation_id: String,
    pub input: Vec<ChatMessage>,
}

#[derive(Clone, Serialize)]
pub struct ChatConversation {
    pub id: String,
    pub title: String,
}

#[derive(Deserialize, Serialize)]
pub struct SavedChatConversation {
    pub id: String,
    registration: String,
    turns: Vec<SavedChatTurn>,
}

#[derive(Deserialize, Serialize)]
struct SavedChatTurn {
    id: u64,
    prompt: String,
    reply: String,
    completed_at: u64,
    last_verified_at: Option<u64>,
}

pub struct CompletedReply {
    pub text: String,
    pub saved_locally: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubSnapshot {
    pub chatgpt: Option<crate::chatgpt::ConnectionSnapshot>,
    pub status_checks: crate::status_watch::CheckSchedule,
    pub revision: u64,
    pub requests: Vec<RequestRecord>,
    pub chatgpt_draft: String,
    pub chatgpt_conversations: Vec<ChatConversation>,
    pub codex_draft: String,
    pub startup_messages: Vec<String>,
}
#[derive(Default)]
pub struct AiHub {
    revision: u64,
    status_checks: crate::status_watch::CheckSchedule,
    next_id: u64,
    requests: Vec<RequestRecord>,
    drafts: [String; 2],
    startup_messages: Vec<String>,
}
impl AiHub {
    pub fn can_switch_profile(&self) -> Result<(), String> {
        if self.requests.iter().any(|r| r.terminal_outcome.is_none() || !r.computer.safe_to_forget()) {
            return Err("Finish or cancel requests and resolve computer-session cleanup before changing Google profiles.".into());
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn snapshot(&self) -> HubSnapshot {
        self.snapshot_for(None, None)
    }

    pub fn snapshot_for(&self, registration: Option<&str>, selection: Option<&str>) -> HubSnapshot {
        let conversations = registration.map_or_else(Vec::new, |id| self.chatgpt_conversations(id));
        let selected = selection.or_else(|| conversations.first().map(|chat| chat.id.as_str()));
        let recent_start = self.requests.len().saturating_sub(50);
        let mut requests: Vec<_> = self
            .requests
            .iter()
            .enumerate()
            .filter_map(|(index, request)| {
                if request.is_live() && request.chatgpt_registration.as_deref() != registration {
                    return None;
                }
                (!request.saved_locally
                    || index >= recent_start
                    || Some(request.conversation_id.as_str()) == selected)
                    .then(|| request.clone())
            })
            .collect();
        let checked_at = now();
        for request in requests.iter_mut().filter(|request| request.is_live()) {
            request.verification = VerificationSnapshot::stream(
                request.verification.last_verified_at,
                request.last_confirmed_at,
                checked_at,
                request.stage == Stage::Complete,
            );
        }
        HubSnapshot {
            chatgpt: None,
            status_checks: self.status_checks.clone(),
            revision: self.revision,
            requests,
            chatgpt_draft: self.drafts[0].clone(),
            chatgpt_conversations: conversations,
            codex_draft: self.drafts[1].clone(),
            startup_messages: self.startup_messages.clone(),
        }
    }

    pub fn restore_chatgpt(&mut self, conversations: Vec<SavedChatConversation>) {
        for conversation in conversations {
            for turn in conversation.turns {
                let mut request = RequestRecord::new(
                    turn.id,
                    AiProvider::Chatgpt,
                    RequestMode::LiveChatgpt {
                        conversation_id: &conversation.id,
                        registration: &conversation.registration,
                    },
                    turn.prompt,
                );
                request.set_stage(Stage::Complete);
                request.state_since = turn.completed_at;
                request.last_confirmed_at = turn.completed_at;
                request.verification.last_verified_at = turn.last_verified_at;
                request.response = turn.reply;
                request.saved_locally = true;
                self.next_id = self.next_id.max(turn.id);
                self.requests.push(request);
            }
        }
        self.requests.sort_unstable_by_key(|turn| turn.id);
        self.revision += 1;
    }

    pub fn completed_chatgpt_transcript(
        &self,
        id: u64,
        reply: &str,
    ) -> Option<SavedChatConversation> {
        let current = self.requests.iter().find(|turn| turn.id == id)?;
        let turns = self
            .requests
            .iter()
            .filter(|turn| {
                turn.conversation_id == current.conversation_id
                    && (turn.stage == Stage::Complete || turn.id == id)
            })
            .map(|turn| SavedChatTurn {
                id: turn.id,
                prompt: turn.message.clone(),
                reply: if turn.id == id {
                    reply.to_owned()
                } else {
                    turn.response.clone()
                },
                completed_at: if turn.id == id {
                    now()
                } else {
                    turn.state_since
                },
                last_verified_at: turn.verification.last_verified_at,
            })
            .collect();
        Some(SavedChatConversation {
            id: current.conversation_id.clone(),
            registration: current.chatgpt_registration.clone()?,
            turns,
        })
    }
    pub fn check_status(&mut self, now: u64) {
        // Inspect owned metadata only. Request sequence and evidence timestamps stay intact.
        self.status_checks.check(now);
        self.revision += 1;
    }
    pub fn startup_message(&mut self, message: String) {
        self.startup_messages.push(message);
        self.revision += 1;
    }
    pub fn touch(&mut self) {
        self.revision += 1;
    }
    pub fn draft(&mut self, provider: AiProvider, text: String) -> Result<(), String> {
        if text.len() > 16_000 {
            return Err("Draft limit is 16,000 UTF-8 bytes.".into());
        }
        self.drafts[if provider == AiProvider::Chatgpt {
            0
        } else {
            1
        }] = text;
        self.revision += 1;
        Ok(())
    }
    pub fn submit(&mut self, provider: AiProvider, demo: bool) -> Result<(), String> {
        let mode = if demo {
            RequestMode::Demo
        } else {
            RequestMode::Unavailable
        };
        self.create_request(provider, mode).map(|_| ())
    }

    fn chatgpt_conversations(&self, registration: &str) -> Vec<ChatConversation> {
        let mut by_id = HashMap::new();
        for request in self
            .requests
            .iter()
            .filter(|request| request.chatgpt_registration.as_deref() == Some(registration))
        {
            let (_, latest_request) = by_id
                .entry(request.conversation_id.as_str())
                .or_insert((request.message.as_str(), request.id));
            *latest_request = request.id;
        }
        let mut conversations: Vec<_> = by_id.into_iter().collect();
        conversations.sort_unstable_by_key(|(_, (_, latest))| std::cmp::Reverse(*latest));
        conversations
            .into_iter()
            .map(|(id, (first_prompt, _))| ChatConversation {
                id: id.to_owned(),
                title: first_prompt.chars().take(60).collect(),
            })
            .collect()
    }

    pub fn start_chatgpt(
        &mut self,
        expected_draft: &str,
        selected_conversation: Option<&str>,
        registration: &str,
    ) -> Result<PendingChat, String> {
        let prompt = self.drafts[0].trim();
        if prompt != expected_draft.trim() {
            return Err(
                "The draft changed before it could be sent. Review it and try again.".into(),
            );
        }
        let mut input = Vec::new();
        let conversation_id = if let Some(id) = selected_conversation {
            let mut found = false;
            for turn in self
                .requests
                .iter()
                .filter(|turn| turn.conversation_id == id)
            {
                if turn.chatgpt_registration.as_deref() != Some(registration) {
                    return Err(
                        "This conversation is unavailable for the connected ChatGPT account."
                            .into(),
                    );
                }
                found = true;
                if turn.stage == Stage::Complete {
                    input.push(ChatMessage {
                        role: ChatRole::User,
                        content: turn.message.clone(),
                    });
                    if !turn.response.is_empty() {
                        input.push(ChatMessage {
                            role: ChatRole::Assistant,
                            content: turn.response.clone(),
                        });
                    }
                }
            }
            if !found {
                return Err(
                    "This conversation is no longer available. Choose New chat to send your draft."
                        .into(),
                );
            }
            id.to_owned()
        } else {
            format!("Chatgpt-{}", uuid::Uuid::new_v4())
        };
        input.push(ChatMessage {
            role: ChatRole::User,
            content: prompt.to_owned(),
        });
        let request_id = self.create_request(
            AiProvider::Chatgpt,
            RequestMode::LiveChatgpt {
                conversation_id: &conversation_id,
                registration,
            },
        )?;
        Ok(PendingChat {
            request_id,
            conversation_id,
            input,
        })
    }

    fn create_request(
        &mut self,
        provider: AiProvider,
        mode: RequestMode<'_>,
    ) -> Result<u64, String> {
        let index = if provider == AiProvider::Chatgpt {
            0
        } else {
            1
        };
        let message = self.drafts[index].trim().to_string();
        if message.is_empty() {
            return Err("Enter a prompt first.".into());
        }
        let mut request = RequestRecord::new(self.next_id + 1, provider, mode, message);
        self.make_room(&request.conversation_id)?;
        self.next_id = request.id;
        if mode == RequestMode::Unavailable {
            request.blocker_kind = Some(BlockerKind::AdapterUnavailable);
            request.response = "Provider adapter is not implemented or verified. No request was transmitted. OAuth setup and a verified funding path are required; no paid fallback is enabled.".into();
        } else {
            self.drafts[index].clear();
        }
        self.requests.push(request);
        self.revision += 1;
        Ok(self.next_id)
    }

    fn make_room(&mut self, selected_conversation: &str) -> Result<(), String> {
        if self
            .requests
            .iter()
            .filter(|request| !request.saved_locally)
            .count()
            < 50
        {
            return Ok(());
        }
        // Saved turns stay in the archive. Keep any unsaved completed context together.
        let oldest = self.requests.iter().find(|request| {
            !request.saved_locally
                && request.conversation_id != selected_conversation
                && self
                    .requests
                    .iter()
                    .filter(|turn| turn.conversation_id == request.conversation_id)
                    .all(|turn| turn.stage.terminal() && turn.computer.safe_to_forget())
        });
        let Some(oldest) = oldest else {
            return Err("The local 50-request limit was reached. Finish active work or choose New chat to continue.".into());
        };
        let id = oldest.conversation_id.clone();
        self.requests
            .retain(|request| request.saved_locally || request.conversation_id != id);
        Ok(())
    }

    pub fn update_chatgpt(&mut self, id: u64, text: &str, sequence: Option<u64>) {
        let Some(request) = self.requests.iter_mut().find(|request| request.id == id) else {
            return;
        };
        if request.stage.terminal() || request.stage == Stage::Stopping {
            return;
        }
        if request.stage != Stage::Streaming {
            request.set_stage(Stage::Streaming);
        }
        let observed_at = now();
        request
            .verification
            .observe_stream(request.provider_sequence, sequence, observed_at);
        request.provider_sequence = sequence;
        request.last_confirmed_at = observed_at;
        request.response.clear();
        request.response.push_str(text);
        request.sequence += 1;
        self.revision += 1;
    }

    pub fn finish_chatgpt(
        &mut self,
        id: u64,
        outcome: Result<CompletedReply, String>,
        stopped: bool,
    ) {
        let Some(request) = self.requests.iter_mut().find(|request| request.id == id) else {
            return;
        };
        let mut saved_conversation = None;
        let stage = match outcome {
            Ok(reply) => {
                request.response = reply.text;
                if reply.saved_locally {
                    saved_conversation = Some(request.conversation_id.clone());
                } else {
                    request.detail = Some("Reply completed, but saving locally failed. It is available in this session.".into());
                }
                Stage::Complete
            }
            Err(detail) => {
                request.detail = Some(detail);
                if stopped || request.stage == Stage::Stopping {
                    Stage::Cancelled
                } else {
                    Stage::Failed
                }
            }
        };
        request.set_stage(stage);
        if let Some(conversation_id) = saved_conversation {
            for turn in self.requests.iter_mut().filter(|turn| {
                turn.conversation_id == conversation_id && turn.stage == Stage::Complete
            }) {
                turn.saved_locally = true;
                turn.detail = None;
            }
        }
        self.revision += 1;
    }
    pub fn act(&mut self, id: u64, sequence: u64, action: RequestAction) -> Result<(), String> {
        self.requests
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or("Request not found.")?
            .transition(action, sequence)?;
        self.revision += 1;
        Ok(())
    }
    pub fn stop_all(&mut self) {
        for request in &mut self.requests {
            if !request.stage.terminal() {
                request.set_stage(Stage::Stopping);
            } else {
                request.cleanup_computer(CleanupReason::Stop);
                request.sequence += 1;
            }
        }
        self.revision += 1;
    }
    pub fn prepare_shutdown(&mut self) -> Result<(), String> {
        // WindowManager waits for the live worker to exit before calling this method.
        for request in &mut self.requests {
            if !request.stage.terminal() {
                request.set_stage(Stage::Cancelled);
            }
            request.cleanup_computer(CleanupReason::Quit);
        }
        self.revision += 1;
        if self.requests.iter().any(|r| !r.computer.safe_to_forget()) {
            return Err("Quit blocked: computer-session release or indicator cleanup is unverified. Inspect Activity & Approvals; task completion is not capture-release proof.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn demo() -> AiHub {
        let mut h = AiHub::default();
        h.draft(AiProvider::Codex, "Test".into()).unwrap();
        h.submit(AiProvider::Codex, true).unwrap();
        h
    }
    fn step(h: &mut AiHub, a: RequestAction) {
        let r = &h.requests[0];
        h.act(r.id, r.sequence, a).unwrap();
    }
    #[test]
    fn unavailable_never_sends_and_preserves_draft() {
        let mut h = demo();
        h.draft(AiProvider::Chatgpt, "Private".into()).unwrap();
        h.submit(AiProvider::Chatgpt, false).unwrap();
        assert_eq!(h.requests[1].pending_side, PendingSide::Neither);
        assert_eq!(h.requests[1].stage, Stage::Failed);
        assert_eq!(h.drafts[0], "Private");
    }
    #[test]
    fn approval_requires_verified_continuation() {
        let mut h = demo();
        for _ in 0..9 {
            step(&mut h, RequestAction::Advance);
        }
        assert_eq!(h.requests[0].stage, Stage::Approval);
        step(&mut h, RequestAction::Approve);
        assert_eq!(h.requests[0].stage, Stage::Resuming);
        step(&mut h, RequestAction::Advance);
        assert_eq!(h.requests[0].stage, Stage::Streaming);
        step(&mut h, RequestAction::Advance);
        assert_eq!(h.requests[0].pending_side, PendingSide::Neither);
        assert!(h
            .act(1, h.requests[0].sequence, RequestAction::LoseConnection)
            .is_err());
    }
    #[test]
    fn cancellation_and_uncertainty_are_not_completion() {
        let mut h = demo();
        step(&mut h, RequestAction::Stop);
        step(&mut h, RequestAction::LoseConnection);
        assert_eq!(h.requests[0].pending_side, PendingSide::Unknown);
        assert!(h
            .act(1, h.requests[0].sequence, RequestAction::AcknowledgeStop)
            .is_err());
        step(&mut h, RequestAction::Reconcile);
        assert_eq!(h.requests[0].stage, Stage::Stopping);
        step(&mut h, RequestAction::AcknowledgeStop);
        assert_eq!(h.requests[0].terminal_outcome, Some(Stage::Cancelled));
    }
    #[test]
    fn stale_actions_and_concurrent_ownership() {
        let mut h = demo();
        let seq = h.requests[0].sequence;
        step(&mut h, RequestAction::Advance);
        assert!(h.act(1, seq, RequestAction::Stop).is_err());
        h.draft(AiProvider::Chatgpt, "Second".into()).unwrap();
        h.submit(AiProvider::Chatgpt, true).unwrap();
        assert_ne!(h.requests[0].conversation_id, h.requests[1].conversation_id);
        assert_eq!(h.requests[1].stage, Stage::Queued);
        assert!(h.drafts[1].is_empty());
    }
    #[test]
    fn blockers_are_owned_and_denial_is_terminal() {
        let mut h = demo();
        for _ in 0..2 {
            step(&mut h, RequestAction::Advance);
        }
        assert_eq!(h.requests[0].blocker_kind, Some(BlockerKind::SignIn));
        assert_eq!(h.requests[0].pending_side, PendingSide::User);
        for _ in 0..4 {
            step(&mut h, RequestAction::Advance);
        }
        assert_eq!(h.requests[0].blocker_kind, Some(BlockerKind::RequiredInput));
        for _ in 0..3 {
            step(&mut h, RequestAction::Advance);
        }
        step(&mut h, RequestAction::Deny);
        assert_eq!(h.requests[0].terminal_outcome, Some(Stage::Cancelled));
        assert!(h
            .act(1, h.requests[0].sequence, RequestAction::Approve)
            .is_err());
    }
    #[test]
    fn active_limit_never_evicts_owned_work_and_retry_has_new_identity() {
        let mut h = AiHub::default();
        for _ in 0..50 {
            h.draft(AiProvider::Codex, "Demo".into()).unwrap();
            h.submit(AiProvider::Codex, true).unwrap();
        }
        h.draft(AiProvider::Codex, "Retry".into()).unwrap();
        assert!(h.submit(AiProvider::Codex, true).is_err());
        assert_eq!(h.requests[0].id, 1);
        step(&mut h, RequestAction::Stop);
        step(&mut h, RequestAction::AcknowledgeStop);
        h.submit(AiProvider::Codex, true).unwrap();
        assert_eq!(h.requests.len(), 50);
        assert_eq!(h.requests.last().unwrap().id, 51);
        assert!(!h.requests.iter().any(|r| r.id == 1));
    }
    #[test]
    fn oversized_drafts_leave_other_provider_and_previous_value_intact() {
        let mut h = AiHub::default();
        h.draft(AiProvider::Codex, "Keep Codex".into()).unwrap();
        h.draft(AiProvider::Chatgpt, "Keep ChatGPT".into()).unwrap();
        assert!(h.draft(AiProvider::Codex, "x".repeat(16_001)).is_err());
        assert_eq!(h.drafts, ["Keep ChatGPT", "Keep Codex"]);
    }
    #[test]
    fn terminal_outcomes_request_cleanup_without_claiming_release() {
        use crate::computer_session::{CaptureState, IndicatorState};
        for (stage, reason) in [
            (Stage::Complete, CleanupReason::Completed),
            (Stage::Cancelled, CleanupReason::Cancelled),
            (Stage::Failed, CleanupReason::Failed),
        ] {
            let mut h = demo();
            step(&mut h, RequestAction::DemoActive);
            h.requests[0].set_stage(stage);
            let request = &h.requests[0];
            assert_eq!(request.pending_side, PendingSide::Neither);
            assert_eq!(request.terminal_outcome, Some(stage));
            assert_eq!(request.computer.capture_state, CaptureState::Unknown);
            assert_eq!(request.computer.cleanup_reason, Some(reason));
            assert_eq!(request.computer.indicator_state, IndicatorState::Visible);
            let sequence = request.sequence;
            h.act(1, sequence, RequestAction::Cleanup).unwrap();
            assert_eq!(h.requests[0].terminal_outcome, Some(stage));
        }
    }
    #[test]
    fn unresolved_orphans_prevent_eviction_and_quit() {
        let mut h = demo();
        step(&mut h, RequestAction::DemoOrphan);
        step(&mut h, RequestAction::Fail);
        for _ in 0..49 {
            h.draft(AiProvider::Codex, "Other work".into()).unwrap();
            h.submit(AiProvider::Codex, true).unwrap();
        }
        h.draft(AiProvider::Codex, "New work".into()).unwrap();
        assert!(h.submit(AiProvider::Codex, true).is_err());
        assert_eq!(h.requests[0].id, 1);
        assert!(h.prepare_shutdown().is_err());
        assert_eq!(
            h.requests[0].computer.cleanup_reason,
            Some(CleanupReason::Quit)
        );
        step(&mut h, RequestAction::DemoVerifyResources);
        assert!(!h.requests[0].computer.safe_to_forget());
        step(&mut h, RequestAction::DemoClearIndicator);
        assert!(h.prepare_shutdown().is_ok());
    }
    #[test]
    fn global_stop_cleans_unknown_and_finished_requests() {
        use crate::computer_session::CaptureState;
        let mut h = demo();
        step(&mut h, RequestAction::DemoActive);
        step(&mut h, RequestAction::LoseConnection);
        h.stop_all();
        assert_eq!(h.requests[0].stage, Stage::Stopping);
        assert!(h.requests[0].computer.release_requested);
        assert_eq!(h.requests[0].computer.capture_state, CaptureState::Unknown);
        step(&mut h, RequestAction::AcknowledgeStop);
        h.stop_all();
        assert_eq!(h.requests[0].terminal_outcome, Some(Stage::Cancelled));
    }
    #[test]
    fn scheduled_check_does_not_advance_work_or_invalidate_actions() {
        let mut h = demo();
        let before = h.snapshot();
        h.check_status(5_000);
        h.check_status(86_405_000);
        let after = h.snapshot();
        assert_eq!(after.status_checks.check_sequence, 2);
        assert_eq!(after.status_checks.next_check_at, 86_410_000);
        assert_eq!(before.requests[0].sequence, after.requests[0].sequence);
        assert_eq!(
            before.requests[0].last_confirmed_at,
            after.requests[0].last_confirmed_at
        );
        assert_eq!(after.requests[0].verification.last_verified_at, None);
        assert_eq!(before.requests[0].stage, after.requests[0].stage);
        h.act(1, before.requests[0].sequence, RequestAction::Advance)
            .unwrap();
    }
    #[test]
    fn checks_preserve_terminal_and_computer_ownership() {
        let mut h = demo();
        step(&mut h, RequestAction::DemoActive);
        step(&mut h, RequestAction::Fail);
        let before = h.snapshot();
        h.check_status(u64::MAX - 5000);
        let after = h.snapshot();
        assert_eq!(after.requests[0].terminal_outcome, Some(Stage::Failed));
        assert_eq!(
            after.requests[0].computer.capture_state,
            before.requests[0].computer.capture_state
        );
        assert_eq!(
            after.requests[0].computer.indicator_state,
            before.requests[0].computer.indicator_state
        );
        assert_eq!(after.requests[0].verification.last_verified_at, None);
    }
}
