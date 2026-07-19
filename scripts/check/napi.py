#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path
import re
import sys

NAPI_SOURCE_ROOT = Path("crates/versionlens-napi/src")
NAPI_INPUT_PATH = Path("crates/versionlens-napi/src/binding/input.rs")
NATIVE_TYPE_PATH = Path("packages/vscode-extension/src/extension/native/module.ts")
NATIVE_CONFIG_PATH = Path("packages/vscode-extension/src/extension/native/config.ts")
NATIVE_INPUT_PATH = Path("packages/vscode-extension/src/extension/native/input.ts")
NATIVE_OUTPUT_PATH = Path("packages/vscode-extension/src/extension/native/output.ts")
CORE_SUGGESTION_PATH = Path("crates/versionlens-core/src/suggestion.rs")
BUNDLED_EXTENSION_PATH = Path("packages/vscode-extension/dist/extension.js")

ALLOWED_FUNCTIONS = {
    "analyze_document",
    "apply_command",
    "clear_cache",
    "create_session",
    "dispose_session",
    "resolve_document",
}
ALLOWED_TYPE_METHODS = {
    "analyzeDocument",
    "applyCommand",
    "clearCache",
    "createSession",
    "disposeSession",
    "resolveDocument",
}
ALLOWED_STRUCTS = {"NativeSession"}

PLAIN_NAPI_ITEM_PATTERN = re.compile(
    r"#\[napi\]\s*(?:impl\s+([A-Za-z_][A-Za-z0-9_]*)|pub\s+(fn|struct)\s+([A-Za-z_][A-Za-z0-9_]*))"
)
NATIVE_TYPE_METHOD_PATTERN = re.compile(
    r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*\([\s\S]*?\)\s*=>\s*([^;]+);$",
    re.MULTILINE,
)
RESTRICTED_NAPI_DEPENDENCY_PATTERN = re.compile(r"^(napi|napi-build|napi-derive)\.workspace\s*=")
NAPI_MACRO_PATTERN = re.compile(r"(#\[napi(?:\([^)]*\))?\]|use\s+napi_derive::napi)")
JSON_BRIDGE_PATTERN = re.compile(r"\b(?:serde_json|JSON\.(?:parse|stringify))\b")
STRING_BRIDGE_RETURN_PATTERN = re.compile(r"^(?:Promise<)?string(?:>)?$")
APPLY_COMMAND_RUST_INPUT_PATTERN = re.compile(r"struct\s+NativeApplyCommandInput\s*\{(?P<body>[\s\S]*?)\n\}")
SUGGESTION_STATUS_NAME_PATTERN = re.compile(r"(?:SuggestionStatus::[A-Za-z]+|Status[A-Za-z]+)\s*=>\s*\"(?P<status>[^\"]+)\"")
STRING_LITERAL_PATTERN = re.compile(r"\"(?P<value>[^\"]+)\"")
HARDCODED_BUNDLED_CREATE_REQUIRE_PATTERN = re.compile(r"\bcreateRequire\(\s*[\"'](?:file://)?/")
NAPI_OBJECT_PATTERN = re.compile(
    r"#\[napi\(object\)\]\s*pub\s+struct\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\{(?P<body>[\s\S]*?)\n\}"
)
RUST_PUBLIC_FIELD_PATTERN = re.compile(
    r"^\s*pub\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?P<type>.+),\s*$",
    re.MULTILINE,
)
TYPESCRIPT_TYPE_OBJECT_PATTERN = re.compile(
    r"export\s+type\s+(?P<name>[A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*\{(?P<body>[\s\S]*?)\n\};"
)
TYPESCRIPT_INTERFACE_PATTERN = re.compile(
    r"export\s+interface\s+(?P<name>[A-Za-z_$][A-Za-z0-9_$]*)\s*\{(?P<body>[\s\S]*?)\n\}"
)
TYPESCRIPT_FIELD_PATTERN = re.compile(
    r"^\s*(?P<name>[A-Za-z_$][A-Za-z0-9_$]*)(?P<optional>\?)?\s*:",
    re.MULTILINE,
)
NATIVE_TYPE_NAMES = {
    "NativeAnalyzeDocumentOutput": "AnalyzeDocumentOutput",
    "NativeResolveDocumentOutput": "ResolveDocumentOutput",
}


def read(path: Path) -> str:
    return path.read_text()


def line_number(source: str, index: int) -> int:
    return source[:index].count("\n") + 1


def walk(root: Path):
    for entry in root.iterdir():
        if entry.is_dir():
            yield from walk(entry)
        else:
            yield entry


def check_json_bridge_lines(offenders: list[str], file_path: Path, source: str, label: str) -> None:
    for index, line in enumerate(source.split("\n"), start=1):
        if JSON_BRIDGE_PATTERN.search(line):
            offenders.append(f"{file_path}:{index} uses a JSON bridge in {label}")


def check_napi_item(offenders: list[str], exported_functions: set[str], file_path: Path, source: str, match: re.Match[str]) -> None:
    impl_name = match.group(1)
    item_kind = match.group(2)
    item_name = match.group(3)

    if impl_name and impl_name not in ALLOWED_STRUCTS:
        offenders.append(f"{file_path}:{line_number(source, match.start())} unexpected napi impl {impl_name}")
        return

    if item_kind == "struct" and item_name not in ALLOWED_STRUCTS:
        offenders.append(f"{file_path}:{line_number(source, match.start())} unexpected napi struct {item_name}")
        return

    if not impl_name and item_kind == "fn":
        exported_functions.add(item_name)
        if item_name not in ALLOWED_FUNCTIONS:
            offenders.append(f"{file_path}:{line_number(source, match.start())} unexpected napi function {item_name}")


def check_napi_rust_source(offenders: list[str], exported_functions: set[str], file_path: Path) -> None:
    if file_path.suffix != ".rs":
        return
    source = read(file_path)
    check_json_bridge_lines(offenders, file_path, source, "the N-API boundary")
    for match in PLAIN_NAPI_ITEM_PATTERN.finditer(source):
        check_napi_item(offenders, exported_functions, file_path, source, match)


def check_napi_dependency_line(offenders: list[str], file_path: Path, line: str, line_number_value: int) -> None:
    trimmed = line.strip()
    file_label = str(file_path)
    if "versionlens-napi/" not in file_label and RESTRICTED_NAPI_DEPENDENCY_PATTERN.search(trimmed):
        offenders.append(f"{file_path}:{line_number_value} N-API dependency outside versionlens-napi")
    if "versionlens-napi/" in file_label and trimmed.startswith("serde_json"):
        offenders.append(f"{file_path}:{line_number_value} serde_json would create a JSON bridge in versionlens-napi")


def check_cargo_manifest(offenders: list[str], file_path: Path) -> None:
    if file_path.name != "Cargo.toml":
        return
    for index, line in enumerate(read(file_path).split("\n"), start=1):
        check_napi_dependency_line(offenders, file_path, line, index)


def check_non_napi_rust_file(offenders: list[str], file_path: Path) -> None:
    if not (file_path.suffix == ".rs" and "versionlens-napi/" not in str(file_path)):
        return
    for index, line in enumerate(read(file_path).split("\n"), start=1):
        if NAPI_MACRO_PATTERN.search(line):
            offenders.append(f"{file_path}:{index} N-API macro outside versionlens-napi")


def check_expected_napi_functions(offenders: list[str], exported_functions: set[str]) -> None:
    for expected in sorted(ALLOWED_FUNCTIONS):
        if expected not in exported_functions:
            offenders.append(f"{NAPI_SOURCE_ROOT} missing expected napi function {expected}")


def check_native_method(offenders: list[str], typed_methods: set[str], method: str, return_type: str) -> None:
    typed_methods.add(method)
    if method not in ALLOWED_TYPE_METHODS:
        offenders.append(f"{NATIVE_TYPE_PATH} unexpected native method {method}")
    if STRING_BRIDGE_RETURN_PATTERN.search(return_type):
        offenders.append(f"{NATIVE_TYPE_PATH} native method {method} returns {return_type}; use typed N-API objects")


def check_native_loader_path(offenders: list[str], native_type_source: str) -> None:
    if "loadNative(extensionPath: string)" not in native_type_source:
        offenders.append(f"{NATIVE_TYPE_PATH} loadNative must receive the VS Code extension path")
    if not (
        'join(extensionPath, "dist", "extension.js")' in native_type_source
        and 'join(extensionPath, "native", "versionlens_napi.node")' in native_type_source
    ):
        offenders.append(f"{NATIVE_TYPE_PATH} must load native/versionlens_napi.node from the VS Code extension path")
    if "__filename" in native_type_source:
        offenders.append(f"{NATIVE_TYPE_PATH} must not use bundled __filename for native loading")

    if BUNDLED_EXTENSION_PATH.exists():
        bundled_source = read(BUNDLED_EXTENSION_PATH)
        if HARDCODED_BUNDLED_CREATE_REQUIRE_PATTERN.search(bundled_source) or "__filename" in bundled_source:
            offenders.append(
                f"{BUNDLED_EXTENSION_PATH} hardcodes the source native loader path; rerun bun run build after loader changes"
            )


def check_native_types(offenders: list[str], typed_methods: set[str], native_type_source: str) -> None:
    check_json_bridge_lines(offenders, NATIVE_TYPE_PATH, native_type_source, "native typings")
    for match in NATIVE_TYPE_METHOD_PATTERN.finditer(native_type_source):
        check_native_method(offenders, typed_methods, match.group(1), match.group(2).strip())
    for expected in sorted(ALLOWED_TYPE_METHODS):
        if expected not in typed_methods:
            offenders.append(f"{NATIVE_TYPE_PATH} missing expected native method {expected}")


def check_apply_command_rust_input(offenders: list[str], napi_input_source: str) -> None:
    match = APPLY_COMMAND_RUST_INPUT_PATTERN.search(napi_input_source)
    body = match.group("body") if match else ""
    if not body:
        offenders.append(f"{NAPI_INPUT_PATH} missing NativeApplyCommandInput")
        return
    if "pub document: NativeDocumentInput" not in body:
        offenders.append(f"{NAPI_INPUT_PATH} NativeApplyCommandInput must nest NativeDocumentInput")
    for field in ["uri", "language_id", "text", "workspace_root"]:
        if re.search(rf"\bpub\s+{field}\s*:", body):
            offenders.append(f"{NAPI_INPUT_PATH} NativeApplyCommandInput duplicates document field {field}")


def check_apply_command_typescript_input(offenders: list[str], native_input_source: str) -> None:
    body = typescript_object_body(native_input_source, "NativeApplyCommandInput")
    if not body:
        offenders.append(f"{NATIVE_INPUT_PATH} missing NativeApplyCommandInput")
        return
    if "document: NativeDocumentInput" not in body:
        offenders.append(f"{NATIVE_INPUT_PATH} NativeApplyCommandInput must nest NativeDocumentInput")
    if "type NativeApplyCommandInput = NativeDocumentInput &" in native_input_source:
        offenders.append(f"{NATIVE_INPUT_PATH} NativeApplyCommandInput must not intersect document fields into the command shape")


def collect_string_matches(source: str, pattern: re.Pattern[str], group_name: str) -> set[str]:
    return {match.group(group_name) for match in pattern.finditer(source) if match.group(group_name)}


def camel_case(name: str) -> str:
    head, *tail = name.split("_")
    return head + "".join(word.capitalize() for word in tail)


def rust_napi_requiredness() -> dict[str, dict[str, bool]]:
    structs: dict[str, dict[str, bool]] = {}
    for file_path in walk(NAPI_SOURCE_ROOT):
        if file_path.suffix != ".rs":
            continue
        for struct_match in NAPI_OBJECT_PATTERN.finditer(read(file_path)):
            fields: dict[str, bool] = {}
            for field_match in RUST_PUBLIC_FIELD_PATTERN.finditer(struct_match.group("body")):
                field_type = field_match.group("type").strip()
                fields[camel_case(field_match.group("name"))] = not field_type.startswith("Option<")
            structs[struct_match.group("name")] = fields
    return structs


def typescript_requiredness(source: str) -> dict[str, dict[str, bool]]:
    structs: dict[str, dict[str, bool]] = {}
    for pattern in (TYPESCRIPT_TYPE_OBJECT_PATTERN, TYPESCRIPT_INTERFACE_PATTERN):
        for type_match in pattern.finditer(source):
            structs[type_match.group("name")] = {
                field_match.group("name"): field_match.group("optional") is None
                for field_match in TYPESCRIPT_FIELD_PATTERN.finditer(type_match.group("body"))
            }
    return structs


def typescript_object_body(source: str, name: str) -> str:
    for pattern in (TYPESCRIPT_TYPE_OBJECT_PATTERN, TYPESCRIPT_INTERFACE_PATTERN):
        for type_match in pattern.finditer(source):
            if type_match.group("name") == name:
                return type_match.group("body")
    return ""


def check_native_field_requiredness(offenders: list[str], native_sources: list[str]) -> None:
    rust_structs = rust_napi_requiredness()
    typescript_structs: dict[str, dict[str, bool]] = {}
    for source in native_sources:
        typescript_structs.update(typescript_requiredness(source))

    for rust_name, rust_fields in sorted(rust_structs.items()):
        typescript_name = NATIVE_TYPE_NAMES.get(rust_name, rust_name)
        typescript_fields = typescript_structs.get(typescript_name)
        if typescript_fields is None:
            offenders.append(f"native typings missing N-API object {typescript_name} for {rust_name}")
            continue
        for field_name, rust_required in sorted(rust_fields.items()):
            if field_name not in typescript_fields:
                offenders.append(f"native typings {typescript_name} missing Rust field {field_name}")
                continue
            typescript_required = typescript_fields[field_name]
            if typescript_required != rust_required:
                rust_label = "required" if rust_required else "optional"
                typescript_label = "required" if typescript_required else "optional"
                offenders.append(
                    f"native typings {typescript_name}.{field_name} is {typescript_label}; "
                    f"Rust N-API field is {rust_label}"
                )
        for field_name in sorted(typescript_fields.keys() - rust_fields.keys()):
            offenders.append(f"native typings {typescript_name}.{field_name} has no Rust N-API field")


def check_native_suggestion_statuses(offenders: list[str], core_suggestion_source: str, native_output_source: str) -> None:
    rust_statuses = collect_string_matches(core_suggestion_source, SUGGESTION_STATUS_NAME_PATTERN, "status")
    suggestion_body = typescript_object_body(native_output_source, "NativeSuggestion")
    status_match = re.search(r"^\s*status\s*:\s*(?P<body>[\s\S]*?);", suggestion_body, re.MULTILINE)
    status_body = status_match.group("body") if status_match else ""
    if not rust_statuses:
        offenders.append(f"{CORE_SUGGESTION_PATH} missing suggestion status mappings")
        return
    if not status_body:
        offenders.append(f"{NATIVE_OUTPUT_PATH} missing NativeSuggestion.status union")
        return
    native_statuses = collect_string_matches(status_body, STRING_LITERAL_PATTERN, "value")
    for status in sorted(rust_statuses):
        if status not in native_statuses:
            offenders.append(f"{NATIVE_OUTPUT_PATH} NativeSuggestion.status missing Rust-emitted status {status}")


def main() -> int:
    offenders: list[str] = []
    exported_functions: set[str] = set()
    typed_methods: set[str] = set()

    native_type_source = read(NATIVE_TYPE_PATH)
    native_config_source = read(NATIVE_CONFIG_PATH)
    napi_input_source = read(NAPI_INPUT_PATH)
    native_input_source = read(NATIVE_INPUT_PATH)
    native_output_source = read(NATIVE_OUTPUT_PATH)
    core_suggestion_source = read(CORE_SUGGESTION_PATH)

    for entry in walk(NAPI_SOURCE_ROOT):
        check_napi_rust_source(offenders, exported_functions, entry)
    for entry in walk(Path("crates")):
        check_cargo_manifest(offenders, entry)
        check_non_napi_rust_file(offenders, entry)

    check_expected_napi_functions(offenders, exported_functions)
    check_native_loader_path(offenders, native_type_source)
    check_native_types(offenders, typed_methods, native_type_source)
    check_native_field_requiredness(
        offenders,
        [native_config_source, native_input_source, native_output_source],
    )
    check_apply_command_rust_input(offenders, napi_input_source)
    check_apply_command_typescript_input(offenders, native_input_source)
    check_native_suggestion_statuses(offenders, core_suggestion_source, native_output_source)

    if offenders:
        print("\n".join(offenders), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
