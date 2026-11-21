import json
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRESENTATION_CASES = {
    "src/ui/new_panel.rs": "\u2713",
    "ui/src/components/NewPanel.tsx": "\u2026",
    "webcomponents/src/components/NewPanel.tsx": "\u2600",
    "webcomponents/stories/NewPanel.stories.tsx": "\u25c6",
    "vscode-extension/webview-ui/components/NewPanel.tsx": "\u2715",
    "vscode-extension/src/sections/new-section.ts": "\u00b7",
    "src/docs_gen/new_generator.rs": "\u2192",
    "docs/new-page.md": "\u00a7",
    "README.md": "\u2500",
    "ui/README.md": "\u2500",
    "docs/README.md": "\u2500",
    "ui/src/README.md": "\u2500",
    "webcomponents/src/README.md": "\u2500",
    "vscode-extension/webview-ui/components/README.md": "\u2500",
    "tests/new_report.rs": "\u2713",
}


class UnicodePolicyTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="operator-unicode-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        for name in ("unicleaner.toml", ".ignore"):
            (self.root / name).write_bytes((ROOT / name).read_bytes())

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def scan(self, path=".", config="unicleaner.toml"):
        result = subprocess.run(
            ["unicleaner", "scan", path, "--config", config, "--format", "json"],
            cwd=self.root, capture_output=True, text=True, check=False,
        )
        return result

    def test_presentation_directories_and_relative_paths(self):
        for path, character in PRESENTATION_CASES.items():
            self.write(path, character)
            for spelling in (path, "./" + path):
                with self.subTest(path=spelling):
                    result = self.scan(spelling)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_unlisted_and_invisible_characters_rejected(self):
        for path in (*PRESENTATION_CASES, "src/core.rs"):
            for character in ("\u03a9", "\u200b", "\u202e"):
                with self.subTest(path=path, codepoint=ord(character)):
                    self.write(path, character)
                    result = self.scan(path)
                    self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                    findings = json.loads(result.stdout)["violations"]
                    self.assertIn(ord(character), [v["code_point"] for v in findings])

    def test_test_inputs_stay_narrow_and_overrides_keep_display_glyphs(self):
        cases = [
            ("src/ui/masked_input.rs", "\u00e9\U0001f510\u2713", 0),
            ("ui/src/naiveasync.d.ts", "\U0001f501\u2026", 0),
            ("src/queue/ids.rs", "\u00e9\u65e5", 0),
            ("src/ui/new_panel.rs", "\u00e9", 1),
            ("ui/src/new.ts", "\U0001f501", 1),
            ("src/core.rs", "\u2713", 1),
            ("src/queue/new.rs", "\u00e9", 1),
        ]
        for path, content, status in cases:
            with self.subTest(path=path):
                self.write(path, content)
                result = self.scan(path)
                self.assertEqual(result.returncode, status, result.stdout + result.stderr)

    def test_directory_exclusions(self):
        for path in ("AGENTS.md", "openapi.json", "nested/schema.json", "image.png",
                     "font.ttf", "font.woff", "bindings/example.ts", "shared/types.ts",
                     ".github/workflows/example.yml", "nested/.hidden/example.rs"):
            self.write(path, "\u200b\u202e")
        self.write("src/core.rs", "ASCII\n")
        result = self.scan()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(json.loads(result.stdout)["errors"])
        self.write("src/core.rs", "\u200b")
        self.assertEqual(self.scan().returncode, 1)

    def test_invalid_config_and_missing_input_fail(self):
        self.write("broken.toml", "[invalid")
        self.assertNotEqual(self.scan(config="broken.toml").returncode, 0)
        self.assertNotEqual(self.scan("missing.rs").returncode, 0)


if __name__ == "__main__":
    unittest.main()
