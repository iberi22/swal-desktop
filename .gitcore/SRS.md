# SWAL Desktop — Software Requirements Specification (SRS v1.1.0)

## 1. Functional Requirements

### 1.1 Dynamic Theme Management
- **FR-01**: The system shall support hot-swappable visual themes defined in JSON (`schemas/theme.schema.json`).
- **FR-02**: Theme switching must apply across Eww layer shell, Hyprland window borders, and GTK applications within 15 milliseconds.
- **FR-03**: The default theme shall be `SWAL Hive Dark` (`hive-dark.json`) matching `@swal/ui` design tokens.
- **FR-04**: The system shall provide an interactive GUI theme picker via Rofi (`swal-theme picker`).

### 1.2 Desktop Management & System Configuration
- **FR-05**: The Dashboard header shall feature an interactive gear button (`⚙`) opening the Manual Settings panel (`swal_settings`).
- **FR-06**: The Settings panel shall provide 4 tabs: Apariencia, Nodo & Rust Core, Rendimiento, and Diagnóstico.
- **FR-07**: The Process Monitor shall display live CPU/GPU/RAM metrics and allow terminating specific processes via interactive kill buttons.

### 1.3 Autonomous Node & Background Daemons
- **FR-08**: NixOS configuration must provide auto-starting systemd user services for Xavier Cognitive Memory (`:8006`) and Edge-Mesh P2P daemon.
- **FR-09**: The system shall include `swal-doctor` diagnostic CLI verifying SCSS encoding, YUCK syntax, log health, and service availability.

### 1.4 Native Rust Core Migration (Wave 2)
- **FR-10**: Telemetry gathering shall be ported to native Rust reading directly from `/proc` and sysfs.
- **FR-11**: The Generative A2UI engine shall parse declarative JSON widget trees into native GPU-accelerated Wayland surfaces.

---

## 2. Non-Functional Requirements
- **NFR-01 (Performance)**: Idle CPU usage of desktop daemons must remain below 0.5%.
- **NFR-02 (Safety)**: Zero memory leaks and strict error isolation in telemetry parsers.
- **NFR-03 (Reproducibility)**: One-command node deployment on any workstation via `scripts/install.sh` and NixOS Flake.
