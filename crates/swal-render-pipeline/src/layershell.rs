//! Wayland Layer Shell Protocol Surface Manager (zwlr_layer_shell_v1)
//! Pure Rust implementation for native overlay and UI surface management.

use serde::{Deserialize, Serialize};

/// Wayland Layer Shell Protocol Layer Placement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum LayerType {
    Background = 0,
    Bottom = 1,
    Top = 2,
    Overlay = 3,
}

impl LayerType {
    pub fn to_protocol_value(&self) -> u32 {
        *self as u32
    }
}

/// Anchor edges for Wayland Layer Surface positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum AnchorEdge {
    Top = 1,
    Bottom = 2,
    Left = 4,
    Right = 8,
}

impl AnchorEdge {
    pub fn bitmask(&self) -> u32 {
        *self as u32
    }
}

/// Keyboard interactivity mode for layer surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum KeyboardInteractivity {
    None = 0,
    Exclusive = 1,
    OnDemand = 2,
}

impl KeyboardInteractivity {
    pub fn to_protocol_value(&self) -> u32 {
        *self as u32
    }
}

/// Declarative surface layout configuration for Layer Shell
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerSurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub margin: (i32, i32, i32, i32), // (top, right, bottom, left)
    pub anchors: Vec<AnchorEdge>,
    pub layer: LayerType,
    pub keyboard_interactivity: KeyboardInteractivity,
    pub exclusive_zone: i32,
}

impl LayerSurfaceConfig {
    pub fn new(width: u32, height: u32, layer: LayerType) -> Self {
        Self {
            width,
            height,
            margin: (0, 0, 0, 0),
            anchors: Vec::new(),
            layer,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: 0,
        }
    }

    pub fn with_margin(mut self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
        self.margin = (top, right, bottom, left);
        self
    }

    pub fn with_anchors(mut self, anchors: Vec<AnchorEdge>) -> Self {
        self.anchors = anchors;
        self
    }

    pub fn with_keyboard_interactivity(mut self, mode: KeyboardInteractivity) -> Self {
        self.keyboard_interactivity = mode;
        self
    }

    pub fn with_exclusive_zone(mut self, zone: i32) -> Self {
        self.exclusive_zone = zone;
        self
    }

    /// Calculates bitwise OR of all anchor edge flags
    pub fn anchor_bitmask(&self) -> u32 {
        self.anchors.iter().fold(0, |acc, &anchor| acc | anchor.bitmask())
    }
}

/// Mock/tracked protocol requests sent to the Wayland server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolRequest {
    SetSize { width: u32, height: u32 },
    SetAnchor { bitmask: u32 },
    SetLayer { layer: LayerType },
    SetMargin { top: i32, right: i32, bottom: i32, left: i32 },
    SetKeyboardInteractivity { mode: KeyboardInteractivity },
    SetExclusiveZone { zone: i32 },
    AckConfigure { serial: u32 },
    Commit,
    Destroy,
}

/// Native Wayland Layer Shell Surface Manager handling configuration, commit states, resize ack, and protocol requests.
#[derive(Debug, Clone)]
pub struct WaylandLayerSurface {
    config: LayerSurfaceConfig,
    current_width: u32,
    current_height: u32,
    committed: bool,
    configured: bool,
    pending_ack_serial: Option<u32>,
    last_acked_serial: Option<u32>,
    closed: bool,
    protocol_requests: Vec<ProtocolRequest>,
}

impl WaylandLayerSurface {
    pub fn new(config: LayerSurfaceConfig) -> Self {
        let initial_width = config.width;
        let initial_height = config.height;

        let mut surface = Self {
            config: config.clone(),
            current_width: initial_width,
            current_height: initial_height,
            committed: false,
            configured: false,
            pending_ack_serial: None,
            last_acked_serial: None,
            closed: false,
            protocol_requests: Vec::new(),
        };

        surface.sync_protocol_requests();
        surface
    }

    pub fn config(&self) -> &LayerSurfaceConfig {
        &self.config
    }

    pub fn current_size(&self) -> (u32, u32) {
        (self.current_width, self.current_height)
    }

    pub fn is_committed(&self) -> bool {
        self.committed
    }

    pub fn is_configured(&self) -> bool {
        self.configured
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn pending_ack_serial(&self) -> Option<u32> {
        self.pending_ack_serial
    }

    pub fn last_acked_serial(&self) -> Option<u32> {
        self.last_acked_serial
    }

    pub fn protocol_requests(&self) -> &[ProtocolRequest] {
        &self.protocol_requests
    }

    fn sync_protocol_requests(&mut self) {
        self.protocol_requests.push(ProtocolRequest::SetLayer {
            layer: self.config.layer,
        });
        self.protocol_requests.push(ProtocolRequest::SetSize {
            width: self.config.width,
            height: self.config.height,
        });
        self.protocol_requests.push(ProtocolRequest::SetAnchor {
            bitmask: self.config.anchor_bitmask(),
        });
        self.protocol_requests.push(ProtocolRequest::SetMargin {
            top: self.config.margin.0,
            right: self.config.margin.1,
            bottom: self.config.margin.2,
            left: self.config.margin.3,
        });
        self.protocol_requests.push(ProtocolRequest::SetKeyboardInteractivity {
            mode: self.config.keyboard_interactivity,
        });
        self.protocol_requests.push(ProtocolRequest::SetExclusiveZone {
            zone: self.config.exclusive_zone,
        });
    }

    pub fn update_config(&mut self, new_config: LayerSurfaceConfig) {
        if self.config != new_config {
            self.config = new_config;
            self.sync_protocol_requests();
            self.committed = false;
        }
    }

    pub fn receive_configure(&mut self, serial: u32, suggested_width: u32, suggested_height: u32) {
        if suggested_width > 0 {
            self.current_width = suggested_width;
        }
        if suggested_height > 0 {
            self.current_height = suggested_height;
        }
        self.pending_ack_serial = Some(serial);
        self.configured = true;
    }

    pub fn ack_configure(&mut self, serial: u32) -> Result<(), &'static str> {
        if self.pending_ack_serial != Some(serial) {
            return Err("Invalid or unmatching configure serial");
        }
        self.last_acked_serial = Some(serial);
        self.pending_ack_serial = None;
        self.protocol_requests
            .push(ProtocolRequest::AckConfigure { serial });
        Ok(())
    }

    pub fn commit(&mut self) {
        self.committed = true;
        self.protocol_requests.push(ProtocolRequest::Commit);
    }

    pub fn close(&mut self) {
        if !self.closed {
            self.closed = true;
            self.protocol_requests.push(ProtocolRequest::Destroy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_type_and_keyboard_interactivity_protocol_values() {
        assert_eq!(LayerType::Background.to_protocol_value(), 0);
        assert_eq!(LayerType::Bottom.to_protocol_value(), 1);
        assert_eq!(LayerType::Top.to_protocol_value(), 2);
        assert_eq!(LayerType::Overlay.to_protocol_value(), 3);

        assert_eq!(KeyboardInteractivity::None.to_protocol_value(), 0);
        assert_eq!(KeyboardInteractivity::Exclusive.to_protocol_value(), 1);
        assert_eq!(KeyboardInteractivity::OnDemand.to_protocol_value(), 2);
    }

    #[test]
    fn test_anchor_edge_bitmask() {
        assert_eq!(AnchorEdge::Top.bitmask(), 1);
        assert_eq!(AnchorEdge::Bottom.bitmask(), 2);
        assert_eq!(AnchorEdge::Left.bitmask(), 4);
        assert_eq!(AnchorEdge::Right.bitmask(), 8);

        let config = LayerSurfaceConfig::new(1920, 1080, LayerType::Top).with_anchors(vec![
            AnchorEdge::Top,
            AnchorEdge::Left,
            AnchorEdge::Right,
        ]);

        assert_eq!(config.anchor_bitmask(), 1 | 4 | 8);
    }

    #[test]
    fn test_layer_surface_config_builder() {
        let config = LayerSurfaceConfig::new(800, 600, LayerType::Overlay)
            .with_margin(10, 20, 30, 40)
            .with_anchors(vec![AnchorEdge::Bottom, AnchorEdge::Right])
            .with_keyboard_interactivity(KeyboardInteractivity::Exclusive)
            .with_exclusive_zone(50);

        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.layer, LayerType::Overlay);
        assert_eq!(config.margin, (10, 20, 30, 40));
        assert_eq!(config.anchors, vec![AnchorEdge::Bottom, AnchorEdge::Right]);
        assert_eq!(config.keyboard_interactivity, KeyboardInteractivity::Exclusive);
        assert_eq!(config.exclusive_zone, 50);
        assert_eq!(config.anchor_bitmask(), 2 | 8);
    }

    #[test]
    fn test_wayland_layer_surface_initialization_and_sync() {
        let config = LayerSurfaceConfig::new(1280, 720, LayerType::Bottom)
            .with_margin(5, 5, 5, 5)
            .with_anchors(vec![AnchorEdge::Top, AnchorEdge::Bottom]);

        let surface = WaylandLayerSurface::new(config.clone());

        assert_eq!(surface.config(), &config);
        assert_eq!(surface.current_size(), (1280, 720));
        assert!(!surface.is_committed());
        assert!(!surface.is_configured());
        assert!(!surface.is_closed());
        assert_eq!(surface.pending_ack_serial(), None);
        assert_eq!(surface.last_acked_serial(), None);

        let reqs = surface.protocol_requests();
        assert_eq!(reqs.len(), 6);
        assert_eq!(
            reqs[0],
            ProtocolRequest::SetLayer {
                layer: LayerType::Bottom
            }
        );
        assert_eq!(
            reqs[1],
            ProtocolRequest::SetSize {
                width: 1280,
                height: 720
            }
        );
        assert_eq!(
            reqs[2],
            ProtocolRequest::SetAnchor {
                bitmask: 1 | 2
            }
        );
        assert_eq!(
            reqs[3],
            ProtocolRequest::SetMargin {
                top: 5,
                right: 5,
                bottom: 5,
                left: 5
            }
        );
    }

    #[test]
    fn test_wayland_layer_surface_configure_ack_and_commit() {
        let config = LayerSurfaceConfig::new(1000, 1000, LayerType::Top);
        let mut surface = WaylandLayerSurface::new(config);

        surface.receive_configure(101, 1000, 800);
        assert!(surface.is_configured());
        assert_eq!(surface.pending_ack_serial(), Some(101));
        assert_eq!(surface.current_size(), (1000, 800));

        let err = surface.ack_configure(999);
        assert!(err.is_err());

        let res = surface.ack_configure(101);
        assert!(res.is_ok());
        assert_eq!(surface.pending_ack_serial(), None);
        assert_eq!(surface.last_acked_serial(), Some(101));

        surface.commit();
        assert!(surface.is_committed());

        let reqs = surface.protocol_requests();
        assert!(reqs.contains(&ProtocolRequest::AckConfigure { serial: 101 }));
        assert!(reqs.contains(&ProtocolRequest::Commit));
    }

    #[test]
    fn test_wayland_layer_surface_update_config_and_close() {
        let config1 = LayerSurfaceConfig::new(400, 300, LayerType::Background);
        let mut surface = WaylandLayerSurface::new(config1);

        surface.commit();
        assert!(surface.is_committed());

        let config2 = LayerSurfaceConfig::new(500, 300, LayerType::Background);
        surface.update_config(config2.clone());

        assert_eq!(surface.config(), &config2);
        assert!(!surface.is_committed());

        surface.close();
        assert!(surface.is_closed());

        // Repeated close should not add redundant destroy requests
        let count_before = surface.protocol_requests().len();
        surface.close();
        assert_eq!(surface.protocol_requests().len(), count_before);

        let reqs = surface.protocol_requests();
        assert_eq!(reqs.last(), Some(&ProtocolRequest::Destroy));
    }
}
