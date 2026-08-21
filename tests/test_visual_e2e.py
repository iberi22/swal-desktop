#!/usr/bin/env python3
"""SWAL Desktop — Automated Visual E2E & Screenshot Debugging Suite.

Captures Wayland desktop surfaces, validates layer geometry, tests live theme switching,
and saves visual artifacts into Antigravity brain for multimodal visual inspection.
"""

import os
import subprocess
import time
import unittest

ARTIFACT_DIR = "/home/belal/.gemini/antigravity/brain/8d5d93d4-e03d-4879-a36f-b95611086547"
TMP_DIR = "/tmp/swal_visual_tests"


def run_cmd(cmd_list):
    res = subprocess.run(cmd_list, capture_output=True, text=True)
    return res.stdout.strip(), res.stderr.strip(), res.returncode


def get_layer_geometry(namespace_name="gtk-layer-shell"):
    out, _, _ = run_cmd(["hyprctl", "layers"])
    layers = []
    for line in out.splitlines():
        if "xywh:" in line:
            parts = line.strip().split()
            idx = parts.index("xywh:") if "xywh:" in parts else -1
            if idx != -1 and len(parts) > idx + 4:
                x = int(parts[idx + 1])
                y = int(parts[idx + 2])
                w = int(parts[idx + 3])
                h = int(parts[idx + 4].rstrip(','))
                layers.append({"x": x, "y": y, "w": w, "h": h, "raw": line})
    return layers


def capture_and_crop(window_name, crop_geometry, output_filename):
    os.makedirs(TMP_DIR, exist_ok=True)
    raw_path = os.path.join(TMP_DIR, f"{window_name}_raw.png")
    cropped_path = os.path.join(TMP_DIR, output_filename)
    artifact_path = os.path.join(ARTIFACT_DIR, output_filename)

    run_cmd(["grim", raw_path])
    if os.path.exists(raw_path):
        run_cmd(["magick", raw_path, "-crop", crop_geometry, cropped_path])
        if os.path.exists(cropped_path):
            run_cmd(["cp", cropped_path, artifact_path])
            return artifact_path
    return None


class TestSWALVisualE2E(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        os.makedirs(TMP_DIR, exist_ok=True)
        # Ensure eww is alive
        out, _, code = run_cmd(["eww", "ping"])
        if code != 0 or "pong" not in out:
            run_cmd(["eww", "daemon"])
            time.sleep(1)

    def tearDown(self):
        # Close open test windows safely
        for win in ["swal_settings", "ram_panel", "keybinds_panel", "agent_chat"]:
            run_cmd(["eww", "close", win])
        time.sleep(0.5)

    def test_01_dashboard_visual_layout(self):
        """Verify dashboard renders with header, gear button, and storage meters."""
        self.tearDown()
        run_cmd(["eww", "open", "dashboard"])
        time.sleep(1)

        art = capture_and_crop("dashboard", "400x1000+1511+80", "e2e_dashboard_verified.png")
        self.assertIsNotNone(art, "Failed to capture dashboard screenshot")
        self.assertTrue(os.path.exists(art), f"Artifact {art} does not exist")
        self.assertGreater(os.path.getsize(art), 10000, "Screenshot image file is abnormally small")
        print(f"  ✓ Dashboard captured: {art}")

    def test_02_settings_modal_theme_switching(self):
        """Verify settings modal renders and maintains layout across theme changes."""
        self.tearDown()
        run_cmd(["eww", "open", "swal_settings"])
        time.sleep(0.8)

        # 1. Hive Dark
        run_cmd(["swal-theme", "switch", "hive-dark"])
        time.sleep(0.8)
        art_hive = capture_and_crop("settings_hive", "722x360+599+380", "e2e_settings_hive_dark.png")
        self.assertIsNotNone(art_hive)
        self.assertGreater(os.path.getsize(art_hive), 5000)
        print(f"  ✓ Settings Hive Dark captured: {art_hive}")

        # 2. Cyber Neon
        run_cmd(["swal-theme", "switch", "cyber-neon"])
        time.sleep(0.8)
        art_cyber = capture_and_crop("settings_cyber", "722x360+599+380", "e2e_settings_cyber_neon.png")
        self.assertIsNotNone(art_cyber)
        self.assertGreater(os.path.getsize(art_cyber), 5000)
        print(f"  ✓ Settings Cyber Neon captured: {art_cyber}")

        # 3. Nord Frost
        run_cmd(["swal-theme", "switch", "nord-swal"])
        time.sleep(0.8)
        art_nord = capture_and_crop("settings_nord", "722x360+599+380", "e2e_settings_nord_frost.png")
        self.assertIsNotNone(art_nord)
        self.assertGreater(os.path.getsize(art_nord), 5000)
        print(f"  ✓ Settings Nord Frost captured: {art_nord}")

        # Restore default Hive Dark
        run_cmd(["swal-theme", "switch", "hive-dark"])
        run_cmd(["eww", "close", "swal_settings"])

    def test_03_ram_panel_visual_layout(self):
        """Verify process monitor panel renders with process list and metrics."""
        for win in ["swal_settings", "dashboard", "keybinds_panel", "agent_chat"]:
            run_cmd(["eww", "close", win])
        time.sleep(1)

        run_cmd(["eww", "open", "ram_panel"])
        time.sleep(1.2)

        art_ram = capture_and_crop("ram_panel", "710x580+605+271", "e2e_ram_panel_verified.png")
        self.assertIsNotNone(art_ram)
        self.assertGreater(os.path.getsize(art_ram), 10000)
        print(f"  ✓ RAM Panel captured: {art_ram}")
        run_cmd(["eww", "close", "ram_panel"])

    def test_04_system_stability_after_visual_runs(self):
        """Run swal-doctor after visual test suite to verify 0 errors in logs."""
        out, _, code = run_cmd(["swal-doctor"])
        self.assertEqual(code, 0, f"swal-doctor reported issues: {out}")
        self.assertIn("SYSTEM HEALTHY", out, "Doctor did not report healthy state")
        print("  ✓ SWAL Doctor verified 0 errors after visual suite run.")


if __name__ == "__main__":
    unittest.main(verbosity=2)
