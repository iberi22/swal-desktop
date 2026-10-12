#!/usr/bin/env python3
"""Automated E2E test suite for SWAL Theme Engine and Schema Conformance."""
import json
import os
import shutil
import subprocess
import unittest

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _resolve(repo_relpath, home_path):
    """Prefer the checked-in repo copy so tests run on a bare CI checkout;

    fall back to the user install location (~/.config/swal/...) on a real desktop.
    """
    repo_path = os.path.join(PROJECT_ROOT, repo_relpath)
    if os.path.exists(repo_path):
        return repo_path
    return os.path.expanduser(home_path)


THEMES_DIR = _resolve("themes", "~/.config/swal/themes")
SCHEMA_FILE = _resolve(os.path.join("schemas", "theme.schema.json"),
                       "~/.config/swal/schemas/theme.schema.json")

NEEDS_SWAL_THEME = "swal-theme binary not installed (no user install in CI)"


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

    @unittest.skipIf(shutil.which("swal-theme") is None, NEEDS_SWAL_THEME)
    def test_cli_list(self):
        res = subprocess.run(["swal-theme", "list"], capture_output=True, text=True)
        self.assertEqual(res.returncode, 0)
        self.assertIn("hive-dark", res.stdout)
        self.assertIn("cyber-neon", res.stdout)

    @unittest.skipIf(shutil.which("swal-theme") is None, NEEDS_SWAL_THEME)
    def test_cli_switch(self):
        res = subprocess.run(["swal-theme", "switch", "hive-dark"], capture_output=True, text=True)
        self.assertEqual(res.returncode, 0)
        self.assertIn("successfully switched", res.stdout)


if __name__ == "__main__":
    unittest.main()
