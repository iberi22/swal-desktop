//! Integration test suite for Hermes Orb pipeline and A2UI streaming payload interface.

use swal_ambient_orb::shaders;
use swal_ambient_orb::{LockFreeAudioConsumer, OrbController, OrbInputSignal, OrbState, OrbUniforms};

#[test]
fn test_hermes_state_transitions_all_variants() {
    let consumer = LockFreeAudioConsumer::new();
    let mut controller = OrbController::new(consumer.clone());

    // Initial default state must be Listening
    assert_eq!(controller.state(), OrbState::Listening);
    assert_eq!(consumer.get_state(), OrbState::Listening);

    // Transition to Thinking
    controller.transition_to(OrbState::Thinking);
    assert_eq!(controller.state(), OrbState::Thinking);
    assert_eq!(consumer.get_state(), OrbState::Thinking);

    // Transition to Speaking
    controller.transition_to(OrbState::Speaking);
    assert_eq!(controller.state(), OrbState::Speaking);
    assert_eq!(consumer.get_state(), OrbState::Speaking);

    // Transition back to Listening
    controller.transition_to(OrbState::Listening);
    assert_eq!(controller.state(), OrbState::Listening);
    assert_eq!(consumer.get_state(), OrbState::Listening);

    // Test signal processing updating state
    consumer.process_signal(OrbInputSignal::SetState(OrbState::Thinking));
    assert_eq!(consumer.get_state(), OrbState::Thinking);
    let uniforms = controller.tick(0.016);
    assert_eq!(controller.state(), OrbState::Thinking);
    let _ = uniforms;
}

#[test]
fn test_hermes_shader_lookup_and_fallback() {
    let states = [OrbState::Listening, OrbState::Thinking, OrbState::Speaking];

    for state in &states {
        let shader = shaders::get_shader_for_state(state);
        assert!(!shader.is_empty(), "Shader for state {:?} should not be empty", state);
        assert!(
            shader.contains("#version 450"),
            "Shader for state {:?} should include GLSL header",
            state
        );
    }

    let listening_shader = shaders::get_shader_for_state(&OrbState::Listening);
    assert!(listening_shader.contains("CYAN_CYBER") || listening_shader.contains("#06b6d4"));

    let thinking_shader = shaders::get_shader_for_state(&OrbState::Thinking);
    assert!(thinking_shader.contains("ORANGE_THOUGHT") || thinking_shader.contains("#f97316"));

    let speaking_shader = shaders::get_shader_for_state(&OrbState::Speaking);
    assert!(speaking_shader.contains("EMERALD_PARTICLE") || speaking_shader.contains("#00ff88"));

    let consumer = LockFreeAudioConsumer::new();
    let mut controller = OrbController::new(consumer);
    assert_eq!(controller.current_shader_source(), listening_shader);

    controller.transition_to(OrbState::Thinking);
    assert_eq!(controller.current_shader_source(), thinking_shader);

    controller.transition_to(OrbState::Speaking);
    assert_eq!(controller.current_shader_source(), speaking_shader);
}

#[tokio::test]
async fn test_hermes_ipc_packet_serialization_roundtrip() {
    // Serde JSON roundtrip checks
    let state = OrbState::Thinking;
    let serialized_state = match serde_json::to_string(&state) {
        Ok(s) => s,
        Err(e) => panic!("Failed to serialize OrbState: {}", e),
    };
    let deserialized_state: OrbState = match serde_json::from_str(&serialized_state) {
        Ok(s) => s,
        Err(e) => panic!("Failed to deserialize OrbState: {}", e),
    };
    assert_eq!(state, deserialized_state);

    let uniforms = OrbUniforms {
        time: 1.25,
        audio_amplitude: 0.8,
        thought_trigger: 0.4,
        padding: 0.0,
    };
    let serialized_uniforms = match serde_json::to_string(&uniforms) {
        Ok(s) => s,
        Err(e) => panic!("Failed to serialize OrbUniforms: {}", e),
    };
    let deserialized_uniforms: OrbUniforms = match serde_json::from_str(&serialized_uniforms) {
        Ok(u) => u,
        Err(e) => panic!("Failed to deserialize OrbUniforms: {}", e),
    };
    assert_eq!(uniforms, deserialized_uniforms);

    let signal = OrbInputSignal::AudioLevel(0.95);
    let serialized_signal = match serde_json::to_string(&signal) {
        Ok(s) => s,
        Err(e) => panic!("Failed to serialize OrbInputSignal: {}", e),
    };
    let deserialized_signal: OrbInputSignal = match serde_json::from_str(&serialized_signal) {
        Ok(sig) => sig,
        Err(e) => panic!("Failed to deserialize OrbInputSignal: {}", e),
    };
    if let OrbInputSignal::AudioLevel(level) = deserialized_signal {
        assert!((level - 0.95).abs() < f32::EPSILON);
    } else {
        panic!("Expected OrbInputSignal::AudioLevel variant");
    }

    // Unix Socket IPC mock test
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => panic!("Failed to create tempdir: {}", e),
    };
    let socket_path = temp_dir.path().join("hermes_ipc.sock");

    let listener = match tokio::net::UnixListener::bind(&socket_path) {
        Ok(l) => l,
        Err(e) => panic!("Failed to bind UnixListener: {}", e),
    };

    let payload = match serde_json::to_vec(&OrbInputSignal::ThoughtTrigger(0.88)) {
        Ok(p) => p,
        Err(e) => panic!("Failed to serialize payload: {}", e),
    };

    let payload_clone = payload.clone();
    let server_task = tokio::spawn(async move {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                use tokio::io::AsyncWriteExt;
                if let Err(e) = stream.write_all(&payload_clone).await {
                    panic!("Failed to write to stream: {}", e);
                }
            }
            Err(e) => panic!("Listener accept failed: {}", e),
        }
    });

    let client_task = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut stream = match tokio::net::UnixStream::connect(&socket_path).await {
            Ok(s) => s,
            Err(e) => panic!("Client connection failed: {}", e),
        };
        let mut buf = vec![0u8; 1024];
        let n = match stream.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => panic!("Read from socket failed: {}", e),
        };
        let packet: OrbInputSignal = match serde_json::from_slice(&buf[..n]) {
            Ok(p) => p,
            Err(e) => panic!("Deserialization failed: {}", e),
        };
        packet
    });

    let (server_res, client_res) = tokio::join!(server_task, client_task);
    if let Err(e) = server_res {
        panic!("Server task panicked: {}", e);
    }
    let received_signal = match client_res {
        Ok(sig) => sig,
        Err(e) => panic!("Client task panicked: {}", e),
    };

    if let OrbInputSignal::ThoughtTrigger(intensity) = received_signal {
        assert!((intensity - 0.88).abs() < f32::EPSILON);
    } else {
        panic!("Expected OrbInputSignal::ThoughtTrigger variant");
    }
}

#[test]
fn test_hermes_orb_progress_and_audio_clamping() {
    let consumer = LockFreeAudioConsumer::new();
    let mut controller = OrbController::new(consumer.clone());

    // Test time accumulation
    let uniforms1 = controller.tick(0.016);
    assert!((uniforms1.time - 0.016).abs() < 1e-4);

    let uniforms2 = controller.tick(0.033);
    assert!((uniforms2.time - 0.049).abs() < 1e-4);

    // Audio level clamping bounds [0.0, 1.0]
    consumer.set_audio_amplitude(1.8);
    assert_eq!(consumer.get_audio_amplitude(), 1.0);

    consumer.set_audio_amplitude(-0.7);
    assert_eq!(consumer.get_audio_amplitude(), 0.0);

    // Thought trigger clamping bounds [0.0, 1.0]
    consumer.set_thought_trigger(2.5);
    assert_eq!(consumer.get_thought_trigger(), 1.0);

    consumer.set_thought_trigger(-1.2);
    assert_eq!(consumer.get_thought_trigger(), 0.0);

    // Signal processing clamping
    consumer.process_signal(OrbInputSignal::AudioLevel(1.5));
    assert_eq!(consumer.get_audio_amplitude(), 1.0);

    consumer.process_signal(OrbInputSignal::AudioLevel(-0.3));
    assert_eq!(consumer.get_audio_amplitude(), 0.0);

    consumer.process_signal(OrbInputSignal::ThoughtTrigger(1.2));
    assert_eq!(consumer.get_thought_trigger(), 1.0);

    consumer.process_signal(OrbInputSignal::ThoughtTrigger(-0.8));
    assert_eq!(consumer.get_thought_trigger(), 0.0);

    // Ticking uniforms reflect current consumer values
    consumer.set_audio_amplitude(0.6);
    consumer.set_thought_trigger(0.3);
    let uniforms3 = controller.tick(0.01);
    assert!((uniforms3.audio_amplitude - 0.6).abs() < f32::EPSILON);
    assert!((uniforms3.thought_trigger - 0.3).abs() < f32::EPSILON);
}
