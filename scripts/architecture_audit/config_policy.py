"""Small, fail-closed readers for structured lint configuration policy."""

from __future__ import annotations

import json
import re
from collections.abc import Mapping
from typing import Any


_SEVERITIES = {"off": 0, "info": 1, "warn": 2, "warning": 2, "error": 3}


def parse_jsonc(text: str) -> dict[str, Any] | None:
    """Parse JSON/JSONC without interpreting comment markers in strings."""

    stripped = _strip_comments(text)
    stripped = re.sub(r",\s*([}\]])", r"\1", stripped)
    try:
        value = json.loads(stripped)
    except json.JSONDecodeError:
        return None
    return value if isinstance(value, dict) else None


def severity_rank(value: object) -> int | None:
    """Return a linter severity rank, or ``None`` for an unknown shape."""

    if isinstance(value, list):
        value = value[0] if value else None
    if isinstance(value, bool):
        return 3 if value else 0
    if isinstance(value, int) and value in range(4):
        return value
    if isinstance(value, str):
        return _SEVERITIES.get(value.lower())
    return None


def rule_severities(config: Mapping[str, Any]) -> dict[str, int]:
    """Collect explicit rule severities from common linter config locations."""

    result: dict[str, int] = {}

    def collect(value: object) -> None:
        if not isinstance(value, Mapping):
            return
        rules = value.get("rules")
        if isinstance(rules, Mapping):
            _collect_rule_map(rules, result)
        overrides = value.get("overrides")
        if isinstance(overrides, list):
            for override in overrides:
                collect(override)

    linter = config.get("linter")
    if isinstance(linter, Mapping):
        collect(linter)
    collect(config)
    return result


def policy_downgrades(
    previous: Mapping[str, Any], current: Mapping[str, Any]
) -> dict[str, tuple[int, int]]:
    """Return only explicit error/warn/info/off severity decreases."""

    before = rule_severities(previous)
    after = rule_severities(current)
    return {
        name: (before[name], after[name])
        for name in before.keys() & after.keys()
        if before[name] > after[name]
    }


def policy_equivalent_or_stronger(
    previous: Mapping[str, Any], current: Mapping[str, Any]
) -> bool:
    """Prove a structured replacement retains all explicit policy strength."""

    if policy_downgrades(previous, current):
        return False
    for key in ("formatter", "files", "organizeImports", "assist"):
        if key in previous and key not in current:
            return False
    previous_linter = previous.get("linter")
    current_linter = current.get("linter")
    if isinstance(previous_linter, Mapping):
        if not isinstance(current_linter, Mapping):
            return False
        for key in ("enabled", "rules", "includes", "excludes"):
            if key in previous_linter and key not in current_linter:
                return False
    previous_presets = _presets(previous)
    current_presets = _presets(current)
    if not previous_presets <= current_presets:
        # A preset can be replaced by a complete explicit rule map.  This is
        # a proof obligation, not a waiver: the replacement must contain a
        # real policy and be at least as specific as the old configuration.
        if not rule_severities(current) or len(rule_severities(current)) <= len(
            rule_severities(previous)
        ):
            return False
    previous_domains = _domains(previous)
    current_domains = _domains(current)
    if previous_domains and not previous_domains <= current_domains:
        if not rule_severities(current):
            return False
    return True


def _presets(config: Mapping[str, Any]) -> set[str]:
    presets: set[str] = set()
    for owner in (config, config.get("linter")):
        if not isinstance(owner, Mapping):
            continue
        value = owner.get("recommended")
        if isinstance(value, str):
            presets.add(value)
        elif isinstance(value, bool) and value:
            presets.add("recommended")
        value = owner.get("presets")
        if isinstance(value, list):
            presets.update(item for item in value if isinstance(item, str))
        rules = owner.get("rules")
        if isinstance(rules, Mapping) and isinstance(rules.get("preset"), str):
            presets.add(rules["preset"])
    return presets


def _domains(config: Mapping[str, Any]) -> set[str]:
    domains: set[str] = set()
    for owner in (config, config.get("linter")):
        if not isinstance(owner, Mapping):
            continue
        value = owner.get("domains")
        if isinstance(value, Mapping):
            domains.update(str(key) for key in value)
        elif isinstance(value, list):
            domains.update(item for item in value if isinstance(item, str))
    return domains


def _collect_rule_map(value: Mapping[str, Any], result: dict[str, int]) -> None:
    for name, setting in value.items():
        if not isinstance(name, str):
            continue
        rank = severity_rank(setting)
        if rank is not None:
            result[name] = rank
        elif isinstance(setting, Mapping):
            _collect_rule_map(setting, result)


def _strip_comments(text: str) -> str:
    output: list[str] = []
    index = 0
    quote = False
    escaped = False
    while index < len(text):
        char = text[index]
        if quote:
            output.append(char)
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quote = False
            index += 1
            continue
        if char == '"':
            quote = True
            output.append(char)
            index += 1
        elif text.startswith("//", index):
            newline = text.find("\n", index)
            if newline < 0:
                break
            output.append("\n")
            index = newline + 1
        elif text.startswith("/*", index):
            end = text.find("*/", index + 2)
            if end < 0:
                return ""
            output.append("\n" * text[index : end + 2].count("\n"))
            index = end + 2
        else:
            output.append(char)
            index += 1
    return "".join(output)
