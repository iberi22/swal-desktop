#!/usr/bin/env python3
"""Unit Test Suite for Files Fluent Theme Tokens & Schema Validator.

Validates fluent-dark.json and fluent-mica.json against schemas/theme.schema.json,
checks WCAG color contrast ratios (text vs background >= 4.5:1), and verifies
Hyprland border color definitions and Fluent Blue accents using Python standard library.
"""

import json
import math
import os
import re
import unittest

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCHEMA_PATH = os.path.join(PROJECT_ROOT, "schemas", "theme.schema.json")
THEMES_DIR = os.path.join(PROJECT_ROOT, "themes")
HOME_THEMES_DIR = os.path.expanduser("~/.config/swal/themes")


def parse_color_to_rgb(color_str):
    """Parses hex (#rgb, #rrggbb) or rgba/rgb strings into (r, g, b) tuple (0-255)."""
    color_str = color_str.strip()
    if color_str.startswith("#"):
        hex_val = color_str.lstrip("#")
        if len(hex_val) == 3:
            r = int(hex_val[0] * 2, 16)
            g = int(hex_val[1] * 2, 16)
            b = int(hex_val[2] * 2, 16)
            return (r, g, b)
        elif len(hex_val) in (6, 8):
            r = int(hex_val[0:2], 16)
            g = int(hex_val[2:4], 16)
            b = int(hex_val[4:6], 16)
            return (r, g, b)
    elif color_str.startswith("rgb"):
        m = re.search(r"rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)", color_str)
        if m:
            return (int(m.group(1)), int(m.group(2)), int(m.group(3)))
    raise ValueError(f"Unrecognized color format: {color_str}")


def relative_luminance(rgb):
    """Calculates relative luminance for an (r, g, b) tuple per WCAG 2.1 specs."""
    rgb_normalized = []
    for c in rgb:
        s = c / 255.0
        if s <= 0.04045:
            rgb_normalized.append(s / 12.92)
        else:
            rgb_normalized.append(((s + 0.055) / 1.055) ** 2.4)
    r, g, b = rgb_normalized
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast_ratio(color1, color2):
    """Computes contrast ratio between two color strings."""
    rgb1 = parse_color_to_rgb(color1)
    rgb2 = parse_color_to_rgb(color2)
    l1 = relative_luminance(rgb1)
    l2 = relative_luminance(rgb2)
    lighter = max(l1, l2)
    darker = min(l1, l2)
    return (lighter + 0.05) / (darker + 0.05)


def validate_against_schema(data, schema):
    """Validates theme data dictionary against schema definition using stdlib python."""
    # Validate required top-level fields
    for req in schema.get("required", []):
        if req not in data:
            raise ValueError(f"Missing required top-level field: {req}")

    # Validate id pattern if defined
    if "id" in schema.get("properties", {}) and "pattern" in schema["properties"]["id"]:
        pattern = schema["properties"]["id"]["pattern"]
        if not re.match(pattern, data.get("id", "")):
            raise ValueError(f"id '{data.get('id')}' does not match pattern '{pattern}'")

    # Validate colors object and required color properties
    colors_schema = schema.get("properties", {}).get("colors", {})
    if "colors" in data:
        colors_data = data["colors"]
        for req in colors_schema.get("required", []):
            if req not in colors_data:
                raise ValueError(f"Missing required color field: {req}")


class TestFluentThemeTokensAndSchema(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        with open(SCHEMA_PATH, "r", encoding="utf-8") as f:
            cls.schema = json.load(f)

    def _get_theme_file_path(self, theme_filename):
        repo_path = os.path.join(THEMES_DIR, theme_filename)
        home_path = os.path.join(HOME_THEMES_DIR, theme_filename)
        if os.path.exists(repo_path):
            return repo_path
        elif os.path.exists(home_path):
            return home_path
        raise FileNotFoundError(f"Theme file '{theme_filename}' not found in '{THEMES_DIR}' or '{HOME_THEMES_DIR}'")

    def load_theme(self, theme_filename):
        path = self._get_theme_file_path(theme_filename)
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)

    def test_fluent_dark_schema(self):
        """Validates fluent-dark.json theme against schemas/theme.schema.json."""
        data = self.load_theme("fluent-dark.json")
        validate_against_schema(data, self.schema)
        self.assertEqual(data["id"], "fluent-dark")

    def test_fluent_mica_schema(self):
        """Validates fluent-mica.json theme against schemas/theme.schema.json."""
        data = self.load_theme("fluent-mica.json")
        validate_against_schema(data, self.schema)
        self.assertEqual(data["id"], "fluent-mica")

    def test_fluent_dark_contrast_ratio(self):
        """Verifies text vs background color contrast ratio >= 4.5:1 for fluent-dark."""
        data = self.load_theme("fluent-dark.json")
        colors = data["colors"]
        ratio = contrast_ratio(colors["text_primary"], colors["bg"])
        self.assertGreaterEqual(
            ratio, 4.5,
            f"Primary text contrast ratio {ratio:.2f} is lower than WCAG requirement 4.5:1"
        )

    def test_fluent_mica_contrast_ratio(self):
        """Verifies text vs background color contrast ratio >= 4.5:1 for fluent-mica."""
        data = self.load_theme("fluent-mica.json")
        colors = data["colors"]
        ratio = contrast_ratio(colors["text_primary"], colors["bg"])
        self.assertGreaterEqual(
            ratio, 4.5,
            f"Primary text contrast ratio {ratio:.2f} is lower than WCAG requirement 4.5:1"
        )

    def test_hyprland_border_color_definitions(self):
        """Verifies presence of Hyprland border color definitions in Fluent themes."""
        for theme_name in ["fluent-dark.json", "fluent-mica.json"]:
            data = self.load_theme(theme_name)
            self.assertIn("hyprland", data, f"Theme {theme_name} missing 'hyprland' section")
            hypr = data["hyprland"]
            self.assertIn("active_border", hypr, f"Theme {theme_name} missing 'active_border'")
            self.assertIn("inactive_border", hypr, f"Theme {theme_name} missing 'inactive_border'")
            self.assertTrue(hypr["active_border"].startswith("rgba("))
            self.assertTrue(hypr["inactive_border"].startswith("rgba("))

    def test_fluent_blue_accents(self):
        """Verifies presence of Fluent Blue accents in Fluent themes."""
        fluent_blue_shades = ["#0078d4", "#0f6cbd", "#2b88d8", "#005a9e"]
        for theme_name in ["fluent-dark.json", "fluent-mica.json"]:
            data = self.load_theme(theme_name)
            colors = data.get("colors", {})
            primary_accent = colors.get("accent_primary", "").lower()
            secondary_accent = colors.get("accent_secondary", "").lower()

            # Check primary or secondary accent uses a Fluent Blue tone or blue hex starting with #0
            is_fluent_blue = (
                any(shade in primary_accent or shade in secondary_accent for shade in fluent_blue_shades) or
                primary_accent.startswith("#007") or primary_accent.startswith("#005") or primary_accent.startswith("#0f6")
            )
            self.assertTrue(
                is_fluent_blue,
                f"Theme {theme_name} accent colors ({primary_accent}, {secondary_accent}) do not include Fluent Blue accents"
            )


if __name__ == "__main__":
    unittest.main()
