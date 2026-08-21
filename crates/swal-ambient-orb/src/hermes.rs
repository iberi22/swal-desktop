//! Hermes Agent Protocol & Cognition State Machine
//!
//! Defines specialized agent cognition states, packet telemetry format, state transition validation,
//! and crossbeam-channel based event dispatching with registered callbacks for the Hermes agent.

#![allow(dead_code)]

use crossbeam_channel::{bounded, unbounded, Receiver, SendError, Sender, TryRecvError};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, Mutex};

/// Hermes Agent cognition states for ambient voice & desktop UI orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HermesAgentState {
    /// Agent is idle and ready for user input.
    #[default]
    Idle,
    /// Agent is listening to voice audio stream.
    ListeningVoice,
    /// Agent is decomposing intent into an execution plan.
    DecomposingPlan,
    /// Agent is streaming Generative A2UI layout/widgets.
    StreamingA2Ui,
    /// Agent is executing a tool or system action.
    ExecutingToolAction,
    /// Agent is paused awaiting direct user feedback/confirmation.
    AwaitingUserFeedback,
    /// Agent encountered an error state.
    ErrorAlert,
}

impl HermesAgentState {
    /// Returns true if transitioning from `self` to `target` is valid.
    pub fn can_transition_to(&self, target: HermesAgentState) -> bool {
        can_transition(*self, target)
    }

    /// Validates transition from `self` to `target`, returning `Ok(())` or `TransitionError`.
    pub fn validate_transition_to(&self, target: HermesAgentState) -> Result<(), TransitionError> {
        validate_transition(*self, target)
    }
}

/// Error returned when an invalid state transition is attempted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionError {
    pub from: HermesAgentState,
    pub to: HermesAgentState,
    pub reason: String,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid Hermes transition from {:?} to {:?}: {}",
            self.from, self.to, self.reason
        )
    }
}

impl std::error::Error for TransitionError {}

/// State transition validator function.
pub fn can_transition(from: HermesAgentState, to: HermesAgentState) -> bool {
    if from == to {
        return true;
    }

    // ErrorAlert can be reached from any state
    if to == HermesAgentState::ErrorAlert {
        return true;
    }

    match from {
        HermesAgentState::Idle => matches!(
            to,
            HermesAgentState::ListeningVoice
                | HermesAgentState::DecomposingPlan
                | HermesAgentState::ExecutingToolAction
                | HermesAgentState::StreamingA2Ui
                | HermesAgentState::AwaitingUserFeedback
        ),
        HermesAgentState::ListeningVoice => matches!(
            to,
            HermesAgentState::DecomposingPlan
                | HermesAgentState::Idle
                | HermesAgentState::ExecutingToolAction
                | HermesAgentState::AwaitingUserFeedback
        ),
        HermesAgentState::DecomposingPlan => matches!(
            to,
            HermesAgentState::ExecutingToolAction
                | HermesAgentState::StreamingA2Ui
                | HermesAgentState::AwaitingUserFeedback
                | HermesAgentState::Idle
        ),
        HermesAgentState::StreamingA2Ui => matches!(
            to,
            HermesAgentState::ExecutingToolAction
                | HermesAgentState::AwaitingUserFeedback
                | HermesAgentState::Idle
                | HermesAgentState::DecomposingPlan
        ),
        HermesAgentState::ExecutingToolAction => matches!(
            to,
            HermesAgentState::StreamingA2Ui
                | HermesAgentState::AwaitingUserFeedback
                | HermesAgentState::DecomposingPlan
                | HermesAgentState::Idle
        ),
        HermesAgentState::AwaitingUserFeedback => matches!(
            to,
            HermesAgentState::ListeningVoice
                | HermesAgentState::DecomposingPlan
                | HermesAgentState::ExecutingToolAction
                | HermesAgentState::Idle
        ),
        HermesAgentState::ErrorAlert => matches!(
            to,
            HermesAgentState::Idle
                | HermesAgentState::ListeningVoice
                | HermesAgentState::DecomposingPlan
        ),
    }
}

/// Validates whether a state transition from `from` to `to` is allowed.
pub fn validate_transition(from: HermesAgentState, to: HermesAgentState) -> Result<(), TransitionError> {
    if can_transition(from, to) {
        Ok(())
    } else {
        Err(TransitionError {
            from,
            to,
            reason: format!("Direct transition from {:?} to {:?} is forbidden by cognition state machine", from, to),
        })
    }
}

/// Telemetry and state synchronization packet for the Hermes Agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HermesOrbPacket {
    /// Identifier of the agent (defaults to "hermes").
    pub agent_id: String,
    /// Active cognition state of the agent.
    pub state: HermesAgentState,
    /// Summary of current prompt or task instruction.
    pub prompt_summary: String,
    /// Normalized audio amplitude level [0.0, 1.0].
    pub audio_level: f32,
    /// Execution progress percentage [0.0, 1.0].
    pub progress_pct: f32,
}

impl Default for HermesOrbPacket {
    fn default() -> Self {
        Self {
            agent_id: "hermes".to_string(),
            state: HermesAgentState::Idle,
            prompt_summary: String::new(),
            audio_level: 0.0,
            progress_pct: 0.0,
        }
    }
}

impl HermesOrbPacket {
    /// Creates a new packet for `hermes` with the specified state.
    pub fn new(state: HermesAgentState) -> Self {
        Self {
            state,
            ..Default::default()
        }
    }

    /// Builder pattern: set prompt summary.
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.prompt_summary = summary.into();
        self
    }

    /// Builder pattern: set audio level, clamping to [0.0, 1.0].
    pub fn with_audio_level(mut self, level: f32) -> Self {
        self.audio_level = level.clamp(0.0, 1.0);
        self
    }

    /// Builder pattern: set execution progress percentage, clamping to [0.0, 1.0].
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress_pct = progress.clamp(0.0, 1.0);
        self
    }

    /// Clamp numeric metrics to valid normalized ranges.
    pub fn clamp_values(&mut self) {
        self.audio_level = self.audio_level.clamp(0.0, 1.0);
        self.progress_pct = self.progress_pct.clamp(0.0, 1.0);
    }

    /// Transition packet state to target state if permitted.
    pub fn transition_to(&mut self, next_state: HermesAgentState) -> Result<(), TransitionError> {
        validate_transition(self.state, next_state)?;
        self.state = next_state;
        Ok(())
    }
}

/// Callback function type for event dispatching.
pub type HermesCallbackFn = Arc<dyn Fn(&HermesOrbPacket) + Send + Sync + 'static>;

/// Crossbeam-channel based event dispatcher with callback hooks for Hermes agent state events.
#[derive(Clone)]
pub struct HermesEventDispatcher {
    sender: Sender<HermesOrbPacket>,
    receiver: Receiver<HermesOrbPacket>,
    callbacks: Arc<Mutex<Vec<HermesCallbackFn>>>,
}

impl HermesEventDispatcher {
    /// Creates a new dispatcher. If `capacity` is provided, a bounded channel is created;
    /// otherwise an unbounded channel is created.
    pub fn new(capacity: Option<usize>) -> Self {
        let (sender, receiver) = match capacity {
            Some(cap) => bounded(cap),
            None => unbounded(),
        };
        Self {
            sender,
            receiver,
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns a copy of the channel sender.
    pub fn sender(&self) -> Sender<HermesOrbPacket> {
        self.sender.clone()
    }

    /// Returns a copy of the channel receiver.
    pub fn receiver(&self) -> Receiver<HermesOrbPacket> {
        self.receiver.clone()
    }

    /// Registers a callback function triggered on event dispatch.
    pub fn register_callback<F>(&self, callback: F)
    where
        F: Fn(&HermesOrbPacket) + Send + Sync + 'static,
    {
        if let Ok(mut cbs) = self.callbacks.lock() {
            cbs.push(Arc::new(callback));
        }
    }

    /// Dispatches a packet: sends it over the crossbeam channel and notifies all registered callbacks.
    pub fn dispatch(&self, packet: HermesOrbPacket) -> Result<(), SendError<HermesOrbPacket>> {
        if let Ok(cbs) = self.callbacks.lock() {
            for cb in cbs.iter() {
                cb(&packet);
            }
        }
        self.sender.send(packet)
    }

    /// Attempts to receive a packet from the channel without blocking.
    pub fn try_recv(&self) -> Result<HermesOrbPacket, TryRecvError> {
        self.receiver.try_recv()
    }

    /// Receives a packet from the channel, blocking until available.
    pub fn recv(&self) -> Result<HermesOrbPacket, crossbeam_channel::RecvError> {
        self.receiver.recv()
    }
}

impl Default for HermesEventDispatcher {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermes_agent_state_default() {
        assert_eq!(HermesAgentState::default(), HermesAgentState::Idle);
    }

    #[test]
    fn test_hermes_orb_packet_default_and_builder() {
        let packet = HermesOrbPacket::default();
        assert_eq!(packet.agent_id, "hermes");
        assert_eq!(packet.state, HermesAgentState::Idle);
        assert_eq!(packet.prompt_summary, "");
        assert_eq!(packet.audio_level, 0.0);
        assert_eq!(packet.progress_pct, 0.0);

        let packet = HermesOrbPacket::new(HermesAgentState::ListeningVoice)
            .with_summary("Analyze audio request")
            .with_audio_level(0.85)
            .with_progress(0.45);

        assert_eq!(packet.agent_id, "hermes");
        assert_eq!(packet.state, HermesAgentState::ListeningVoice);
        assert_eq!(packet.prompt_summary, "Analyze audio request");
        assert!((packet.audio_level - 0.85).abs() < f32::EPSILON);
        assert!((packet.progress_pct - 0.45).abs() < f32::EPSILON);
    }

    #[test]
    fn test_packet_clamping() {
        let mut packet = HermesOrbPacket::new(HermesAgentState::Idle)
            .with_audio_level(1.5)
            .with_progress(-0.5);

        assert_eq!(packet.audio_level, 1.0);
        assert_eq!(packet.progress_pct, 0.0);

        packet.audio_level = 2.0;
        packet.progress_pct = 3.0;
        packet.clamp_values();
        assert_eq!(packet.audio_level, 1.0);
        assert_eq!(packet.progress_pct, 1.0);
    }

    #[test]
    fn test_state_transitions_coverage() {
        let states = [
            HermesAgentState::Idle,
            HermesAgentState::ListeningVoice,
            HermesAgentState::DecomposingPlan,
            HermesAgentState::StreamingA2Ui,
            HermesAgentState::ExecutingToolAction,
            HermesAgentState::AwaitingUserFeedback,
            HermesAgentState::ErrorAlert,
        ];

        for &from in &states {
            for &to in &states {
                let valid = can_transition(from, to);
                let result = validate_transition(from, to);
                assert_eq!(valid, result.is_ok());
                assert_eq!(from.can_transition_to(to), valid);
                assert_eq!(from.validate_transition_to(to).is_ok(), valid);

                // Self transitions and transitions to ErrorAlert should always be valid
                if from == to || to == HermesAgentState::ErrorAlert {
                    assert!(valid, "Transition from {:?} to {:?} should be valid", from, to);
                }
            }
        }
    }

    #[test]
    fn test_specific_transition_rules() {
        let mut packet = HermesOrbPacket::new(HermesAgentState::Idle);

        assert!(packet.transition_to(HermesAgentState::ListeningVoice).is_ok());
        assert_eq!(packet.state, HermesAgentState::ListeningVoice);

        assert!(packet.transition_to(HermesAgentState::DecomposingPlan).is_ok());
        assert_eq!(packet.state, HermesAgentState::DecomposingPlan);

        assert!(packet.transition_to(HermesAgentState::ExecutingToolAction).is_ok());
        assert_eq!(packet.state, HermesAgentState::ExecutingToolAction);

        assert!(packet.transition_to(HermesAgentState::StreamingA2Ui).is_ok());
        assert_eq!(packet.state, HermesAgentState::StreamingA2Ui);

        assert!(packet.transition_to(HermesAgentState::AwaitingUserFeedback).is_ok());
        assert_eq!(packet.state, HermesAgentState::AwaitingUserFeedback);

        assert!(packet.transition_to(HermesAgentState::Idle).is_ok());
        assert_eq!(packet.state, HermesAgentState::Idle);

        // ErrorAlert and recovery
        assert!(packet.transition_to(HermesAgentState::ErrorAlert).is_ok());
        assert_eq!(packet.state, HermesAgentState::ErrorAlert);

        assert!(packet.transition_to(HermesAgentState::Idle).is_ok());
        assert_eq!(packet.state, HermesAgentState::Idle);
    }

    #[test]
    fn test_event_dispatcher_channel_and_callbacks() {
        let dispatcher = HermesEventDispatcher::new(Some(10));
        let _sender = dispatcher.sender();
        let _receiver = dispatcher.receiver();

        let callback_count = Arc::new(Mutex::new(0));

        let count_clone = callback_count.clone();
        dispatcher.register_callback(move |pkt| {
            if pkt.state == HermesAgentState::DecomposingPlan {
                if let Ok(mut count) = count_clone.lock() {
                    *count += 1;
                }
            }
        });

        let packet = HermesOrbPacket::new(HermesAgentState::DecomposingPlan)
            .with_summary("Planning layout");

        assert!(dispatcher.dispatch(packet.clone()).is_ok());

        let recv_packet = dispatcher.try_recv().unwrap();
        assert_eq!(recv_packet, packet);
        assert_eq!(*callback_count.lock().unwrap(), 1);

        assert!(dispatcher.dispatch(packet.clone()).is_ok());
        let recv_blocking = dispatcher.recv().unwrap();
        assert_eq!(recv_blocking, packet);
    }

    #[test]
    fn test_transition_error_display() {
        let err = TransitionError {
            from: HermesAgentState::ErrorAlert,
            to: HermesAgentState::StreamingA2Ui,
            reason: "Forbidden".to_string(),
        };
        assert!(err.to_string().contains("Invalid Hermes transition"));
    }
}
