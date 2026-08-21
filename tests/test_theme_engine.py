#!/usr/bin/env python3
"""Automated E2E test suite for SWAL Theme Engine and Schema Conformance."""
import json
import os
import subprocess
import unittest

THEMES_DIR = os.path.expanduser("~/.config/swal/themes")
SCHEMA_FILE = os.path.expanduser("~/.config/swal/schemas/theme.schema.json")


class TestSWALThemeEngine(unittest.TestCase):

    def test_theme_schema_exists(self):
        self.assertTrue(os.path.exists(SCHEMA_FILE), "theme.schema.json must exist")

    def test_themes_exist(self):
        self.assertTrue(os.path.exists(os.path.join(THEMES_DIR, "hive-dark.json")))
        self.assertTrue(os.path.exists(os.path.join(THEMES_DIR, "cyber-neon.json")))
        self.assertTrue(os.path.exists(os.path.join(THEMES_DIR, "nord-swal.json")))

    def test_theme_keys(self):
        with open(os.path.join(THEMES_DIR, "hive-dark.json")) as f:
            data = json.load(f)
            self.assertEqual(data["id"], "hive-dark")
            self.assertIn("colors", data)
            self.assertIn("accent_primary", data["colors"])
            self.assertEqual(data["colors"]["accent_primary"], "#06b6d4")

    def test_cli_list(self):
        res = subprocess.run(["swal-theme", "list"], capture_output=True, text=True)
        self.assertEqual(res.returncode, 0)
        self.assertIn("hive-dark", res.stdout)
        self.assertIn("cyber-neon", res.stdout)

    def test_cli_switch(self):
        res = subprocess.run(["swal-theme", "switch", "hive-dark"], capture_output=True, text=True)
        self.assertEqual(res.returncode, 0)
        self.assertIn("successfully switched", res.stdout)


if __name__ == "__main__":
    unittest.main()
