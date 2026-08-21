//! Ambient Voice & Thought Reactive Orb Surface for SWAL Desktop
//!
//! Provides the state machine, lock-free audio amplitude & thought trigger consumer,
//! and shader compiler/pipeline bindings for ambient visual states.

pub mod shaders;
pub mod socket;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Ambient operational states of the SWAL Orb
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrbState {
    /// Pulsing cyan energy ripple (#06b6d4)
    Listening,
    /// Orange multi-frequency interference (#f97316)
    Thinking,
    /// Morphed fluid particle boundary
    Speaking,
}

impl Default for OrbState {
    fn default() -> Self {
        Self::Listening
    }
}

/// Uniforms passed to GLSL fragment shaders for rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OrbUniforms {
    pub time: f32,
    pub audio_amplitude: f32,
    pub thought_trigger: f32,
    pub padding: f32,
}

impl Default for OrbUniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            audio_amplitude: 0.0,
            thought_trigger: 0.0,
            padding: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orb_state_default() {
        assert_eq!(OrbState::default(), OrbState::Listening);
    }

    #[test]
    fn test_lock_free_audio_consumer_amplitude() {
        let consumer = LockFreeAudioConsumer::new();
        assert_eq!(consumer.get_audio_amplitude(), 0.0);

        consumer.set_audio_amplitude(0.75);
        assert!((consumer.get_audio_amplitude() - 0.75).abs() < f32::EPSILON);

        // Test clamping upper and lower bounds
        consumer.set_audio_amplitude(1.5);
        assert_eq!(consumer.get_audio_amplitude(), 1.0);

        consumer.set_audio_amplitude(-0.5);
        assert_eq!(consumer.get_audio_amplitude(), 0.0);
    }

    #[test]
    fn test_lock_free_audio_consumer_thought_trigger() {
        let consumer = LockFreeAudioConsumer::new();
        assert_eq!(consumer.get_thought_trigger(), 0.0);

        consumer.set_thought_trigger(0.9);
        assert!((consumer.get_thought_trigger() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_orb_state_transitions() {
        let consumer = LockFreeAudioConsumer::new();
        let mut controller = OrbController::new(consumer.clone());

        assert_eq!(controller.state(), OrbState::Listening);
        assert!(controller.current_shader_source().contains("CYAN_CYBER"));

        controller.transition_to(OrbState::Thinking);
        assert_eq!(controller.state(), OrbState::Thinking);
        assert_eq!(consumer.get_state(), OrbState::Thinking);
        assert!(controller.current_shader_source().contains("ORANGE_THOUGHT"));

        controller.transition_to(OrbState::Speaking);
        assert_eq!(controller.state(), OrbState::Speaking);
        assert_eq!(consumer.get_state(), OrbState::Speaking);
        assert!(controller.current_shader_source().contains("EMERALD_PARTICLE"));
    }

    #[test]
    fn test_process_signal() {
        let consumer = LockFreeAudioConsumer::new();
        consumer.process_signal(OrbInputSignal::AudioLevel(0.85));
        assert!((consumer.get_audio_amplitude() - 0.85).abs() < f32::EPSILON);

        consumer.process_signal(OrbInputSignal::ThoughtTrigger(0.65));
        assert!((consumer.get_thought_trigger() - 0.65).abs() < f32::EPSILON);

        consumer.process_signal(OrbInputSignal::SetState(OrbState::Speaking));
        assert_eq!(consumer.get_state(), OrbState::Speaking);
    }

    #[test]
    fn test_controller_tick_uniforms() {
        let consumer = LockFreeAudioConsumer::new();
        let mut controller = OrbController::new(consumer.clone());

        consumer.set_audio_amplitude(0.4);
        consumer.set_thought_trigger(0.8);

        let uniforms = controller.tick(0.016);
        assert!((uniforms.time - 0.016).abs() < 1e-4);
        assert!((uniforms.audio_amplitude - 0.4).abs() < f32::EPSILON);
        assert!((uniforms.thought_trigger - 0.8).abs() < f32::EPSILON);
    }
}

/// Incoming signals for ambient audio and Xavier thought triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrbInputSignal {
    /// Microphone audio amplitude normalized [0.0, 1.0]
    AudioLevel(f32),
    /// Xavier thought trigger intensity [0.0, 1.0]
    ThoughtTrigger(f32),
    /// Request state transition
    SetState(OrbState),
}

/// Lock-free consumer for microphone audio levels and Xavier thought triggers.
/// Uses atomic bit conversion (`f32::to_bits` / `f32::from_bits`) to guarantee
/// wait-free, lock-free reads on high-frequency 200Hz+ render threads.
#[derive(Clone)]
pub struct LockFreeAudioConsumer {
    audio_amplitude_bits: Arc<AtomicU32>,
    thought_trigger_bits: Arc<AtomicU32>,
    state_bits: Arc<AtomicU32>,
}

impl LockFreeAudioConsumer {
    pub fn new() -> Self {
        let initial_state = OrbState::Listening as u32;
        Self {
            audio_amplitude_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            thought_trigger_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            state_bits: Arc::new(AtomicU32::new(initial_state)),
        }
    }

    /// Lock-free update of microphone audio amplitude
    pub fn set_audio_amplitude(&self, level: f32) {
        let clamped = level.clamp(0.0, 1.0);
        self.audio_amplitude_bits
            .store(clamped.to_bits(), Ordering::Release);
    }

    /// Lock-free read of current audio amplitude
    pub fn get_audio_amplitude(&self) -> f32 {
        f32::from_bits(self.audio_amplitude_bits.load(Ordering::Acquire))
    }

    /// Lock-free update of Xavier thought trigger intensity
    pub fn set_thought_trigger(&self, intensity: f32) {
        let clamped = intensity.clamp(0.0, 1.0);
        self.thought_trigger_bits
            .store(clamped.to_bits(), Ordering::Release);
    }

    /// Lock-free read of current thought trigger intensity
    pub fn get_thought_trigger(&self) -> f32 {
        f32::from_bits(self.thought_trigger_bits.load(Ordering::Acquire))
    }

    /// Lock-free update of current Orb state
    pub fn set_state(&self, state: OrbState) {
        self.state_bits.store(state as u32, Ordering::Release);
    }

    /// Lock-free read of current Orb state
    pub fn get_state(&self) -> OrbState {
        match self.state_bits.load(Ordering::Acquire) {
            1 => OrbState::Thinking,
            2 => OrbState::Speaking,
            _ => OrbState::Listening,
        }
    }

    /// Process an input signal
    pub fn process_signal(&self, signal: OrbInputSignal) {
        match signal {
            OrbInputSignal::AudioLevel(level) => self.set_audio_amplitude(level),
            OrbInputSignal::ThoughtTrigger(intensity) => self.set_thought_trigger(intensity),
            OrbInputSignal::SetState(state) => self.set_state(state),
        }
    }
}

impl Default for LockFreeAudioConsumer {
    fn default() -> Self {
        Self::new()
    }
}

/// Orb surface state machine and shader compiler coordinator
pub struct OrbController {
    current_state: OrbState,
    consumer: LockFreeAudioConsumer,
    elapsed_time: f32,
}

impl OrbController {
    pub fn new(consumer: LockFreeAudioConsumer) -> Self {
        Self {
            current_state: OrbState::Listening,
            consumer,
            elapsed_time: 0.0,
        }
    }

    pub fn state(&self) -> OrbState {
        self.current_state
    }

    pub fn transition_to(&mut self, new_state: OrbState) {
        self.current_state = new_state;
        self.consumer.set_state(new_state);
    }

    /// Returns the GLSL fragment shader source code for the active Orb state
    pub fn current_shader_source(&self) -> &'static str {
        shaders::get_shader_for_state(&self.current_state)
    }

    /// Advance time by `dt` seconds and snapshot uniforms for rendering
    pub fn tick(&mut self, dt: f32) -> OrbUniforms {
        self.elapsed_time += dt;
        // Sync state if consumer updated externally
        self.current_state = self.consumer.get_state();

        OrbUniforms {
            time: self.elapsed_time,
            audio_amplitude: self.consumer.get_audio_amplitude(),
            thought_trigger: self.consumer.get_thought_trigger(),
            padding: 0.0,
        }
    }
}
