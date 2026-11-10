import { readTargets } from "../package/targets.mjs";
import { caseless, editorSupport, languageIds } from "./manifest-table.mjs";

const NEOVIM_SUPPORT = "packages/neovim-plugin/lua/versionlens/support.lua";
const NEOVIM_DOWNLOAD = "packages/neovim-plugin/lua/versionlens/download.lua";
const JETBRAINS_SUPPORT =
	"packages/jetbrains-plugin/src/main/kotlin/com/versionlens/jetbrains/VersionLensLspServerSupportProvider.kt";
const ZED_EXTENSION = "packages/zed-extension/extension.toml";

/**
 * Zed language names: `[name, language ID, ...VS Code language IDs that map to
 * it]`. A row without VS Code IDs is always listed, because the table has no
 * file type for it; Zed hosts those languages for files the server reads.
 * @type {[string, string, ...string[]][]}
 */
const ZED_LANGUAGES = [
	["C", "c"],
	["C#", "csharp"],
	["C++", "cpp"],
	["Clojure", "clojure", "clojure"],
	["Dockerfile", "dockerfile", "dockerfile"],
	["Elixir", "elixir", "elixir"],
	["Erlang", "erlang", "erlang"],
	["Gleam", "gleam"],
	["Go", "go", "go.mod"],
	["HCL", "terraform", "terraform", "hcl"],
	["Java", "java"],
	["JavaScript", "javascript"],
	["JSON", "json", "json"],
	["JSONC", "jsonc", "jsonc"],
	["Kotlin", "kotlin", "kotlin"],
	["Lua", "lua", "lua"],
	["Nix", "nix", "nix"],
	["Plain Text", "plaintext", "plaintext"],
	["Python", "python", "python"],
	["Ruby", "ruby", "ruby"],
	["Rust", "rust"],
	["Scala", "scala", "scala"],
	["Shell Script", "shellscript"],
	["Swift", "swift", "swift"],
	["TOML", "toml", "toml"],
	["TSX", "typescriptreact"],
	["TypeScript", "typescript"],
	["XML", "xml", "xml"],
	["YAML", "yaml", "yaml"],
	["Zig", "zig", "zig"],
];

/**
 * Replace the first match of `pattern`, whose first group is kept as the
 * opening of the replacement built by `build`.
 * @param {string} source
 * @param {RegExp} pattern
 * @param {string} lacks what the error names when the pattern is missing
 * @param {(open: string) => string} build
 * @returns {string}
 */
const replaceBlock = (source, pattern, lacks, build) => {
	const match = pattern.exec(source);
	if (match === null) {
		throw new Error(lacks);
	}
	return (
		source.slice(0, match.index) +
		build(match[1] ?? "") +
		source.slice(match.index + match[0].length)
	);
};

/**
 * @param {string} source
 * @param {string} variable
 * @param {string[]} lines
 * @returns {string}
 */
const replaceLuaTable = (source, variable, lines) =>
	replaceBlock(
		source,
		new RegExp(`(local ${variable} = \\{\\n)(?: {2}.*\\n)*\\}\\n`, "u"),
		`${NEOVIM_SUPPORT} lacks ${variable}`,
		(open) => `${open}${lines.join("\n")}\n}\n`,
	);

/**
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {string}
 */
const generateNeovim = (text, manifests) => {
	const { names, extensions, suffixes, patterns } = editorSupport(manifests);
	let result = replaceLuaTable(
		text,
		"supported_file_names",
		names.map((name) => `  [${JSON.stringify(name)}] = true,`),
	);
	result = replaceLuaTable(
		result,
		"supported_extensions",
		extensions.map((extension) => `  ${extension} = true,`),
	);
	result = replaceLuaTable(
		result,
		"supported_path_suffixes",
		suffixes.map((suffix) => `  ${JSON.stringify(suffix)},`),
	);
	return replaceLuaTable(
		result,
		"supported_name_patterns",
		patterns.map(
			([contains, extension]) =>
				`  { contains = ${JSON.stringify(contains)}, extension = ${JSON.stringify(extension)} },`,
		),
	);
};

/**
 * Release target triple per editor target, from scripts/package/targets.json.
 * @param {string} text
 * @returns {Promise<string>}
 */
const generateNeovimTargets = async (text) => {
	const targets = await readTargets();
	const lines = targets.map(
		({ vscode, rust }) =>
			`  [${JSON.stringify(vscode)}] = ${JSON.stringify(rust)},`,
	);
	return replaceBlock(
		text,
		/(local triples = \{\n)(?: {2}.*\n)*\}\n/u,
		`${NEOVIM_DOWNLOAD} lacks triples`,
		(open) => `${open}${lines.join("\n")}\n}\n`,
	);
};

/**
 * Editor targets the JetBrains plugin bundles, from scripts/package/targets.json.
 * @param {string} text
 * @returns {Promise<string>}
 */
const generateJetBrainsTargets = async (text) => {
	const targets = await readTargets();
	const lines = targets.map(
		({ vscode }) => `            ${JSON.stringify(vscode)},`,
	);
	return replaceBlock(
		text,
		/(private val releaseTargets = setOf\(\n)(?: {12}.*\n)* {8}\)/u,
		`${JETBRAINS_SUPPORT} lacks releaseTargets`,
		(open) => `${open}${lines.join("\n")}\n        )`,
	);
};

/**
 * @param {string} source
 * @param {string} variable
 * @param {string[]} values
 * @returns {string}
 */
const replaceKotlinList = (source, variable, values) =>
	replaceBlock(
		source,
		new RegExp(
			`(private val ${variable} = (?:setOf|listOf<[^(]*>)\\(\\n)(?: {12}.*\\n)* {8}\\)`,
			"u",
		),
		`${JETBRAINS_SUPPORT} lacks ${variable}`,
		(open) =>
			`${open}${values.map((value) => `            ${value},`).join("\n")}\n        )`,
	);

/**
 * @param {string[]} values
 * @returns {string[]}
 */
const quoted = (values) => values.map((value) => JSON.stringify(value));

/**
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {string}
 */
const generateJetBrains = (text, manifests) => {
	const { names, extensions, suffixes, patterns } = editorSupport(manifests);
	let result = replaceKotlinList(text, "supportedFileNames", quoted(names));
	result = replaceKotlinList(result, "supportedExtensions", quoted(extensions));
	result = replaceKotlinList(result, "supportedPathSuffixes", quoted(suffixes));
	return replaceKotlinList(
		result,
		"supportedNamePatterns",
		patterns.map(
			([contains, extension]) =>
				`${JSON.stringify(contains)} to ${JSON.stringify(extension)}`,
		),
	);
};

/**
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {string}
 */
const generateZed = (text, manifests) => {
	const ids = new Set(languageIds(manifests));
	const rows = ZED_LANGUAGES.filter(
		(row) => row.length === 2 || row.slice(2).some((source) => ids.has(source)),
	).toSorted((a, b) => caseless(a[0], b[0]));
	const languages = `languages = [\n${rows.map(([name]) => `  ${JSON.stringify(name)},`).join("\n")}\n]`;
	const languageIdLines = rows
		.map(([name, id]) => `${JSON.stringify(name)} = ${JSON.stringify(id)}`)
		.join("\n");
	const listPattern = /languages = \[\n[\s\S]*?\n\]/u;
	const idsPattern =
		/(\[language_servers\.versionlens\.language_ids\]\n)[\s\S]*$/u;
	return replaceBlock(
		replaceBlock(
			text,
			new RegExp(`(${listPattern.source})`, "u"),
			`${ZED_EXTENSION} lacks the language lists`,
			() => languages,
		),
		idsPattern,
		`${ZED_EXTENSION} lacks the language lists`,
		(header) => `${header}${languageIdLines}\n`,
	);
};

export {
	generateJetBrains,
	generateJetBrainsTargets,
	generateNeovim,
	generateNeovimTargets,
	generateZed,
	JETBRAINS_SUPPORT,
	NEOVIM_DOWNLOAD,
	NEOVIM_SUPPORT,
	ZED_EXTENSION,
};
