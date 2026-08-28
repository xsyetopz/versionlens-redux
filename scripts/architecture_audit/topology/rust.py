"""Source-aware Rust external-module topology extraction."""

from __future__ import annotations

from ast import literal_eval
import re
from collections.abc import Sequence
from pathlib import Path

from ..discovery import iter_audited_files


def children_reachable(
    parent: Path,
    children: Sequence[Path],
    root: Path,
    inventory: Sequence[Path] | None,
) -> bool:
    visible = tuple(inventory or iter_audited_files(root))
    rust_files = {
        path.resolve()
        for path in visible
        if path.suffix.lower() == ".rs" and path.is_file()
    }
    if not rust_files:
        return False
    owners = {
        candidate.resolve()
        for candidate in (
            parent / "lib.rs",
            parent / "main.rs",
            parent / "mod.rs",
            parent.with_suffix(".rs"),
        )
        if candidate.is_file() and candidate.resolve() in rust_files
    }
    if not owners:
        return False
    reachable: set[Path] = set()
    pending = list(owners)
    while pending:
        declaring = pending.pop()
        if declaring in reachable:
            continue
        reachable.add(declaring)
        try:
            source = declaring.read_text(encoding="utf-8")
        except OSError:
            continue
        for name, path_override in external_modules(source):
            resolved = next(
                (
                    candidate.resolve()
                    for candidate in module_candidates(
                        declaring, parent, name, path_override
                    )
                    if candidate.is_file() and candidate.resolve() in rust_files
                ),
                None,
            )
            if resolved is not None:
                pending.append(resolved)
    return {child.resolve() for child in children} <= reachable


def tokens(source: str) -> list[tuple[str, str]]:
    """Tokenize enough Rust syntax to ignore comments and string contents."""

    result: list[tuple[str, str]] = []
    index = 0
    length = len(source)
    while index < length:
        if source.startswith("//", index):
            newline = source.find("\n", index + 2)
            index = length if newline < 0 else newline + 1
            continue
        if source.startswith("/*", index):
            depth = 1
            index += 2
            while index < length and depth:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            continue
        if source[index].isspace():
            index += 1
            continue
        start = index
        raw = re.match(r"(?:br|r)#*\"", source[index:])
        if raw:
            opener = raw.group(0)
            hashes = opener.count("#")
            closing = '"' + ("#" * hashes)
            end = source.find(closing, index + len(opener))
            index = length if end < 0 else end + len(closing)
            result.append(("literal", source[start:index]))
            continue
        if source[index] == "'":
            end = index + 1
            escaped = False
            while end < length and source[end] != "\n":
                if not escaped and source[end] == "'":
                    break
                escaped = not escaped and source[end] == "\\"
                if source[end] != "\\":
                    escaped = False
                end += 1
            if end < length and source[end] == "'":
                result.append(("literal", source[start : end + 1]))
                index = end + 1
                continue
        if source[index] == '"' or (
            source[index] in "bBcC"
            and index + 1 < length
            and source[index + 1] == '"'
        ):
            if source[index] in "bBcC":
                index += 1
            quote = source[index]
            index += 1
            while index < length:
                escaped = source[index] == "\\"
                index += 2 if escaped else 1
                if not escaped and source[index - 1] == quote:
                    break
            result.append(("literal", source[start:index]))
            continue
        identifier = re.match(r"[A-Za-z_][A-Za-z0-9_]*", source[index:])
        if identifier:
            value = identifier.group(0)
            result.append(("ident", value))
            index += len(value)
            continue
        result.append(("punct", source[index]))
        index += 1
    return result


def external_modules(source: str) -> list[tuple[str, str | None]]:
    parsed = tokens(source)
    declarations: list[tuple[str, str | None]] = []
    pending_path: str | None = None
    index = 0
    while index < len(parsed):
        if (
            index + 4 < len(parsed)
            and parsed[index : index + 2] == [("punct", "#"), ("punct", "[")]
            and parsed[index + 2] == ("ident", "path")
            and parsed[index + 3] == ("punct", "=")
            and parsed[index + 4][0] == "literal"
        ):
            pending_path = literal_value(parsed[index + 4][1])
            index += 5
            while index < len(parsed) and parsed[index][1] != "]":
                index += 1
            index += 1
            continue
        if (
            parsed[index] == ("ident", "mod")
            and index + 2 < len(parsed)
            and parsed[index + 1][0] == "ident"
        ):
            name = parsed[index + 1][1]
            terminator = parsed[index + 2][1]
            if terminator == ";":
                declarations.append((name, pending_path))
                pending_path = None
            elif terminator == "{":
                pending_path = None
            index += 3
            continue
        index += 1
    return declarations


def literal_value(value: str) -> str:
    if value.startswith("r") or value.startswith("br"):
        quote = value.find('"')
        hashes = value[quote + 1 :].split('"', 1)[0].count("#")
        prefix = quote + 1
        return value[prefix : len(value) - hashes - 1]
    try:
        parsed = literal_eval(value)
    except (SyntaxError, ValueError):
        return value[1:-1]
    return parsed.decode() if isinstance(parsed, bytes) else str(parsed)


def module_candidates(
    declaring: Path,
    parent: Path,
    name: str,
    path_override: str | None,
) -> tuple[Path, ...]:
    if path_override:
        explicit = declaring.parent / path_override
        return (explicit, explicit.with_suffix(".rs"), explicit / "mod.rs")
    if declaring.resolve() == parent.with_suffix(".rs").resolve():
        base = parent
    elif declaring.name in {"lib.rs", "main.rs", "mod.rs"}:
        base = declaring.parent
    else:
        base = declaring.parent / declaring.stem
    return (base / f"{name}.rs", base / name / "mod.rs")
