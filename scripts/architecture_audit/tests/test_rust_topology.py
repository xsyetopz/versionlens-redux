"""Adversarial Rust module-topology tests."""

from __future__ import annotations

import unittest

from architecture_audit.findings import _has_module_declaration_evidence
from architecture_audit.tests.test_support import AuditFixture


class RustTopologyTests(AuditFixture, unittest.TestCase):
    def evidence(self, content: str, names: tuple[str, ...]) -> bool:
        owner = self.write("src/lib.rs", content=content)
        children = [self.write(f"src/{name}.rs") for name in names]
        inventory = tuple([owner, *children])
        return _has_module_declaration_evidence(
            self.root / "src", children, self.root, inventory
        )

    def test_declared_external_modules_are_reachable(self) -> None:
        self.assertTrue(self.evidence("mod alpha;\nmod beta;\n", ("alpha", "beta")))

    def test_partial_declaration_does_not_cover_undeclared_file(self) -> None:
        self.assertFalse(self.evidence("mod alpha;\n", ("alpha", "beta")))

    def test_comment_and_string_mentions_are_not_modules(self) -> None:
        self.assertFalse(
            self.evidence(
                '// mod alpha;\nconst text = "mod beta;";\n',
                ("alpha", "beta"),
            )
        )

    def test_path_attribute_resolves_external_module(self) -> None:
        owner = self.write('src/lib.rs', content='#[path = "external.rs"] mod alpha;\n')
        external = self.write("src/external.rs")
        self.assertTrue(
            _has_module_declaration_evidence(
                self.root / "src", [external], self.root, (owner, external)
            )
        )

    def test_filename_mention_in_inventory_is_not_reachability(self) -> None:
        owner = self.write("src/lib.rs", content="// no module declarations\n")
        child = self.write("src/alpha.rs")
        inventory_note = self.write("docs/inventory.txt", content="alpha.rs\n")
        self.assertFalse(
            _has_module_declaration_evidence(
                self.root / "src", [child], self.root, (owner, child, inventory_note)
            )
        )

    def test_nested_module_layout_is_resolved(self) -> None:
        root = self.write("src/lib.rs", content="mod outer;\n")
        outer = self.write("src/outer/mod.rs", content="mod inner;\n")
        inner = self.write("src/outer/inner.rs")
        self.assertTrue(
            _has_module_declaration_evidence(
                self.root / "src/outer", [outer, inner], self.root, (root, outer, inner)
            )
        )


if __name__ == "__main__":
    unittest.main()
