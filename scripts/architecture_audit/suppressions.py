#!/usr/bin/env python3
"""Fail-closed scanning of lint and check suppression constructs."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from .config_policy import parse_jsonc, policy_downgrades
from .records import Finding
from .suppression_rules import (
    _CHECK_BYPASS,
    _CHECK_EXIT_ZERO,
    _CI_BYPASS,
    _CI_CONTEXT,
    _CI_DISABLED,
    _CODE_SUPPRESSIONS,
    _COMMENT_DIRECTIVES,
    _DISABLED_RULE,
    _DOCUMENTATION_SUFFIXES,
    _DOWNGRADED_RULE,
    _PACKAGE_BYPASS,
    _RUST_ALLOW,
    _code_without_strings_or_comments,
    _comment_fragment,
    _config_suppression,
    _finding,
    _is_comment_or_blank,
    _is_ignore_file,
    _is_linter_config,
    _is_relevant_gitignore_pattern,
    _is_workflow,
    _json_brace_delta,
    _multiline_disabled_rule,
)


def suppression_findings(path: Path, root: Path | None = None) -> list[Finding]:
    """Find suppression and check-bypass constructs in one candidate file."""

    # Broken or external symlink targets are not authored candidate contents;
    # discovery keeps the link visible, while suppression scanning remains
    # content-based and avoids manufacturing a finding for missing targets.
    if path.is_symlink():
        return []
    suffix = path.suffix.lower()
    if suffix in _DOCUMENTATION_SUFFIXES:
        return []
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as exc:
        return [
            _finding(
                path, 1, f"suppression scan failed: {exc}", "suppression-scan-failed"
            )
        ]

    findings: list[Finding] = []
    workflow = _is_workflow(path)
    linter_config = _is_linter_config(path)
    package_manifest = path.name.lower() == "package.json"
    downgraded_lines = _historical_downgrade_lines(path, root) if linter_config and root else set()
    package_script_depth: int | None = None
    section = ""
    if path.name.lower() == ".gitignore":
        for line_number, line in enumerate(lines, 1):
            if _is_relevant_gitignore_pattern(line):
                findings.append(
                    _finding(
                        path,
                        line_number,
                        f"source/test/check/lint path is ignored: {line.strip()}",
                        "gitignore-source-pattern-added",
                    )
                )
    elif _is_ignore_file(path):
        for line_number, line in enumerate(lines, 1):
            if not _is_comment_or_blank(line):
                findings.append(
                    _finding(
                        path,
                        line_number,
                        f"lint/check ignore pattern is active: {line.strip()}",
                        "ignore-pattern-added",
                    )
                )
    if "baseline" in path.name.lower():
        for line_number, line in enumerate(lines, 1):
            if not _is_comment_or_blank(line):
                findings.append(
                    _finding(
                        path,
                        line_number,
                        f"baseline suppression configuration is active: {line.strip()}",
                        "baseline-suppression",
                    )
                )
                break
    for index, line in enumerate(lines):
        line_number = index + 1
        package_script_line = package_script_depth is not None
        if package_manifest:
            scripts_match = re.search(r"[\"']scripts[\"']\s*:", line, re.IGNORECASE)
            if scripts_match:
                package_script_line = True
                package_script_depth = max(
                    0, _json_brace_delta(line[scripts_match.end() :])
                )
            elif package_script_depth is not None:
                package_script_depth += _json_brace_delta(line)
                if package_script_depth <= 0:
                    package_script_depth = None
        section_match = re.match(r"^\s*\[([^\]]+)\]", line)
        if section_match:
            section = section_match.group(1).lower()
        comment = _comment_fragment(line, suffix)
        if comment is not None and any(
            token in comment.lower()
            for token in (
                "disable",
                "ignore",
                "expect-error",
                "nolint",
                "noqa",
                "nowarn",
            )
        ):
            for label, pattern in _COMMENT_DIRECTIVES:
                if pattern.search(comment):
                    findings.append(
                        _finding(
                            path,
                            line_number,
                            f"{label} suppression directive: {comment.strip()}",
                        )
                    )
                    break
        if suffix == ".rs" and "allow" in line and _RUST_ALLOW.search(line):
            findings.append(
                _finding(path, line_number, f"Rust lint allowance: {line.strip()}")
            )
        if (
            "pragma" in line.lower()
            or "suppress" in line.lower()
            or "nowarn" in line.lower()
        ):
            for label, pattern in _CODE_SUPPRESSIONS:
                if pattern.search(line):
                    findings.append(
                        _finding(
                            path, line_number, f"{label} suppression: {line.strip()}"
                        )
                    )
                    break
        code = _code_without_strings_or_comments(line, suffix)
        if "||" in code and _CHECK_BYPASS.search(code):
            findings.append(
                _finding(
                    path,
                    line_number,
                    f"lint/check/test command is forced successful with `|| true` or `|| :`: {line.strip()}",
                    "check-bypass",
                )
            )
        if "exit-zero" in code and _CHECK_EXIT_ZERO.search(code):
            findings.append(
                _finding(
                    path,
                    line_number,
                    f"lint/check provider is forced successful with `--exit-zero`: {line.strip()}",
                    "check-bypass",
                )
            )
        if package_manifest and package_script_line and _PACKAGE_BYPASS.search(line):
            findings.append(
                _finding(
                    path,
                    line_number,
                    f"package lint/check/test script is forced successful: {line.strip()}",
                    "check-bypass",
                )
            )
        if workflow and _CI_BYPASS.search(line):
            context = "\n".join(lines[max(0, index - 8) : index + 9])
            if _CI_CONTEXT.search(context):
                findings.append(
                    _finding(
                        path,
                        line_number,
                        f"CI lint/check/test step allows failure: {line.strip()}",
                        "check-bypass",
                    )
                )
        if workflow and _CI_DISABLED.search(line):
            context = "\n".join(lines[max(0, index - 8) : index + 9])
            if _CI_CONTEXT.search(context):
                findings.append(
                    _finding(
                        path,
                        line_number,
                        f"CI lint/check/test step is disabled: {line.strip()}",
                        "check-bypass",
                    )
                )
        # Linter rule names and severities are commonly quoted in JSON; use
        # the raw configuration line rather than the string-masked command
        # view above.
        lowered_line = line.lower()
        if (
            linter_config
            and not path.name.lower().startswith("tsconfig")
            and any(token in lowered_line for token in ("off", "false", ": 0", ":0"))
            and _DISABLED_RULE.search(line)
        ):
            findings.append(
                _finding(
                    path,
                    line_number,
                    f"linter rule severity is disabled: {line.strip()}",
                    "lint-severity-disabled",
                )
            )
        if (
            linter_config
            and not path.name.lower().startswith("tsconfig")
            and line_number in downgraded_lines
        ):
            findings.append(
                _finding(
                    path,
                    line_number,
                    f"linter rule severity is downgraded: {line.strip()}",
                    "lint-severity-downgraded",
                )
            )
        if linter_config:
            config_message = _config_suppression(path, line, section)
            if config_message is not None:
                findings.append(
                    _finding(
                        path,
                        line_number,
                        f"{config_message}: {line.strip()}",
                        "lint-config-suppression",
                    )
                )
            if _multiline_disabled_rule(lines, index):
                findings.append(
                    _finding(
                        path,
                        line_number,
                        f"linter rule severity is disabled on a continuation line: {line.strip()}",
                        "lint-severity-disabled",
                    )
                )
    return findings


def _historical_downgrade_lines(path: Path, root: Path) -> set[int]:
    """Return lines that changed a tracked rule from error to warning.

    A warning is a valid policy when it is the repository's established
    severity.  It becomes a degradation finding only when the current
    checkout provides transition evidence by changing a tracked rule that was
    previously an error.  This keeps the audit fail-closed for real policy
    weakening without treating every intentionally warning-level rule as a
    suppression.
    """

    try:
        relative = path.relative_to(root).as_posix()
    except ValueError:
        return set()
    try:
        previous = subprocess.run(
            ["git", "-C", str(root), "show", f"HEAD:{relative}"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    except (OSError, subprocess.CalledProcessError):
        return set()
    current_text = "\n".join(lines_for(path))
    previous_config = parse_jsonc("\n".join(previous))
    current_config = parse_jsonc(current_text)
    if previous_config is not None and current_config is not None:
        downgraded_rules = policy_downgrades(previous_config, current_config)
    else:
        downgraded_rules = {}
    if not downgraded_rules:
        return set()
    downgraded: set[int] = set()
    for line_number, line in enumerate(lines_for(path), 1):
        match = re.search(
            r"[\"']([^\"']+)[\"']\s*:\s*(?:\[\s*)?[\"']?(off|info|warn|warning)[\"']?",
            line,
            re.IGNORECASE,
        )
        if match and match.group(1) in downgraded_rules:
            downgraded.add(line_number)
    return downgraded


def lines_for(path: Path) -> list[str]:
    try:
        return path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError):
        return []
