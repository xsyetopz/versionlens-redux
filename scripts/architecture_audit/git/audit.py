#!/usr/bin/env python3
"""Git working-tree suppression and destructive-diff evidence."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from . import diff as _diff
from . import inventory as _inventory
from ..config_policy import parse_jsonc, policy_equivalent_or_stronger
from .. import suppression_rules as _rules
from ..discovery import (
    GitInventoryError,
    _contains_literal_reference,
    _git_inventory,
    git_repository_root,
)
from ..records import Finding


def git_suppression_findings(root: Path) -> list[Finding]:
    """Report suppression and destructive-evidence changes in a Git worktree.

    Git's committed tree is the baseline implicitly: unchanged files are never
    inspected here.  Both index-vs-HEAD and worktree-vs-index diffs are parsed,
    while non-ignored untracked ignore files are inspected explicitly because
    Git does not include them in either diff.  Outside a Git worktree this is a
    no-op.
    """

    try:
        repository = git_repository_root(root)
    except GitInventoryError as exc:
        return [
            Finding("error", "git-suppression-scan-failed", root, str(exc), "tooling")
        ]
    if repository is None:
        return []
    findings: list[Finding] = []
    seen: set[tuple[str, str, int, str]] = set()

    def add(code: str, path: Path, line_number: int, message: str) -> None:
        key = (code, str(path), line_number, message)
        if key in seen:
            return
        seen.add(key)
        findings.append(_rules._finding(path, line_number, message, code))

    try:
        inventory = _git_inventory(repository, repository)
        inventory_contents = _inventory_contents(inventory)
        fixture_candidates = _fixture_candidates(inventory)
        events = _diff._git_diff_events(
            _diff._run_git_diff(repository, head=True), repository
        )
        high_confidence = _diff._git_rename_pairs(
            _diff._run_git_rename_inventory(repository, head=True), repository
        )
        accepted_pairs = {
            pair
            for pair in high_confidence
            if _diff._is_destructive_path(pair[0], repository)
            and _diff._is_destructive_path(pair[1], repository)
            and _rename_consumers_migrated(
                pair, repository, inventory, inventory_contents
            )
        }
    except GitInventoryError as exc:
        return [
            Finding("error", "git-suppression-scan-failed", root, str(exc), "tooling")
        ]
    try:
        audit_root = root.resolve()
    except OSError:
        audit_root = root
    pairs = accepted_pairs
    rename_sources = {old for old, _ in pairs}
    added_text_by_path: dict[Path, str] = {}
    for event in events:
        if event.kind == "added":
            added_text_by_path[event.path] = (
                f"{added_text_by_path.get(event.path, '')}\n{event.text}"
            )
    for event in events:
        try:
            event.path.resolve().relative_to(audit_root)
        except ValueError:
            continue
        if event.kind == "deleted":
            if (
                _diff._is_destructive_path(event.path, repository)
                and event.path not in rename_sources
                and not _has_accepted_deletion(
                    event.path,
                    repository,
                    inventory,
                    inventory_contents,
                    fixture_candidates,
                )
            ):
                add(
                    "check-file-deleted",
                    event.path,
                    event.line_number,
                    f"test/check/lint/CI file was deleted from the working tree: {event.path.name}",
                )
            continue
        if event.kind == "added":
            if _rules._is_ignore_file(
                event.path
            ) and not _rules._is_comment_or_blank(event.text):
                add(
                    "ignore-pattern-added",
                    event.path,
                    event.line_number,
                    f"lint/check ignore pattern was added: {event.text.strip()}",
                )
            elif (
                event.path.name.lower() == ".gitignore"
                and _rules._is_relevant_gitignore_pattern(event.text)
            ):
                add(
                    "gitignore-source-pattern-added",
                    event.path,
                    event.line_number,
                    f"source/test/check/lint path was newly ignored: {event.text.strip()}",
                )
        elif (
            event.kind == "removed"
            and event.path not in rename_sources
            and _diff._is_script_path(event.path)
            and _diff._provider_invocation(event.text)
            and not _provider_invocations_preserved(
                event.text, added_text_by_path.get(event.path, "")
            )
        ):
            add(
                "check-provider-removed",
                event.path,
                event.line_number,
                f"lint/check/test provider invocation was removed: {event.text.strip()}",
            )

    for path, line_number, text in _diff._provider_removals_for_rename(
        repository, head=True, pairs=pairs
    ):
        add(
            "check-provider-removed",
            path,
            line_number,
            f"lint/check/test provider invocation was removed: {text.strip()}",
        )

    try:
        untracked_candidates = _inventory._untracked_candidates(repository, root)
    except GitInventoryError as exc:
        return [
            Finding("error", "git-suppression-scan-failed", root, str(exc), "tooling")
        ]
    for path in untracked_candidates:
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except (OSError, UnicodeError) as exc:
            add(
                "suppression-scan-failed",
                path,
                1,
                f"Git ignore candidate scan failed: {exc}",
            )
            continue
        for line_number, line in enumerate(lines, 1):
            if path.name.lower() == ".gitignore":
                relevant = _rules._is_relevant_gitignore_pattern(line)
                code = "gitignore-source-pattern-added"
                message = f"source/test/check/lint path was newly ignored: {line.strip()}"
            else:
                relevant = not _rules._is_comment_or_blank(line)
                code = "ignore-pattern-added"
                message = f"lint/check ignore pattern was added: {line.strip()}"
            if relevant:
                add(
                    code,
                    path,
                    line_number,
                    message,
                )
    return findings


def _provider_invocations_preserved(removed: str, added: str) -> bool:
    """Accept a script-line rewrite when every provider call remains present."""

    invocations = [
        match.group(0) for match in _rules._PROVIDER_INVOCATION.finditer(removed)
    ]
    return bool(invocations) and all(invocation in added for invocation in invocations)


def _has_accepted_deletion(
    path: Path,
    repository: Path,
    inventory: set[Path],
    inventory_contents: dict[Path, str],
    fixture_candidates: dict[str, tuple[Path, ...]],
) -> bool:
    return (
        _has_semantic_config_replacement(path, repository)
        or _has_verified_fixture_replacement(
            path,
            repository,
            inventory,
            inventory_contents,
            fixture_candidates,
        )
        or _has_proven_orphan(path, repository, inventory, inventory_contents)
    )


def _has_verified_fixture_replacement(
    path: Path,
    repository: Path,
    inventory: set[Path],
    inventory_contents: dict[Path, str],
    fixture_candidates: dict[str, tuple[Path, ...]],
) -> bool:
    """Recognize a fixture consolidation backed by retained fixture content.

    A deleted fixture is safe only when a current same-name fixture is retained
    and a live test consumer still references that one canonical fixture. The
    consumer graph, rather than byte similarity, proves that a scenario was
    consolidated without silently deleting its test input.
    """

    if "fixtures" not in {part.lower() for part in path.parts}:
        return False
    for candidate in fixture_candidates.get(path.name, ()):
        if candidate == path or not candidate.is_file():
            continue
        if "fixtures" not in {part.lower() for part in candidate.parts}:
            continue
        if candidate not in inventory_contents:
            continue
        if _has_current_literal_consumer(candidate, repository, inventory, inventory_contents):
            return True
    return False


def _has_current_literal_consumer(
    path: Path,
    repository: Path,
    inventory: set[Path],
    inventory_contents: dict[Path, str],
) -> bool:
    references = _path_references(path, repository)
    for candidate in inventory:
        if candidate == path or "fixtures" in {part.lower() for part in candidate.parts}:
            continue
        content = inventory_contents.get(candidate, "")
        if _contains_reference(content, references):
            return True
    return False


def _has_proven_orphan(
    path: Path,
    repository: Path,
    inventory: set[Path],
    inventory_contents: dict[Path, str],
) -> bool:
    # A lint/check configuration with a sibling replacement is not an orphan:
    # its effective policy must be proven by the structured replacement check.
    if _rules._is_linter_config(path):
        try:
            if any(
                candidate.parent == path.parent
                and candidate != path
                and _rules._is_linter_config(candidate)
                for candidate in inventory
            ) or any(
                candidate.parent == path.parent
                and candidate != path
                and candidate.is_file()
                and candidate.suffix.lower() in {".json", ".jsonc"}
                for candidate in path.parent.iterdir()
            ):
                return False
        except OSError:
            return False
    references = _path_references(path, repository)
    for candidate in inventory:
        content = inventory_contents.get(candidate, "")
        if _contains_reference(content, references):
            return False
    return True


def _path_references(path: Path, repository: Path) -> tuple[str, ...]:
    relative = path.relative_to(repository).as_posix()
    without_suffix = relative[: -len(path.suffix)] if path.suffix else relative
    dotted = without_suffix.replace("/", ".")
    return tuple(dict.fromkeys((relative, without_suffix, dotted, path.name)))



_contains_reference = _contains_literal_reference


def _inventory_contents(inventory: set[Path]) -> dict[Path, str]:
    contents: dict[Path, str] = {}
    for path in inventory:
        try:
            contents[path] = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
    return contents


def _fixture_candidates(inventory: set[Path]) -> dict[str, tuple[Path, ...]]:
    grouped: dict[str, list[Path]] = {}
    for path in inventory:
        if "fixtures" not in {part.lower() for part in path.parts}:
            continue
        grouped.setdefault(path.name, []).append(path)
    return {name: tuple(paths) for name, paths in grouped.items()}


def _has_semantic_config_replacement(path: Path, repository: Path) -> bool:
    if path.name.lower() != "biome.json" or path.suffix.lower() != ".json":
        return False
    replacement = path.with_suffix(".jsonc")
    if not replacement.is_file():
        return False
    try:
        relative = path.relative_to(repository).as_posix()
        old = subprocess.run(
            ["git", "-C", str(repository), "show", f"HEAD:{relative}"],
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
        if old.returncode != 0:
            return False
        old_value = parse_jsonc(old.stdout)
        new_value = parse_jsonc(replacement.read_text(encoding="utf-8"))
    except (OSError, subprocess.SubprocessError):
        return False
    if old_value is None or new_value is None:
        return False
    return policy_equivalent_or_stronger(old_value, new_value)


def _rename_consumers_migrated(
    pair: tuple[Path, Path],
    repository: Path,
    inventory: set[Path],
    inventory_contents: dict[Path, str],
) -> bool:
    old, new = pair
    old_relative = old.relative_to(repository).as_posix()
    for candidate in inventory:
        if candidate == new or "fixtures" in {part.lower() for part in candidate.parts}:
            continue
        content = inventory_contents.get(candidate, "")
        if _contains_reference(content, _path_references(old, repository)):
            return False
    return new.is_file()
