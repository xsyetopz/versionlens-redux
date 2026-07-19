#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path
import json
import os
import re
import sys
from typing import Any

ROOTS = [Path("packages/vscode-extension/src"), Path("tests")]
ALLOWED_EXTENSION_SOURCE_DIRS = {"extension", "schema"}
ALLOWED_ROOT_SOURCE_FILES = {"extension.ts"}
ALLOWED_BARE_IMPORTS = {"#vscode-host", "bun:test"}
FORBIDDEN_NODE_IMPORTS = {"node:child_process"}
APPROVED_PACKAGE_OVERRIDES: dict[str, Any] = {
    "name": "versionlens-redux",
    "displayName": "VersionLens Redux",
    "publisher": "xsyetopz",
    "engines": {"vscode": ">=1.75.0"},
    "repository": {"type": "git", "url": "https://github.com/xsyetopz/versionlens-redux.git"},
}
INTERNAL_REGISTERED_COMMANDS = {
    "versionlens.suggestion.onChooseBuild",
    "versionlens.suggestion.onFileLink",
    "versionlens.suggestion.onUpdateDependency",
}

IMPORT_PATTERN = re.compile(r"\b(?:import|export)\b(?:[^\"'`]*?\bfrom\s*)?[\"']([^\"']+)[\"']|createRequire\([^)]*\)\([\"']([^\"']+)[\"']\)")
JSON_BRIDGE_PATTERN = re.compile(r"\bJSON\.(?:parse|stringify)\s*\(")
CONFIG_READ_PATTERN = re.compile(r"\b(?:configuredValue(?:<[^>]+>)?|get(?:<[^>]+>)?)\(\s*\"([^\"]+)\"")
CONFIG_PAIR_PATTERN = re.compile(r"\[\s*\"[^\"]+\"\s*,\s*\"([^\"]+)\"")
CONFIG_TUPLE_PATTERN = re.compile(r"\[\s*\"([^\"]+)\"\s*,\s*\"([^\"]+)\"")
CONFIG_KEY_PATTERN = re.compile(r"^[a-z][a-z.]*\.[A-Za-z]")
PACKAGE_ASSET_PATTERN = re.compile(r"^(images|schemas|src/schema)/")
MARKDOWN_IMAGE_PATTERN = re.compile(r"!\[[^\]]*\]\(([^)]+)\)")
IMAGE_ASSET_PATTERN = re.compile(r"^images/")
ABSOLUTE_URL_PATTERN = re.compile(r"^[a-z]+:", re.IGNORECASE)
PACKAGED_IMAGE_BUDGET_BYTES = 147_000
FILE_PATTERN_DETAILS_PATTERN = re.compile(r"\[\s*\"([^\"]+)\"\s*,\s*\"([^\"]+)\"\s*,\s*\[([^\]]*)\]")
STRING_LITERAL_PATTERN = re.compile(r"\"([^\"]+)\"")
RUST_COMMAND_PATTERN = re.compile(r"\"(?P<command>versionlens\.[^\"]+)\"")
REGISTER_COMMAND_PATTERN = re.compile(r"registerCommand\(\s*\"([^\"]+)\"")
COMMAND_TUPLE_PATTERN = re.compile(r"\[\s*\"([^\"]+)\"\s*,\s*\"[^\"]+\"\s*,?\s*\]")


def read(path: str | Path) -> str:
    return Path(path).read_text()


def read_json(path: str | Path) -> Any:
    return json.loads(read(path))


def stable_json(value: Any) -> str:
    return json.dumps(value, separators=(",", ":"), ensure_ascii=False)


def walk_ts(root: str | Path) -> list[Path]:
    root = Path(root)
    if not root.exists():
        return []
    files: list[Path] = []
    for entry in root.iterdir():
        if entry.is_dir():
            files.extend(walk_ts(entry))
        elif entry.name.endswith(".ts"):
            files.append(entry)
    return files


def package_files(root: str | Path) -> list[Path]:
    root = Path(root)
    files: list[Path] = []
    for entry in root.iterdir():
        if entry.is_dir():
            files.extend(package_files(entry))
        else:
            files.append(entry)
    return files


def as_posix(path: str | Path) -> str:
    return Path(path).as_posix()


def is_allowed_import(specifier: str) -> bool:
    return specifier.startswith(".") or specifier.startswith("node:") or specifier in ALLOWED_BARE_IMPORTS


def const_section(source: str, name: str) -> str:
    start = source.find(f"export const {name}")
    if start < 0:
        return ""
    end = source.find("] as const;", start)
    return "" if end < 0 else source[start:end]


def config_tuples(source: str) -> list[tuple[str, str]]:
    return [(m.group(1), m.group(2)) for m in CONFIG_TUPLE_PATTERN.finditer(source) if CONFIG_KEY_PATTERN.search(m.group(2))]


def file_pattern_details(source: str) -> dict[str, dict[str, Any]]:
    details: dict[str, dict[str, Any]] = {}
    for match in FILE_PATTERN_DETAILS_PATTERN.finditer(source):
        details[match.group(2)] = {
            "ecosystem": match.group(1),
            "languages": [entry.group(1) for entry in STRING_LITERAL_PATTERN.finditer(match.group(3))],
        }
    return details


def rust_string_array(offenders: list[str], file_path: str, name: str) -> list[str]:
    source = read(file_path)
    pattern = re.compile(rf"const\s+{re.escape(name)}:\s*&\[&str\]\s*=\s*&\[(?P<body>[\s\S]*?)\];")
    match = pattern.search(source)
    if not match:
        offenders.append(f"{file_path} missing Rust array {name}")
        return []
    return [entry.group(1) for entry in STRING_LITERAL_PATTERN.finditer(match.group("body"))]


def dependency_properties_default(offenders: list[str], extension_package: dict[str, Any], key: str) -> list[Any]:
    setting = (((extension_package.get("contributes") or {}).get("configuration") or {}).get("properties") or {}).get(
        f"versionlens.{key}.dependencyProperties"
    )
    default = setting.get("default") if isinstance(setting, dict) else None
    if not isinstance(default, list):
        offenders.append(f"versionlens.{key}.dependencyProperties default is not an array")
        return []
    return default


def menu_commands(package_json: dict[str, Any], menu: str) -> list[Any]:
    return [item.get("command") for item in (((package_json.get("contributes") or {}).get("menus") or {}).get(menu) or [])]


def menu_placements(package_json: dict[str, Any], menu: str) -> list[str]:
    return [f"{item.get('command')}\0{item.get('group') or ''}" for item in (((package_json.get("contributes") or {}).get("menus") or {}).get(menu) or [])]


def json_validation_file_matches(package_json: dict[str, Any]) -> list[Any]:
    return [item.get("fileMatch") for item in ((package_json.get("contributes") or {}).get("jsonValidation") or [])]


def rust_versionlens_command_strings(file_path: str) -> list[str]:
    return [match.group("command") for match in RUST_COMMAND_PATTERN.finditer(read(file_path))]


def collect_package_asset_paths(value: Any, paths: list[str] | None = None) -> list[str]:
    if paths is None:
        paths = []
    if not value:
        return paths
    if isinstance(value, str):
        if PACKAGE_ASSET_PATTERN.search(value):
            paths.append(value)
        return paths
    if isinstance(value, list):
        for item in value:
            collect_package_asset_paths(item, paths)
        return paths
    if isinstance(value, dict):
        for item in value.values():
            collect_package_asset_paths(item, paths)
    return paths


def markdown_asset_path(markdown_path: Path, asset_path: str) -> str | None:
    if ABSOLUTE_URL_PATTERN.search(asset_path):
        return None
    joined = (markdown_path.parent / asset_path).resolve(strict=False)
    package_root = Path("packages/vscode-extension").resolve(strict=False)
    try:
        package_relative = joined.relative_to(package_root).as_posix()
    except ValueError:
        return None
    return package_relative


def collect_markdown_asset_paths(markdown_path: Path) -> list[str]:
    return [path for match in MARKDOWN_IMAGE_PATTERN.finditer(read(markdown_path)) if (path := markdown_asset_path(markdown_path, match.group(1)))]


def check_adapter_source_layout(offenders: list[str]) -> None:
    for entry in Path("packages/vscode-extension/src").iterdir():
        if entry.is_dir() and entry.name not in ALLOWED_EXTENSION_SOURCE_DIRS:
            offenders.append(f"packages/vscode-extension/src/{entry.name}/ is not adapter-owned source")
        if entry.is_file() and entry.name not in ALLOWED_ROOT_SOURCE_FILES:
            offenders.append(f"packages/vscode-extension/src/{entry.name} must live under extension/ or be an entrypoint")


def check_typescript_adapter_files(offenders: list[str]) -> None:
    for root in ROOTS:
        for file_path in walk_ts(root):
            source = read(file_path)
            file_label = as_posix(file_path)
            for index, line in enumerate(source.split("\n"), start=1):
                if JSON_BRIDGE_PATTERN.search(line):
                    offenders.append(f"{file_label}:{index} uses JSON bridge in TypeScript adapter")
            for match in IMPORT_PATTERN.finditer(source):
                specifier = match.group(1) or match.group(2)
                if not specifier:
                    continue
                if not is_allowed_import(specifier):
                    offenders.append(f"{file_label} imports {specifier}")
                if specifier in FORBIDDEN_NODE_IMPORTS:
                    offenders.append(f"{file_label} imports {specifier}; Rust owns CLI/domain discovery")


def check_package_metadata(offenders: list[str], extension_package: dict[str, Any]) -> None:
    for field, expected in APPROVED_PACKAGE_OVERRIDES.items():
        if stable_json(extension_package.get(field)) != stable_json(expected):
            offenders.append(f"package.json {field} differs from approved local rebrand")
    if extension_package.get("dependencies"):
        offenders.append("packages/vscode-extension/package.json must not have runtime dependencies")
    if extension_package.get("devDependencies"):
        offenders.append("packages/vscode-extension/package.json must not have package-local devDependencies; use the root Bun workspace")
    if extension_package.get("main") != "./dist/extension.js":
        offenders.append("packages/vscode-extension/package.json main must point at ./dist/extension.js")


def check_assets(offenders: list[str], extension_package: dict[str, Any]) -> None:
    package_asset_paths = collect_package_asset_paths({"icon": extension_package.get("icon"), "contributes": extension_package.get("contributes")})
    markdown_asset_paths: list[str] = []
    for file_path in package_files("packages/vscode-extension"):
        if file_path.name.endswith(".md"):
            markdown_asset_paths.extend(collect_markdown_asset_paths(file_path))
    referenced_asset_paths = set(package_asset_paths + markdown_asset_paths)
    for asset_path in referenced_asset_paths:
        if not Path("packages/vscode-extension", asset_path).exists():
            offenders.append(f"package asset {asset_path} does not exist")
    packaged_image_paths = package_files("packages/vscode-extension/images")
    for image_path in packaged_image_paths:
        package_relative_path = os.path.relpath(image_path, "packages/vscode-extension").replace(os.sep, "/")
        if IMAGE_ASSET_PATTERN.search(package_relative_path) and package_relative_path not in referenced_asset_paths:
            offenders.append(f"{package_relative_path} is packaged but unreferenced")
    packaged_image_bytes = sum(image_path.stat().st_size for image_path in packaged_image_paths)
    if packaged_image_bytes > PACKAGED_IMAGE_BUDGET_BYTES:
        offenders.append(f"packaged image assets use {packaged_image_bytes} bytes; budget is {PACKAGED_IMAGE_BUDGET_BYTES} bytes")


def check_vscodeignore(offenders: list[str]) -> None:
    package_ignore_path = Path("packages/vscode-extension/.vscodeignore")
    package_ignore_lines = read(package_ignore_path).split("\n") if package_ignore_path.exists() else []
    for required_file in ["README.md", "LICENSE", "src/schema/versionlens.multi-registries.json"]:
        if not Path("packages/vscode-extension", required_file).exists():
            offenders.append(f"packages/vscode-extension missing {required_file}")
    for required_ignore in ["src/**", "dist/*.node", "*.vsix", "*.tsbuildinfo", "tsconfig.json", "node_modules/**"]:
        if required_ignore not in package_ignore_lines:
            offenders.append(f"{package_ignore_path.as_posix()} missing {required_ignore}")
    for forbidden_ignore in ["native/**", "native/*.node", "*.node", "**/*.node"]:
        if forbidden_ignore in package_ignore_lines:
            offenders.append(f"{package_ignore_path.as_posix()} must not ignore packaged native module via {forbidden_ignore}")


def check_commands(offenders: list[str], extension_package: dict[str, Any]) -> None:
    command_source = "\n".join(
        read(command_path)
        for command_path in package_files("packages/vscode-extension/src/extension/commands")
        if command_path.suffix == ".ts"
    )
    registered_commands = set([m.group(1) for m in REGISTER_COMMAND_PATTERN.finditer(command_source)] + [m.group(1) for m in COMMAND_TUPLE_PATTERN.finditer(command_source)])
    contributed_commands_list = [command.get("command") for command in ((extension_package.get("contributes") or {}).get("commands") or [])]
    contributed_commands = set(contributed_commands_list)
    local_activation_events = extension_package.get("activationEvents") or []
    for item in ((extension_package.get("contributes") or {}).get("jsonValidation") or []):
        if not Path("packages/vscode-extension", item.get("url", "")).exists():
            offenders.append(f"jsonValidation URL {item.get('url')} does not exist")

    for command in registered_commands:
        if command not in contributed_commands and command not in INTERNAL_REGISTERED_COMMANDS:
            offenders.append(f"{command} is registered but not contributed")
    for command in contributed_commands:
        if command not in registered_commands:
            offenders.append(f"{command} is contributed but not registered")
    for command in rust_versionlens_command_strings("crates/versionlens-core/src/presentation.rs"):
        if command not in contributed_commands and command not in INTERNAL_REGISTERED_COMMANDS:
            offenders.append(f"{command} is emitted by Rust but not contributed")
        if command not in registered_commands:
            offenders.append(f"{command} is emitted by Rust but not registered")


def check_configuration(offenders: list[str], extension_package: dict[str, Any]) -> None:
    contributed_configuration = (((extension_package.get("contributes") or {}).get("configuration") or {}).get("properties") or {})
    contributed_settings = set(contributed_configuration.keys())

    dependency_default_specs = [
        ("cargo", "crates/versionlens-parsers/src/cargo_toml/paths.rs", "CARGO_DEPENDENCY_PATHS"),
        ("composer", "crates/versionlens-parsers/src/json_manifest/paths.rs", "COMPOSER_DEPENDENCY_PATHS"),
        ("deno", "crates/versionlens-parsers/src/json_manifest/paths.rs", "DENO_DEPENDENCY_PATHS"),
        ("dotnet", "crates/versionlens-parsers/src/dotnet_xml.rs", "DOTNET_DEPENDENCY_PATHS"),
        ("dub", "crates/versionlens-parsers/src/json_manifest/paths.rs", "DUB_DEPENDENCY_PATHS"),
        ("maven", "crates/versionlens-parsers/src/maven_xml.rs", "MAVEN_DEPENDENCY_PATHS"),
        ("npm", "crates/versionlens-parsers/src/json_manifest/paths.rs", "NPM_DEPENDENCY_PATHS"),
        ("pnpm", "crates/versionlens-parsers/src/pnpm_yaml/paths.rs", "PNPM_DEPENDENCY_PATHS"),
        ("pub", "crates/versionlens-parsers/src/pubspec_yaml/paths.rs", "PUBSPEC_DEPENDENCY_PATHS"),
        ("pypi", "crates/versionlens-parsers/src/pyproject_toml/paths/defaults.rs", "PYPI_DEPENDENCY_PATHS"),
    ]
    for key, rust_path, rust_const in dependency_default_specs:
        rust_defaults = set(rust_string_array(offenders, rust_path, rust_const))
        for dependency_path in dependency_properties_default(offenders, extension_package, key):
            if dependency_path not in rust_defaults:
                offenders.append(f"versionlens.{key}.dependencyProperties default {dependency_path} is not recognized by {rust_const}")

    config_keys_source = "\n".join(
        read(f"packages/vscode-extension/src/extension/config/keys/{name}.ts")
        for name in ["cache", "dependency-properties", "files", "http", "prerelease", "registry"]
    )
    config_key_pairs = [m.group(1) for m in CONFIG_PAIR_PATTERN.finditer(config_keys_source) if CONFIG_KEY_PATTERN.search(m.group(1))]
    config_reads: list[str] = []
    for file_path in walk_ts("packages/vscode-extension/src"):
        if not file_path.name.endswith(".test.ts"):
            config_reads.extend(match.group(1) for match in CONFIG_READ_PATTERN.finditer(read(file_path)))
    versionlens_config_keys = {key for key in [*config_key_pairs, *config_reads] if key not in {"proxy", "proxyStrictSSL"}}
    for key in versionlens_config_keys:
        if f"versionlens.{key}" not in contributed_settings:
            offenders.append(f"versionlens.{key} is read but not contributed")

    file_pattern_keys = dict(config_tuples(const_section(config_keys_source, "filePatternKeys")))
    rust_supported_file_defaults = {
        "deno.files": "**/{deno.json,deno.jsonc,import_map.json,jsr.json,jsr.jsonc}",
        "docker.files": "**/{dockerfile,*.dockerfile,Dockerfile,*.Dockerfile,compose.yaml,compose.yml,*.compose.yaml,*.compose.yml,compose.*.yaml,compose.*.yml,docker-compose.yaml,docker-compose.yml,docker-compose.*.yaml,docker-compose.*.yml}",
        "dotnet.files": "**/{*.csproj,*.fsproj,*.vbproj,project.json,packages.config,paket.dependencies,paket.references,*.targets,*.props}",
        "pnpm.files": "**/{pnpm-workspace.yaml,pnpm-workspace.yml,.yarnrc.yaml,.yarnrc.yml}",
        "pypi.files": "**/{Pipfile,pyproject.toml,*requirements*.txt,*constraints*.txt}",
        "pub.files": "**/{pubspec.yaml,pubspec.yml,pubspec_overrides.yaml}",
    }
    rust_supported_file_languages = {"dotnet.files": ["xml", "json", "jsonc"]}
    for key, expected_default in rust_supported_file_defaults.items():
        contributed_default = (contributed_configuration.get(f"versionlens.{key}") or {}).get("default")
        if contributed_default != expected_default:
            offenders.append(f"versionlens.{key} default does not cover Rust-supported manifest variants")
    file_pattern_details_by_key = file_pattern_details(config_keys_source)
    for key, expected_languages in rust_supported_file_languages.items():
        languages = (file_pattern_details_by_key.get(key) or {}).get("languages") or []
        for language in expected_languages:
            if language not in languages:
                offenders.append(f"{key} selectors missing {language} for Rust-supported manifests")

    provider_setting_lists: list[tuple[str, set[str]]] = [
        (".apiUrl", {key for _, key in config_tuples(const_section(config_keys_source, "registryUrlKeys"))}),
        (".files", set(file_pattern_keys.values())),
        (".prereleaseTagFilter", {key for _, key in config_tuples(const_section(config_keys_source, "prereleaseTagKeys"))}),
        (".http.strictSSL", {key for _, key in config_tuples(const_section(config_keys_source, "providerStrictSslKeys"))}),
        (".caching.duration", {key for _, key in config_tuples(const_section(config_keys_source, "providerCacheKeys"))}),
        (".dependencyProperties", {key for _, key in config_tuples(const_section(config_keys_source, "dependencyPropertyKeys"))}),
    ]
    for key in contributed_settings:
        short_key = re.sub(r"^versionlens\.", "", key)
        for suffix, known_keys in provider_setting_lists:
            if short_key.endswith(suffix) and short_key not in known_keys:
                offenders.append(f"{key} is contributed but missing from extension/config/keys/*.ts")


def main() -> int:
    offenders: list[str] = []
    check_adapter_source_layout(offenders)
    check_typescript_adapter_files(offenders)

    extension_package = read_json("packages/vscode-extension/package.json")
    check_package_metadata(offenders, extension_package)
    check_assets(offenders, extension_package)
    check_vscodeignore(offenders)
    check_commands(offenders, extension_package)
    check_configuration(offenders, extension_package)

    if offenders:
        print("\n".join(offenders), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
