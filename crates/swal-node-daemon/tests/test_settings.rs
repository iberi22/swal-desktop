#[path = "../src/settings_window.rs"]
mod settings_window;

#[test]
fn test_settings_window_integration_runner() {
    let settings = settings_window::SwalSystemSettings::default();
    for category in settings_window::SettingsCategory::all() {
        let layout = settings_window::SettingsWindowBuilder::build_settings_layout(category, &settings);
        let serialized = serde_json::to_string(&layout).expect("Must serialize layout");
        assert!(!serialized.is_empty());
    }
}
