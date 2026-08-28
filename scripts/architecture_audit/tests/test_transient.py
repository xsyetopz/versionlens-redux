"""Transient metadata directory regression tests."""

from __future__ import annotations

import unittest

from architecture_audit.tests.test_support import AuditFixture


class TransientMetadataTests(AuditFixture, unittest.TestCase):
    def test_metadata_only_transient_directories_are_blocking(self) -> None:
        for name in (".gradle", "__pycache__", ".cache", "reports"):
            self.write(f"tests/fixtures/session/commands/sort/{name}/metadata", content="x\n")
        codes = self.codes(self.findings())
        self.assertIn("transient-metadata-directory", codes)

    def test_codegraph_is_preserved_as_repository_intelligence(self) -> None:
        self.write(".codegraph/metadata", content="owned\n")
        self.assertNotIn("transient-metadata-directory", self.codes(self.findings()))


if __name__ == "__main__":
    unittest.main()
