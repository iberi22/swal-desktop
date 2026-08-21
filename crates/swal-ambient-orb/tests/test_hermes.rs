#[path = "../src/hermes.rs"]
mod hermes;

#[test]
fn test_integration_hermes_module() {
    use hermes::{HermesAgentState, HermesOrbPacket, HermesEventDispatcher};

    let dispatcher = HermesEventDispatcher::default();
    let packet = HermesOrbPacket::new(HermesAgentState::DecomposingPlan)
        .with_summary("Integration test hermes module");

    assert_eq!(packet.agent_id, "hermes");
    assert_eq!(packet.state, HermesAgentState::DecomposingPlan);
    assert_eq!(packet.prompt_summary, "Integration test hermes module");

    assert!(dispatcher.dispatch(packet.clone()).is_ok());
    let recv = dispatcher.try_recv().unwrap();
    assert_eq!(recv, packet);
}
