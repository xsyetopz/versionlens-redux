#!/usr/bin/env python3
"""Shared fixtures for architecture-audit regression modules."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from architecture_audit.audit import audit
from architecture_audit.inline_tests import inline_test_findings
from architecture_audit.records import Finding
from architecture_audit.rules import (
    FLAT_CLUSTER_LIMIT,
    HARD_LINE_THRESHOLD,
    SOFT_LINE_THRESHOLD,
    STRONG_LINE_THRESHOLD,
)


class AuditFixture:
    """Reusable filesystem and CLI helpers without inherited test methods."""

    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write(self, relative: str, lines: int = 1, content: str | None = None) -> Path:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            content if content is not None else "x\n" * lines, encoding="utf-8"
        )
        return path

    def findings(
        self,
        *,
        soft: int = SOFT_LINE_THRESHOLD,
        strong: int = STRONG_LINE_THRESHOLD,
        hard: int = HARD_LINE_THRESHOLD,
        flat_limit: int = FLAT_CLUSTER_LIMIT,
        include_generated: bool = False,
    ) -> list[Finding]:
        return audit(
            self.root,
            soft=soft,
            strong=strong,
            hard=hard,
            flat_limit=flat_limit,
            include_generated=include_generated,
        )[1]

    @staticmethod
    def codes(findings: list[Finding]) -> set[str]:
        return {finding.code for finding in findings}

    def inline_findings(self, relative: str, content: str) -> list[Finding]:
        path = self.write(relative, content=content)
        return inline_test_findings(path, self.root)

    @staticmethod
    def git_run(root: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", "-C", str(root), *arguments],
            check=True,
            capture_output=True,
            text=True,
        )

    def init_git_case(self, root: Path, ignore: str | None = None):
        """Create the canonical temporary Git repository used by audit tests."""

        run = lambda *arguments: self.git_run(root, *arguments)
        run("init", "-q")
        run("config", "user.email", "audit@example.invalid")
        run("config", "user.name", "Architecture Audit")
        if ignore is not None:
            (root / ".gitignore").write_text(ignore, encoding="utf-8")
        return run

    def init_ignored_source_case(self, root: Path):
        """Create the common ignored-source inventory scenario."""

        run = self.init_git_case(root, "ignored/**\n")
        run("add", ".gitignore")
        run("commit", "-qm", "baseline")
        ignored = root / "ignored/source.ts"
        ignored.parent.mkdir()
        ignored.write_text("export const ignored = 1;\n", encoding="utf-8")
        return run

    def run_cli(
        self, *arguments: str, root: Path | None = None
    ) -> subprocess.CompletedProcess[str]:
        script = Path(__file__).parents[1].parent / "audit_architecture.py"
        target = root or self.root
        return subprocess.run(
            [sys.executable, str(script), str(target), *arguments],
            check=False,
            capture_output=True,
            text=True,
        )

    @staticmethod
    def json_output(result: subprocess.CompletedProcess[str]) -> dict[str, Any]:
        return json.loads(result.stdout)
