#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const napiSourceRoot = "crates/versionlens-napi/src";
const napiInputPath = "crates/versionlens-napi/src/model/input.rs";
const nativeTypePath =
	"packages/vscode-extension/src/extension/native/module.ts";
const nativeInputPath =
	"packages/vscode-extension/src/extension/native/input.ts";
const nativeOutputPath =
	"packages/vscode-extension/src/extension/native/output.ts";
const coreSuggestionPath = "crates/versionlens-core/src/suggestion.rs";
const bundledExtensionPath = "packages/vscode-extension/dist/extension.js";
const nativeTypeSource = fs.readFileSync(nativeTypePath, "utf8");
const napiInputSource = fs.readFileSync(napiInputPath, "utf8");
const nativeInputSource = fs.readFileSync(nativeInputPath, "utf8");
const nativeOutputSource = fs.readFileSync(nativeOutputPath, "utf8");
const coreSuggestionSource = fs.readFileSync(coreSuggestionPath, "utf8");
const allowedFunctions = new Set([
	"analyze_document",
	"apply_command",
	"clear_cache",
	"create_session",
	"dispose_session",
	"resolve_document",
]);
const allowedTypeMethods = new Set([
	"analyzeDocument",
	"applyCommand",
	"clearCache",
	"createSession",
	"disposeSession",
	"resolveDocument",
]);
const allowedStructs = new Set(["NativeSession"]);
const plainNapiItemPattern =
	/#\[napi\]\s*(?:impl\s+([A-Za-z_][A-Za-z0-9_]*)|pub\s+(fn|struct)\s+([A-Za-z_][A-Za-z0-9_]*))/g;
const nativeTypeMethodPattern =
	/^\s*([A-Za-z_][A-Za-z0-9_]*)\([^)]*\):\s*([^;]+);$/gmu;
const restrictedNapiDependencyPattern =
	/^(napi|napi-build|napi-derive)\.workspace\s*=/;
const napiMacroPattern = /(#\[napi(?:\([^)]*\))?\]|use\s+napi_derive::napi)/;
const jsonBridgePattern = /\b(?:serde_json|JSON\.(?:parse|stringify))\b/u;
const stringBridgeReturnPattern = /^(?:Promise<)?string(?:>)?$/u;
const applyCommandRustInputPattern =
	/struct\s+NativeApplyCommandInput\s*\{(?<body>[\s\S]*?)\n\}/u;
const applyCommandTsInputPattern =
	/type\s+NativeApplyCommandInput\s*=\s*\{(?<body>[\s\S]*?)\n\};/u;
const suggestionStatusNamePattern =
	/SuggestionStatus::[A-Za-z]+\s*=>\s*"(?<status>[^"]+)"/gu;
const nativeSuggestionStatusPattern =
	/type\s+NativeSuggestion\s*=\s*\{[\s\S]*?\n\tstatus:(?<body>[\s\S]*?);/u;
const stringLiteralPattern = /"(?<value>[^"]+)"/gu;
const hardcodedBundledCreateRequirePattern =
	/\bcreateRequire\(\s*["'](?:file:\/\/)?\//u;
const offenders = [];
const exportedFunctions = new Set();
const typedMethods = new Set();

function lineNumber(source, index) {
	return source.slice(0, index).split("\n").length;
}

function walk(dir, callback) {
	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		const entryPath = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			walk(entryPath, callback);
			continue;
		}
		callback(entryPath);
	}
}

function checkJsonBridgeLines(filePath, source, label) {
	for (const [index, line] of source.split("\n").entries()) {
		if (jsonBridgePattern.test(line)) {
			offenders.push(`${filePath}:${index + 1} uses a JSON bridge in ${label}`);
		}
	}
}

function recordNapiFunction(filePath, source, match) {
	const itemName = match[3];
	exportedFunctions.add(itemName);
	if (!allowedFunctions.has(itemName)) {
		offenders.push(
			`${filePath}:${lineNumber(source, match.index)} unexpected napi function ${itemName}`,
		);
	}
}

function checkNapiItem(filePath, source, match) {
	const implName = match[1];
	const itemKind = match[2];
	const itemName = match[3];

	if (implName && !allowedStructs.has(implName)) {
		offenders.push(
			`${filePath}:${lineNumber(source, match.index)} unexpected napi impl ${implName}`,
		);
		return;
	}

	if (itemKind === "struct" && !allowedStructs.has(itemName)) {
		offenders.push(
			`${filePath}:${lineNumber(source, match.index)} unexpected napi struct ${itemName}`,
		);
		return;
	}

	if (!implName && itemKind === "fn") {
		recordNapiFunction(filePath, source, match);
	}
}

function checkNapiRustSource(filePath) {
	if (!filePath.endsWith(".rs")) {
		return;
	}

	const source = fs.readFileSync(filePath, "utf8");
	checkJsonBridgeLines(filePath, source, "the N-API boundary");
	for (const match of source.matchAll(plainNapiItemPattern)) {
		checkNapiItem(filePath, source, match);
	}
}

function checkNapiDependencyLine(filePath, line, lineNumberValue) {
	const trimmed = line.trim();
	if (
		!filePath.includes("versionlens-napi/") &&
		restrictedNapiDependencyPattern.test(trimmed)
	) {
		offenders.push(
			`${filePath}:${lineNumberValue} N-API dependency outside versionlens-napi`,
		);
	}
	if (
		filePath.includes("versionlens-napi/") &&
		trimmed.startsWith("serde_json")
	) {
		offenders.push(
			`${filePath}:${lineNumberValue} serde_json would create a JSON bridge in versionlens-napi`,
		);
	}
}

function checkCargoManifest(filePath) {
	if (!filePath.endsWith("Cargo.toml")) {
		return;
	}

	const manifest = fs.readFileSync(filePath, "utf8");
	for (const [index, line] of manifest.split("\n").entries()) {
		checkNapiDependencyLine(filePath, line, index + 1);
	}
}

function checkNonNapiRustFile(filePath) {
	if (!(filePath.endsWith(".rs") && !filePath.includes("versionlens-napi/"))) {
		return;
	}

	const rust = fs.readFileSync(filePath, "utf8");
	for (const [index, line] of rust.split("\n").entries()) {
		if (napiMacroPattern.test(line)) {
			offenders.push(
				`${filePath}:${index + 1} N-API macro outside versionlens-napi`,
			);
		}
	}
}

function checkExpectedNapiFunctions() {
	for (const expected of allowedFunctions) {
		if (!exportedFunctions.has(expected)) {
			offenders.push(
				`${napiSourceRoot} missing expected napi function ${expected}`,
			);
		}
	}
}

function checkNativeMethod(match) {
	const method = match[1];
	const returnType = match[2].trim();
	typedMethods.add(method);
	if (!allowedTypeMethods.has(method)) {
		offenders.push(`${nativeTypePath} unexpected native method ${method}`);
	}
	if (stringBridgeReturnPattern.test(returnType)) {
		offenders.push(
			`${nativeTypePath} native method ${method} returns ${returnType}; use typed N-API objects`,
		);
	}
}

function checkNativeLoaderPath() {
	if (!nativeTypeSource.includes("loadNative(extensionPath: string)")) {
		offenders.push(
			`${nativeTypePath} loadNative must receive the VS Code extension path`,
		);
	}
	if (
		!(
			nativeTypeSource.includes(
				'join(extensionPath, "dist", "extension.js")',
			) &&
			nativeTypeSource.includes(
				'join(extensionPath, "native", "versionlens_napi.node")',
			)
		)
	) {
		offenders.push(
			`${nativeTypePath} must load native/versionlens_napi.node from the VS Code extension path`,
		);
	}
	if (nativeTypeSource.includes("__filename")) {
		offenders.push(
			`${nativeTypePath} must not use bundled __filename for native loading`,
		);
	}

	if (fs.existsSync(bundledExtensionPath)) {
		const bundledSource = fs.readFileSync(bundledExtensionPath, "utf8");
		if (
			hardcodedBundledCreateRequirePattern.test(bundledSource) ||
			bundledSource.includes("__filename")
		) {
			offenders.push(
				`${bundledExtensionPath} hardcodes the source native loader path; rerun bun run build after loader changes`,
			);
		}
	}
}

function checkNativeTypes() {
	checkJsonBridgeLines(nativeTypePath, nativeTypeSource, "native typings");
	for (const match of nativeTypeSource.matchAll(nativeTypeMethodPattern)) {
		checkNativeMethod(match);
	}
	for (const expected of allowedTypeMethods) {
		if (!typedMethods.has(expected)) {
			offenders.push(
				`${nativeTypePath} missing expected native method ${expected}`,
			);
		}
	}
}

function checkApplyCommandRustInput() {
	const body = napiInputSource.match(applyCommandRustInputPattern)?.groups
		?.body;
	if (!body) {
		offenders.push(`${napiInputPath} missing NativeApplyCommandInput`);
		return;
	}

	if (!body.includes("pub document: NativeDocumentInput")) {
		offenders.push(
			`${napiInputPath} NativeApplyCommandInput must nest NativeDocumentInput`,
		);
	}
	for (const field of ["uri", "language_id", "text", "workspace_root"]) {
		if (new RegExp(`\\bpub\\s+${field}\\s*:`, "u").test(body)) {
			offenders.push(
				`${napiInputPath} NativeApplyCommandInput duplicates document field ${field}`,
			);
		}
	}
}

function checkApplyCommandTypeScriptInput() {
	const body = nativeInputSource.match(applyCommandTsInputPattern)?.groups
		?.body;
	if (!body) {
		offenders.push(`${nativeInputPath} missing NativeApplyCommandInput`);
		return;
	}

	if (!body.includes("document: NativeDocumentInput")) {
		offenders.push(
			`${nativeInputPath} NativeApplyCommandInput must nest NativeDocumentInput`,
		);
	}
	if (
		nativeInputSource.includes(
			"type NativeApplyCommandInput = NativeDocumentInput &",
		)
	) {
		offenders.push(
			`${nativeInputPath} NativeApplyCommandInput must not intersect document fields into the command shape`,
		);
	}
}

function collectStringMatches(source, pattern, groupName) {
	return new Set(
		Array.from(source.matchAll(pattern), (match) => match.groups?.[groupName])
			.filter((value) => typeof value === "string")
			.toSorted(),
	);
}

function checkNativeSuggestionStatuses() {
	const rustStatuses = collectStringMatches(
		coreSuggestionSource,
		suggestionStatusNamePattern,
		"status",
	);
	const statusBody = nativeOutputSource.match(nativeSuggestionStatusPattern)
		?.groups?.body;

	if (rustStatuses.size === 0) {
		offenders.push(`${coreSuggestionPath} missing suggestion status mappings`);
		return;
	}
	if (!statusBody) {
		offenders.push(`${nativeOutputPath} missing NativeSuggestion.status union`);
		return;
	}

	const nativeStatuses = collectStringMatches(
		statusBody,
		stringLiteralPattern,
		"value",
	);
	for (const status of rustStatuses) {
		if (!nativeStatuses.has(status)) {
			offenders.push(
				`${nativeOutputPath} NativeSuggestion.status missing Rust-emitted status ${status}`,
			);
		}
	}
}

walk(napiSourceRoot, checkNapiRustSource);
walk("crates", (entryPath) => {
	checkCargoManifest(entryPath);
	checkNonNapiRustFile(entryPath);
});
checkExpectedNapiFunctions();
checkNativeLoaderPath();
checkNativeTypes();
checkApplyCommandRustInput();
checkApplyCommandTypeScriptInput();
checkNativeSuggestionStatuses();

if (offenders.length > 0) {
	console.error(offenders.join("\n"));
	process.exit(1);
}

console.log(
	`checked N-API boundary; ${allowedFunctions.size} coarse operations only with no JSON bridge`,
);
