#!/usr/bin/env python3

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
import sys

MAX_FIELDS = 10
SPLIT_HINT = (
    "split it on the same Rust/TypeScript side into smaller responsibility objects "
    "with fields distributed evenly; do not move overflow to the other language or a catch-all object"
)
ROOTS = [Path("crates"), Path("packages/vscode-extension/src"), Path("tests")]
IGNORED_DIRS = {"dist", "node_modules", "target"}

RUST_FIELD_PATTERN = re.compile(r"^(pub\([^)]*\)\s+|pub\s+)?[A-Za-z_][A-Za-z0-9_]*\s*:")
TYPESCRIPT_FIELD_PATTERN = re.compile(
    r"^(?:(?:public|private|protected|readonly|static|declare|override)\s+)*(#?[A-Za-z_$][\w$]*|\"[^\"]+\"|'[^']+')\??\s*[:=]"
)
TYPESCRIPT_PARAMETER_PROPERTY_PATTERN = re.compile(
    r"^(?:(?:public|private|protected|readonly|override)\s+)+#?[A-Za-z_$][\w$]*\??\s*[:=]"
)
BASE_NAME_SEPARATOR_PATTERN = re.compile(r"\W")
BASE_LIST_SEPARATOR_PATTERN = re.compile(r"[,&]")
RUST_STRUCT_PATTERN = re.compile(r"\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{")
RUST_TUPLE_STRUCT_PATTERN = re.compile(r"\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")
RUST_ENUM_PATTERN = re.compile(r"\benum\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{")
TYPESCRIPT_INTERFACE_PATTERN = re.compile(
    r"\binterface\s+([A-Za-z_$][\w$]*)(?:\s+extends\s+([^\{]+))?\s*\{"
)
TYPESCRIPT_CLASS_PATTERN = re.compile(r"\bclass\s+([A-Za-z_$][\w$]*)[^\{]*\{")
TYPESCRIPT_TYPE_ALIAS_PATTERN = re.compile(r"\btype\s+([A-Za-z_$][\w$]*)\s*=")
TYPESCRIPT_CONST_OBJECT_PATTERN = re.compile(r"\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)[^=]*=\s*\{")
TYPESCRIPT_REFERENCE_NAME_PATTERN = re.compile(r"^([A-Za-z_$][\w$]*)\b")
WHITESPACE_PATTERN = re.compile(r"\s+")
BLOCK_COMMENT_PATTERN = re.compile(r"/\*[\s\S]*?\*/")
LINE_COMMENT_PATTERN = re.compile(r"//.*$", re.MULTILINE)


@dataclass
class DepthState:
    brace_depth: int = 0
    bracket_depth: int = 0
    paren_depth: int = 0
    quote: str = ""


@dataclass
class ObjectInfo:
    fields: int
    line: int
    name: str


@dataclass
class InheritedObject:
    bases: list[str]
    object: ObjectInfo


@dataclass
class CollectedObjects:
    field_counts: dict[str, int]
    inherited: list[InheritedObject]
    objects: list[ObjectInfo]


def walk(root: Path) -> list[Path]:
    if not root.exists():
        return []
    files: list[Path] = []
    for entry in root.iterdir():
        if entry.name in IGNORED_DIRS:
            continue
        if entry.is_dir():
            files.extend(walk(entry))
        elif entry.suffix in {".rs", ".ts"}:
            files.append(entry)
    return files


def strip_comments(source: str) -> str:
    return LINE_COMMENT_PATTERN.sub("", BLOCK_COMMENT_PATTERN.sub("", source))


def is_quote(char: str) -> bool:
    return char in {'"', "'", "`"}


def track_quote(state: DepthState, char: str, previous: str) -> bool:
    if state.quote:
        if char == state.quote and previous != "\\":
            state.quote = ""
        return True
    if is_quote(char):
        state.quote = char
        return True
    return False


def update_depth(state: DepthState, char: str) -> None:
    if char == "{":
        state.brace_depth += 1
    elif char == "}":
        state.brace_depth -= 1
    elif char == "(":
        state.paren_depth += 1
    elif char == ")":
        state.paren_depth -= 1
    elif char == "[":
        state.bracket_depth += 1
    elif char == "]":
        state.bracket_depth -= 1


def is_top_level(state: DepthState) -> bool:
    return state.brace_depth == 0 and state.paren_depth == 0 and state.bracket_depth == 0


def find_matching_brace(source: str, open_index: int) -> int:
    state = DepthState()
    for index in range(open_index, len(source)):
        char = source[index]
        previous = source[index - 1] if index > 0 else ""
        if track_quote(state, char, previous):
            continue
        update_depth(state, char)
        if state.brace_depth == 0:
            return index
    return -1


def find_matching_paren(source: str, open_index: int) -> int:
    state = DepthState()
    for index in range(open_index, len(source)):
        char = source[index]
        previous = source[index - 1] if index > 0 else ""
        if track_quote(state, char, previous):
            continue
        update_depth(state, char)
        if state.paren_depth == 0:
            return index
    return -1


def find_type_alias_end(source: str, body_start: int) -> int:
    state = DepthState()
    for index in range(body_start, len(source)):
        char = source[index]
        previous = source[index - 1] if index > 0 else ""
        if track_quote(state, char, previous):
            continue
        if char == ";" and is_top_level(state):
            return index
        update_depth(state, char)
    return len(source)


def line_number(source: str, index: int) -> int:
    return source[:index].count("\n") + 1


def split_top_level_ranges(source: str, separators: set[str]) -> list[tuple[int, str]]:
    parts: list[tuple[int, str]] = []
    start = 0
    state = DepthState()
    for index, char in enumerate(source):
        previous = source[index - 1] if index > 0 else ""
        if track_quote(state, char, previous):
            continue
        if char in separators and is_top_level(state):
            parts.append((start, source[start:index]))
            start = index + 1
            continue
        update_depth(state, char)
    parts.append((start, source[start:]))
    return parts


def split_top_level(source: str, separators: set[str]) -> list[str]:
    return [part for _, part in split_top_level_ranges(source, separators)]


def top_level_fields(body: str, language: str) -> list[str]:
    clean = strip_comments(body)
    fields: list[str] = []
    for part in split_top_level(clean, {",", ";"}):
        trimmed = part.strip()
        if not trimmed:
            continue
        pattern = RUST_FIELD_PATTERN if language == "rust" else TYPESCRIPT_FIELD_PATTERN
        if pattern.search(trimmed):
            fields.append(trimmed)
    return fields


def count_typescript_constructor_parameter_fields(body: str) -> int:
    clean = strip_comments(body)
    state = DepthState()
    for index, char in enumerate(clean):
        previous = clean[index - 1] if index > 0 else ""
        if track_quote(state, char, previous):
            continue
        if is_top_level(state) and clean.startswith("constructor", index):
            open_index = clean.find("(", index)
            if open_index < 0:
                return 0
            close_index = find_matching_paren(clean, open_index)
            if close_index < 0:
                return 0
            return sum(
                1
                for field in split_top_level(clean[open_index + 1 : close_index], {","})
                if TYPESCRIPT_PARAMETER_PROPERTY_PATTERN.search(field.strip())
            )
        update_depth(state, char)
    return 0


def base_names(source: str) -> list[str]:
    names: list[str] = []
    for base in BASE_LIST_SEPARATOR_PATTERN.split(source):
        head = BASE_NAME_SEPARATOR_PATTERN.split(base.strip())[0]
        if head:
            names.append(head)
    return names


def collect_named_object_fields(source: str, language: str, patterns: list[re.Pattern[str]]) -> CollectedObjects:
    objects: list[ObjectInfo] = []
    inherited: list[InheritedObject] = []
    field_counts: dict[str, int] = {}
    for pattern in patterns:
        for match in pattern.finditer(source):
            open_index = source.find("{", match.start())
            close_index = find_matching_brace(source, open_index)
            if close_index < 0:
                continue
            fields = len(top_level_fields(source[open_index + 1 : close_index], language))
            if language == "typescript" and pattern is TYPESCRIPT_CLASS_PATTERN:
                fields += count_typescript_constructor_parameter_fields(source[open_index + 1 : close_index])
            name = match.group(1)
            field_counts[name] = fields
            obj = ObjectInfo(fields=fields, line=line_number(source, match.start()), name=name)
            objects.append(obj)
            if language == "typescript" and len(match.groups()) >= 2 and match.group(2):
                inherited.append(InheritedObject(bases=base_names(match.group(2)), object=obj))
    return CollectedObjects(field_counts=field_counts, inherited=inherited, objects=objects)


def type_reference_name(type_part: str) -> str | None:
    match = TYPESCRIPT_REFERENCE_NAME_PATTERN.search(type_part.strip())
    return match.group(1) if match else None


def top_level_object_literal_field_count(type_part: str) -> int:
    fields = 0
    state = DepthState()
    index = 0
    while index < len(type_part):
        char = type_part[index]
        previous = type_part[index - 1] if index > 0 else ""
        if track_quote(state, char, previous):
            index += 1
            continue
        if char == "{" and is_top_level(state):
            close_index = find_matching_brace(type_part, index)
            if close_index < 0:
                return fields
            fields += len(top_level_fields(type_part[index + 1 : close_index], "typescript"))
            index = close_index + 1
            continue
        update_depth(state, char)
        index += 1
    return fields


def type_alias_intersection_part_fields(type_part: str, field_counts: dict[str, int]) -> int:
    own_fields = top_level_object_literal_field_count(type_part)
    if own_fields > 0:
        return own_fields
    reference_name = type_reference_name(type_part)
    return field_counts.get(reference_name, 0) if reference_name else 0


def type_alias_variant_fields(variant: str, field_counts: dict[str, int]) -> int:
    return sum(type_alias_intersection_part_fields(part, field_counts) for part in split_top_level(variant, {"&"}))


def type_alias_object_name(alias_name: str, variant_index: int) -> str:
    return alias_name if variant_index == 1 else f"{alias_name}#{variant_index}"


def scan_type_alias_objects(source: str, field_counts: dict[str, int], name: str, body: str, body_start: int) -> tuple[int, list[ObjectInfo]]:
    objects: list[ObjectInfo] = []
    variant_index = 0
    max_fields = 0
    for start, variant in split_top_level_ranges(body, {"|"}):
        fields = type_alias_variant_fields(variant, field_counts)
        if fields == 0:
            continue
        variant_index += 1
        objects.append(
            ObjectInfo(fields=fields, line=line_number(source, body_start + start), name=type_alias_object_name(name, variant_index))
        )
        max_fields = max(max_fields, fields)
    return max_fields, objects


def collect_type_alias_objects(source: str, field_counts: dict[str, int]) -> list[ObjectInfo]:
    objects: list[ObjectInfo] = []
    for match in TYPESCRIPT_TYPE_ALIAS_PATTERN.finditer(source):
        body_start = source.find("=", match.start()) + 1
        body_end = find_type_alias_end(source, body_start)
        max_fields, scanned = scan_type_alias_objects(source, field_counts, match.group(1), source[body_start:body_end], body_start)
        objects.extend(scanned)
        if max_fields > 0:
            field_counts[match.group(1)] = max_fields
    return objects


def rust_enum_variant_object(source: str, enum_name: str, variant: str, variant_offset: int) -> ObjectInfo | None:
    named_open = variant.find("{")
    tuple_open = variant.find("(")
    if named_open >= 0 and (tuple_open < 0 or named_open < tuple_open):
        variant_open = named_open
    else:
        variant_open = tuple_open
    if variant_open < 0:
        return None
    name = WHITESPACE_PATTERN.split(variant[:variant_open].strip())[0]
    if not name:
        return None
    is_tuple = variant[variant_open] == "("
    variant_close = variant.rfind(")" if is_tuple else "}")
    if variant_close < variant_open:
        return None
    body = variant[variant_open + 1 : variant_close]
    fields = (
        sum(1 for field in split_top_level(body, {","}) if field.strip())
        if is_tuple
        else len(top_level_fields(body, "rust"))
    )
    return ObjectInfo(fields=fields, line=line_number(source, variant_offset), name=f"{enum_name}::{name}")


def collect_rust_enum_variant_objects(source: str) -> list[ObjectInfo]:
    objects: list[ObjectInfo] = []
    for match in RUST_ENUM_PATTERN.finditer(source):
        open_index = source.find("{", match.start())
        close_index = find_matching_brace(source, open_index)
        if close_index < 0:
            continue
        enum_body = source[open_index + 1 : close_index]
        for variant in split_top_level(strip_comments(enum_body), {","}):
            variant_offset = open_index + 1 + enum_body.find(variant)
            obj = rust_enum_variant_object(source, match.group(1), variant, variant_offset)
            if obj:
                objects.append(obj)
    return objects


def collect_rust_tuple_struct_objects(source: str) -> list[ObjectInfo]:
    objects: list[ObjectInfo] = []
    for match in RUST_TUPLE_STRUCT_PATTERN.finditer(source):
        open_index = source.find("(", match.start())
        close_index = find_matching_paren(source, open_index)
        if close_index < 0:
            continue
        fields = sum(
            1
            for field in split_top_level(strip_comments(source[open_index + 1 : close_index]), {","})
            if field.strip()
        )
        objects.append(ObjectInfo(fields=fields, line=line_number(source, match.start()), name=match.group(1)))
    return objects


def collect_typescript_const_object_literals(source: str) -> list[ObjectInfo]:
    objects: list[ObjectInfo] = []
    for match in TYPESCRIPT_CONST_OBJECT_PATTERN.finditer(source):
        open_index = source.find("{", match.start())
        close_index = find_matching_brace(source, open_index)
        if close_index < 0:
            continue
        fields = len(top_level_fields(source[open_index + 1 : close_index], "typescript"))
        if fields == 0:
            continue
        objects.append(ObjectInfo(fields=fields, line=line_number(source, match.start()), name=match.group(1)))
    return objects


def fields_from_bases(bases: list[str], field_counts: dict[str, int]) -> int:
    return sum(field_counts.get(base, 0) for base in bases)


def merge_field_counts(target: dict[str, int], source: dict[str, int]) -> bool:
    changed = False
    for name, fields in source.items():
        if name not in target or fields > target[name]:
            target[name] = fields
            changed = True
    return changed


def build_typescript_field_counts(files: list[Path]) -> dict[str, int]:
    field_counts: dict[str, int] = {}
    ts_files = [file for file in files if file.suffix == ".ts"]
    for file_path in ts_files:
        source = file_path.read_text()
        collected = collect_named_object_fields(source, "typescript", [TYPESCRIPT_INTERFACE_PATTERN, TYPESCRIPT_CLASS_PATTERN])
        merge_field_counts(field_counts, collected.field_counts)

    remaining_passes = len(ts_files)
    while remaining_passes > 0:
        remaining_passes -= 1
        changed = False
        for file_path in ts_files:
            source = file_path.read_text()
            before = dict(field_counts)
            collect_type_alias_objects(source, field_counts)
            changed = changed or before != field_counts
        if not changed:
            break
    return field_counts


def collect_objects(file_path: Path, global_ts_field_counts: dict[str, int], checked_files: dict[str, int]) -> list[ObjectInfo]:
    source = file_path.read_text()
    language = "rust" if file_path.suffix == ".rs" else "typescript"
    checked_files[language] += 1
    patterns = [RUST_STRUCT_PATTERN] if language == "rust" else [TYPESCRIPT_INTERFACE_PATTERN, TYPESCRIPT_CLASS_PATTERN]
    collected = collect_named_object_fields(source, language, patterns)
    if language == "typescript":
        field_counts = dict(global_ts_field_counts)
        merge_field_counts(field_counts, collected.field_counts)
        collected.objects.extend(collect_type_alias_objects(source, field_counts))
        collected.objects.extend(collect_typescript_const_object_literals(source))
        for inherited in collected.inherited:
            inherited.object.fields += fields_from_bases(inherited.bases, field_counts)
    else:
        collected.objects.extend(collect_rust_tuple_struct_objects(source))
        collected.objects.extend(collect_rust_enum_variant_objects(source))
    return collected.objects


def main() -> int:
    files: list[Path] = []
    for root in ROOTS:
        files.extend(walk(root))

    checked_files = {"rust": 0, "typescript": 0}
    global_ts_field_counts = build_typescript_field_counts(files)
    offenders: list[tuple[Path, ObjectInfo]] = []

    for file_path in files:
        for obj in collect_objects(file_path, global_ts_field_counts, checked_files):
            if obj.fields > MAX_FIELDS:
                offenders.append((file_path, obj))

    for language in ["rust", "typescript"]:
        if checked_files[language] == 0:
            print(f"no {language} files checked", file=sys.stderr)
            return 1

    if offenders:
        for file_path, obj in offenders:
            print(f"{file_path}:{obj.line} {obj.name} has {obj.fields} fields; {SPLIT_HINT}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
