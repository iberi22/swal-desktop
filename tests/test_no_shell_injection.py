#!/usr/bin/env python3
"""Security regression tests: no shell=True anywhere in shipped Python scripts.

Guards against command injection like the Omarchy-class vulnerabilities:
any subprocess call must use argv lists, never shell string interpolation.
"""
import ast
import pathlib
import unittest

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
SCRIPT_DIRS = ["eww/scripts", "scripts"]


def find_shell_true(py_path):
    """Yields line numbers where shell=True appears in a Python file."""
    tree = ast.parse(py_path.read_text(encoding="utf-8"))
    for node in ast.walk(tree):
        if isinstance(node, ast.keyword) and node.arg == "shell":
            if isinstance(node.value, ast.Constant) and node.value.value is True:
                yield node.lineno


class TestNoShellInjection(unittest.TestCase):
    def test_no_shell_true_in_scripts(self):
        offenders = []
        for d in SCRIPT_DIRS:
            base = REPO_ROOT / d
            if not base.exists():
                continue
            for py in sorted(base.rglob("*.py")):
                try:
                    for lineno in find_shell_true(py):
                        offenders.append(f"{py.relative_to(REPO_ROOT)}:{lineno}")
                except SyntaxError as e:
                    offenders.append(f"{py}: SYNTAX ERROR {e}")
        self.assertEqual(offenders, [], f"shell=True prohibido en: {offenders}")

    def test_orb_menu_uses_argv_lists(self):
        src = (REPO_ROOT / "eww" / "scripts" / "hermes_orb_menu.py").read_text()
        self.assertNotIn("shell=True", src)
        # The dispatcher must build an argument list, not an f-string command
        self.assertIn('["ghostty", "-e", "hermes", "--prompt", prompt]', src)


if __name__ == "__main__":
    unittest.main(verbosity=2)
