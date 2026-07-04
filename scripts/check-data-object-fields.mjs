#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const MAX_FIELDS = 10;
const SPLIT_HINT =
	"split it on the same Rust/TypeScript side into smaller responsibility objects with fields distributed evenly; do not move overflow to the other language or a catch-all object";
const roots = ["crates", "packages/vscode-extension/src", "tests"];
const ignoredDirs = new Set(["dist", "node_modules", "target"]);
const rustFieldPattern = /^(pub\([^)]*\)\s+|pub\s+)?[A-Za-z_][A-Za-z0-9_]*\s*:/;
const typeScriptFieldPattern =
	/^(?:(?:public|private|protected|readonly|static|declare|override)\s+)*(#?[A-Za-z_$][\w$]*|"[^"]+"|'[^']+')\??\s*[:=]/;
const typeScriptParameterPropertyPattern =
	/^(?:(?:public|private|protected|readonly|override)\s+)+#?[A-Za-z_$][\w$]*\??\s*[:=]/;
const baseNameSeparatorPattern = /\W/u;
const baseListSeparatorPattern = /[,&]/u;
const rustStructPattern = /\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/g;
const rustTupleStructPattern = /\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/g;
const rustEnumPattern = /\benum\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/g;
const typeScriptInterfacePattern =
	/\binterface\s+([A-Za-z_$][\w$]*)(?:\s+extends\s+([^{]+))?\s*\{/g;
const typeScriptClassPattern = /\bclass\s+([A-Za-z_$][\w$]*)[^{]*\{/g;
const typeScriptTypeAliasPattern = /\btype\s+([A-Za-z_$][\w$]*)\s*=/g;
const typeScriptConstObjectPattern =
	/\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)[^=]*=\s*\{/g;
const typeScriptReferenceNamePattern = /^([A-Za-z_$][\w$]*)\b/u;
const whitespacePattern = /\s+/u;
const files = [];
const checkedFiles = {
	rust: 0,
	typescript: 0,
};
let globalTypeScriptFieldCounts = new Map();

function walk(dir) {
	if (!fs.existsSync(dir)) {
		return;
	}

	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		if (ignoredDirs.has(entry.name)) {
			continue;
		}

		const filePath = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			walk(filePath);
			continue;
		}

		if (entry.name.endsWith(".rs") || entry.name.endsWith(".ts")) {
			files.push(filePath);
		}
	}
}

function stripComments(source) {
	return source.replaceAll(/\/\*[\s\S]*?\*\//g, "").replaceAll(/\/\/.*$/gm, "");
}

function isQuote(char) {
	return char === '"' || char === "'" || char === "`";
}

function depthState() {
	return {
		braceDepth: 0,
		bracketDepth: 0,
		parenDepth: 0,
		quote: "",
	};
}

function trackQuote(state, char, previous) {
	if (state.quote) {
		if (char === state.quote && previous !== "\\") {
			state.quote = "";
		}
		return true;
	}

	if (isQuote(char)) {
		state.quote = char;
		return true;
	}

	return false;
}

function updateDepth(state, char) {
	switch (char) {
		case "{":
			state.braceDepth += 1;
			break;
		case "}":
			state.braceDepth -= 1;
			break;
		case "(":
			state.parenDepth += 1;
			break;
		case ")":
			state.parenDepth -= 1;
			break;
		case "[":
			state.bracketDepth += 1;
			break;
		case "]":
			state.bracketDepth -= 1;
			break;
		default:
			break;
	}
}

function isTopLevel(state) {
	return (
		state.braceDepth === 0 && state.parenDepth === 0 && state.bracketDepth === 0
	);
}

function findMatchingBrace(source, openIndex) {
	const state = depthState();

	for (let index = openIndex; index < source.length; index += 1) {
		const char = source[index];
		const previous = source[index - 1];

		if (trackQuote(state, char, previous)) {
			continue;
		}

		updateDepth(state, char);

		if (state.braceDepth === 0) {
			return index;
		}
	}

	return -1;
}

function findTypeAliasEnd(source, bodyStart) {
	const state = depthState();

	for (let index = bodyStart; index < source.length; index += 1) {
		const char = source[index];
		const previous = source[index - 1];

		if (trackQuote(state, char, previous)) {
			continue;
		}

		if (char === ";" && isTopLevel(state)) {
			return index;
		}

		updateDepth(state, char);
	}

	return source.length;
}

function lineNumber(source, index) {
	return source.slice(0, index).split("\n").length;
}

function topLevelFields(body, language) {
	const clean = stripComments(body);
	return splitTopLevel(clean, [",", ";"])
		.map((part) => part.trim())
		.filter((part) => {
			if (!part) {
				return false;
			}

			if (language === "rust") {
				return rustFieldPattern.test(part);
			}

			return typeScriptFieldPattern.test(part);
		});
}

function countTypeScriptConstructorParameterFields(body) {
	const clean = stripComments(body);
	const state = depthState();

	for (let index = 0; index < clean.length; index += 1) {
		const char = clean[index];
		const previous = clean[index - 1];

		if (trackQuote(state, char, previous)) {
			continue;
		}

		if (
			state.braceDepth === 0 &&
			state.parenDepth === 0 &&
			state.bracketDepth === 0 &&
			clean.startsWith("constructor", index)
		) {
			const openIndex = clean.indexOf("(", index);
			if (openIndex < 0) {
				return 0;
			}

			const closeIndex = findMatchingParen(clean, openIndex);
			if (closeIndex < 0) {
				return 0;
			}

			return splitTopLevel(clean.slice(openIndex + 1, closeIndex), [
				",",
			]).filter((field) =>
				typeScriptParameterPropertyPattern.test(field.trim()),
			).length;
		}

		updateDepth(state, char);
	}

	return 0;
}

function splitTopLevel(source, separators) {
	return splitTopLevelRanges(source, separators).map((part) => part.text);
}

function splitTopLevelRanges(source, separators) {
	const parts = [];
	let start = 0;
	const state = depthState();

	for (let index = 0; index < source.length; index += 1) {
		const char = source[index];
		const previous = source[index - 1];

		if (trackQuote(state, char, previous)) {
			continue;
		}

		if (separators.includes(char) && isTopLevel(state)) {
			parts.push({ start, text: source.slice(start, index) });
			start = index + 1;
			continue;
		}

		updateDepth(state, char);
	}

	parts.push({ start, text: source.slice(start) });

	return parts;
}

function baseNames(source) {
	return source
		.split(baseListSeparatorPattern)
		.map((base) => base.trim().split(baseNameSeparatorPattern)[0])
		.filter(Boolean);
}

function collectNamedObjectFields(source, language, patterns) {
	const objects = [];
	const inherited = [];
	const fieldCounts = new Map();
	for (const pattern of patterns) {
		for (const match of source.matchAll(pattern)) {
			const openIndex = source.indexOf("{", match.index);
			const closeIndex = findMatchingBrace(source, openIndex);
			if (closeIndex < 0) {
				continue;
			}

			const fields =
				topLevelFields(source.slice(openIndex + 1, closeIndex), language)
					.length +
				(language === "typescript" && pattern === typeScriptClassPattern
					? countTypeScriptConstructorParameterFields(
							source.slice(openIndex + 1, closeIndex),
						)
					: 0);
			fieldCounts.set(match[1], fields);
			const object = {
				fields,
				line: lineNumber(source, match.index),
				name: match[1],
			};
			objects.push(object);

			if (language === "typescript" && match[2]) {
				inherited.push({ bases: baseNames(match[2]), object });
			}
		}
	}

	return { fieldCounts, inherited, objects };
}

function typeReferenceName(typePart) {
	const trimmed = typePart.trim();
	const match = typeScriptReferenceNamePattern.exec(trimmed);
	return match?.[1];
}

function topLevelObjectLiteralFieldCount(typePart) {
	let fields = 0;
	const state = depthState();

	for (let index = 0; index < typePart.length; index += 1) {
		const char = typePart[index];
		const previous = typePart[index - 1];

		if (trackQuote(state, char, previous)) {
			continue;
		}

		if (char === "{" && isTopLevel(state)) {
			const closeIndex = findMatchingBrace(typePart, index);
			if (closeIndex < 0) {
				return fields;
			}

			fields += topLevelFields(
				typePart.slice(index + 1, closeIndex),
				"typescript",
			).length;
			index = closeIndex;
			continue;
		}

		updateDepth(state, char);
	}

	return fields;
}

function typeAliasIntersectionPartFields(typePart, fieldCounts) {
	const ownFields = topLevelObjectLiteralFieldCount(typePart);
	if (ownFields > 0) {
		return ownFields;
	}

	const referenceName = typeReferenceName(typePart);
	return referenceName ? (fieldCounts.get(referenceName) ?? 0) : 0;
}

function typeAliasVariantFields(variant, fieldCounts) {
	return splitTopLevel(variant, ["&"]).reduce(
		(sum, typePart) =>
			sum + typeAliasIntersectionPartFields(typePart, fieldCounts),
		0,
	);
}

function typeAliasObjectName(aliasName, variantIndex) {
	return variantIndex === 1 ? aliasName : `${aliasName}#${variantIndex}`;
}

function scanTypeAliasObjects(source, fieldCounts, alias) {
	const objects = [];
	let variantIndex = 0;
	let maxFields = 0;

	for (const variant of splitTopLevelRanges(alias.body, ["|"])) {
		const fields = typeAliasVariantFields(variant.text, fieldCounts);
		if (fields === 0) {
			continue;
		}

		variantIndex += 1;
		objects.push({
			fields,
			line: lineNumber(source, alias.bodyStart + variant.start),
			name: typeAliasObjectName(alias.name, variantIndex),
		});
		maxFields = Math.max(maxFields, fields);
	}

	return { maxFields, objects };
}

function collectTypeAliasObjects(source, fieldCounts) {
	const objects = [];
	for (const match of source.matchAll(typeScriptTypeAliasPattern)) {
		const bodyStart = source.indexOf("=", match.index) + 1;
		const bodyEnd = findTypeAliasEnd(source, bodyStart);
		const result = scanTypeAliasObjects(source, fieldCounts, {
			body: source.slice(bodyStart, bodyEnd),
			bodyEnd,
			bodyStart,
			name: match[1],
		});
		objects.push(...result.objects);

		if (result.maxFields > 0) {
			fieldCounts.set(match[1], result.maxFields);
		}
	}

	return objects;
}

function collectRustEnumVariantObjects(source) {
	const objects = [];
	for (const match of source.matchAll(rustEnumPattern)) {
		const openIndex = source.indexOf("{", match.index);
		const closeIndex = findMatchingBrace(source, openIndex);
		if (closeIndex < 0) {
			continue;
		}

		const enumBody = source.slice(openIndex + 1, closeIndex);
		for (const variant of splitTopLevel(stripComments(enumBody), [","])) {
			const variantOffset = openIndex + 1 + enumBody.indexOf(variant);
			const object = rustEnumVariantObject(
				source,
				match[1],
				variant,
				variantOffset,
			);
			if (object) {
				objects.push(object);
			}
		}
	}

	return objects;
}

function rustEnumVariantObject(source, enumName, variant, variantOffset) {
	const namedOpen = variant.indexOf("{");
	const tupleOpen = variant.indexOf("(");
	const variantOpen =
		namedOpen >= 0 && (tupleOpen < 0 || namedOpen < tupleOpen)
			? namedOpen
			: tupleOpen;
	if (variantOpen < 0) {
		return undefined;
	}

	const name = variant.slice(0, variantOpen).trim().split(whitespacePattern)[0];
	if (!name) {
		return undefined;
	}

	const isTuple = variant[variantOpen] === "(";
	const variantClose = variant.lastIndexOf(isTuple ? ")" : "}");
	if (variantClose < variantOpen) {
		return undefined;
	}

	const body = variant.slice(variantOpen + 1, variantClose);
	const fields = isTuple
		? splitTopLevel(body, [","]).filter((field) => field.trim()).length
		: topLevelFields(body, "rust").length;
	return {
		fields,
		line: lineNumber(source, variantOffset),
		name: `${enumName}::${name}`,
	};
}

function collectRustTupleStructObjects(source) {
	const objects = [];
	for (const match of source.matchAll(rustTupleStructPattern)) {
		const openIndex = source.indexOf("(", match.index);
		const closeIndex = findMatchingParen(source, openIndex);
		if (closeIndex < 0) {
			continue;
		}

		const fields = splitTopLevel(
			stripComments(source.slice(openIndex + 1, closeIndex)),
			[","],
		).filter((field) => field.trim()).length;
		objects.push({
			fields,
			line: lineNumber(source, match.index),
			name: match[1],
		});
	}

	return objects;
}

function collectTypeScriptConstObjectLiterals(source) {
	const objects = [];
	for (const match of source.matchAll(typeScriptConstObjectPattern)) {
		const openIndex = source.indexOf("{", match.index);
		const closeIndex = findMatchingBrace(source, openIndex);
		if (closeIndex < 0) {
			continue;
		}

		const fields = topLevelFields(
			source.slice(openIndex + 1, closeIndex),
			"typescript",
		).length;
		if (fields === 0) {
			continue;
		}

		objects.push({
			fields,
			line: lineNumber(source, match.index),
			name: match[1],
		});
	}

	return objects;
}

function findMatchingParen(source, openIndex) {
	const state = depthState();

	for (let index = openIndex; index < source.length; index += 1) {
		const char = source[index];
		const previous = source[index - 1];

		if (trackQuote(state, char, previous)) {
			continue;
		}

		updateDepth(state, char);

		if (state.parenDepth === 0) {
			return index;
		}
	}

	return -1;
}

function fieldsFromBases(bases, fieldCounts) {
	return bases.reduce((sum, base) => sum + (fieldCounts.get(base) ?? 0), 0);
}

function mergeFieldCount(fieldCounts, name, fields) {
	const existingFields = fieldCounts.get(name);
	if (existingFields === undefined || fields > existingFields) {
		fieldCounts.set(name, fields);
		return existingFields !== fields;
	}

	return false;
}

function mergeFieldCounts(target, source) {
	let changed = false;
	for (const [name, fields] of source) {
		changed = mergeFieldCount(target, name, fields) || changed;
	}

	return changed;
}

function fieldCountsChanged(before, after) {
	if (before.size !== after.size) {
		return true;
	}

	for (const [name, fields] of after) {
		if (before.get(name) !== fields) {
			return true;
		}
	}

	return false;
}

function buildTypeScriptFieldCounts() {
	const fieldCounts = new Map();
	const typeScriptFiles = files.filter((filePath) => filePath.endsWith(".ts"));

	for (const filePath of typeScriptFiles) {
		const source = fs.readFileSync(filePath, "utf8");
		const collected = collectNamedObjectFields(source, "typescript", [
			typeScriptInterfacePattern,
			typeScriptClassPattern,
		]);
		mergeFieldCounts(fieldCounts, collected.fieldCounts);
	}

	let remainingPasses = typeScriptFiles.length;
	while (remainingPasses > 0) {
		remainingPasses -= 1;
		let changed = false;
		for (const filePath of typeScriptFiles) {
			const source = fs.readFileSync(filePath, "utf8");
			const before = new Map(fieldCounts);
			collectTypeAliasObjects(source, fieldCounts);
			changed = fieldCountsChanged(before, fieldCounts) || changed;
		}

		if (!changed) {
			break;
		}
	}

	return fieldCounts;
}

function collectObjects(filePath) {
	const source = fs.readFileSync(filePath, "utf8");
	const language = filePath.endsWith(".rs") ? "rust" : "typescript";
	checkedFiles[language] += 1;
	const patterns =
		language === "rust"
			? [rustStructPattern]
			: [typeScriptInterfacePattern, typeScriptClassPattern];
	const collected = collectNamedObjectFields(source, language, patterns);

	if (language === "typescript") {
		const fieldCounts = new Map(globalTypeScriptFieldCounts);
		mergeFieldCounts(fieldCounts, collected.fieldCounts);
		collected.objects.push(...collectTypeAliasObjects(source, fieldCounts));
		collected.objects.push(...collectTypeScriptConstObjectLiterals(source));
		for (const inherited of collected.inherited) {
			inherited.object.fields += fieldsFromBases(inherited.bases, fieldCounts);
		}
	} else {
		collected.objects.push(...collectRustTupleStructObjects(source));
		collected.objects.push(...collectRustEnumVariantObjects(source));
	}

	return collected.objects;
}

for (const root of roots) {
	walk(root);
}

globalTypeScriptFieldCounts = buildTypeScriptFieldCounts();

const offenders = [];

for (const filePath of files) {
	for (const object of collectObjects(filePath)) {
		if (object.fields > MAX_FIELDS) {
			offenders.push({ ...object, filePath });
		}
	}
}

for (const language of ["rust", "typescript"]) {
	if (checkedFiles[language] === 0) {
		console.error(
			`checked 0 ${language} files; data object field limit must cover both Rust and TypeScript`,
		);
		process.exit(1);
	}
}

if (offenders.length > 0) {
	for (const offender of offenders) {
		console.error(
			`${offender.filePath}:${offender.line} ${offender.name} has ${offender.fields} fields; ${SPLIT_HINT}`,
		);
	}
	process.exit(1);
}

console.log(
	`checked ${checkedFiles.rust} Rust files and ${checkedFiles.typescript} TypeScript files; no data objects over ${MAX_FIELDS} fields`,
);
