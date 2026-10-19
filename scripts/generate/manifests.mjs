// Regenerates the editor file lists from the manifest table in
// `crates/versionlens-parsers`, which `versionlens-lsp --print-manifest-table`
// prints as JSON. Everything that is not derived from the table stays as is.
import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

const PACKAGE_JSON = "packages/vscode-extension/package.json";
const NEOVIM_SUPPORT = "packages/neovim-plugin/lua/versionlens/support.lua";
const JETBRAINS_SUPPORT =
  "packages/jetbrains-plugin/src/main/kotlin/com/versionlens/jetbrains/VersionLensLspServerSupportProvider.kt";
const ZED_EXTENSION = "packages/zed-extension/extension.toml";
const CONFIG_KEYS = "packages/vscode-extension/src/extension/config/keys.ts";
const TEST_PATTERNS =
  "packages/vscode-extension/src/extension/__tests__/documents/patterns.ts";
const SUPPORT_MATRIX = "tests/fixtures/manifest-support-matrix.json";

const NEOVIM_VARIABLES = [
  "supported_file_names",
  "supported_extensions",
  "supported_path_suffixes",
  "supported_name_patterns",
];
const JETBRAINS_VARIABLES = [
  "supportedFileNames",
  "supportedExtensions",
  "supportedPathSuffixes",
  "supportedNamePatterns",
];

/** Extensions that a data format implies for editor pre-filters. */
const FORMAT_EXTENSIONS = {
  json: ["json"],
  toml: ["toml"],
  yaml: ["yaml", "yml"],
  xml: ["xml"],
  custom: [],
};

/**
 * Zed language names: `[name, language ID, ...VS Code language IDs that map to
 * it]`. A row without VS Code IDs is always listed, because the table has no
 * file type for it; Zed hosts those languages for files the server reads.
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

export function readManifestTable() {
  const binary = process.env.VERSIONLENS_LSP;
  const [command, args] = binary
    ? [binary, ["--print-manifest-table"]]
    : [
        "cargo",
        [
          "run",
          "--locked",
          "-q",
          "-p",
          "versionlens-lsp",
          "--",
          "--print-manifest-table",
        ],
      ];
  const result = spawnSync(command, args, {
    cwd: ROOT,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.status !== 0) {
    throw new Error(
      `${command} --print-manifest-table failed: ${result.stderr || result.error?.message}`,
    );
  }
  return JSON.parse(result.stdout).manifests;
}

function unique(values) {
  return [...new Set(values)];
}

const ascii = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
const caseless = (a, b) => ascii(a.toLowerCase(), b.toLowerCase());

/** The ordered union of `field` over the rows of each provider. */
function perProvider(manifests, field) {
  const providers = new Map();
  for (const row of manifests) {
    const list = providers.get(row.provider) ?? [];
    list.push(...row[field]);
    providers.set(row.provider, list);
  }
  return new Map([...providers].map(([key, list]) => [key, unique(list)]));
}

/** The `files` setting default for a list of globs. */
export function filesDefault(globs) {
  return globs.length === 1 ? `**/${globs[0]}` : `**/{${globs.join(",")}}`;
}

/** The `versionlens.*` settings that the table decides, keyed by setting. */
export function derivedSettings(manifests) {
  const settings = new Map();
  for (const [provider, globs] of perProvider(manifests, "files")) {
    if (globs.length > 0) {
      settings.set(`versionlens.${provider}.files`, filesDefault(globs));
    }
  }
  for (const [provider, properties] of perProvider(
    manifests,
    "settingsProperties",
  )) {
    if (properties.length > 0) {
      settings.set(`versionlens.${provider}.dependencyProperties`, properties);
    }
  }
  return settings;
}

function languageIds(manifests) {
  return unique(manifests.flatMap((row) => row.languages));
}

function isPlainName(glob) {
  return !/[*/{}?]/u.test(glob);
}

/** Exact file names and extensions that the editor pre-filters accept. */
function editorSupport(manifests) {
  const names = [];
  const extensions = [];
  const suffixes = [];
  const patterns = [];
  for (const row of manifests) {
    names.push(
      ...row.names,
      ...row.editorNames,
      ...row.globs.filter(isPlainName),
    );
    extensions.push(
      ...row.extensions.map((extension) => extension.toLowerCase()),
      ...FORMAT_EXTENSIONS[row.format],
    );
    for (const glob of [...row.globs, ...row.files]) {
      if (/^[^*{}?]+\/[^*{}?]+$/u.test(glob)) {
        suffixes.push(glob);
      }
      const pattern = /^\*([^*/{}]+)\*\.([A-Za-z0-9]+)$/u.exec(glob);
      if (pattern) {
        patterns.push(
          `${pattern[1].toLowerCase()}.${pattern[2].toLowerCase()}`,
        );
      }
      const match = /^\*\.([A-Za-z0-9]+)$/u.exec(glob);
      if (match) {
        extensions.push(match[1].toLowerCase());
      }
    }
  }
  return {
    names: unique(names).sort(ascii),
    extensions: unique(extensions).sort(ascii),
    suffixes: unique(suffixes).sort(ascii),
    patterns: unique(patterns)
      .sort(ascii)
      .map((pattern) => {
        const dot = pattern.lastIndexOf(".");
        return [pattern.slice(0, dot), pattern.slice(dot + 1)];
      }),
  };
}

function indentOf(text, index) {
  const start = text.lastIndexOf("\n", index) + 1;
  return /^ */u.exec(text.slice(start))[0];
}

function jsonArray(values, indent) {
  const inner = `${indent}  `;
  return `[\n${values.map((value) => `${inner}${JSON.stringify(value)}`).join(",\n")}\n${indent}]`;
}

/** Where the value of `"default":` ends, for a string or array value. */
function valueEnd(text, start) {
  if (text[start] === "[") {
    return text.indexOf("]", start) + 1;
  }
  let index = start + 1;
  while (text[index] !== '"') {
    index += text[index] === "\\" ? 2 : 1;
  }
  return index + 1;
}

function patchDefault(text, key, value) {
  const keyIndex = text.indexOf(`\n        "${key}": {`);
  if (keyIndex < 0) {
    throw new Error(`${PACKAGE_JSON} has no setting ${key}`);
  }
  const blockEnd = text.indexOf("\n        }", keyIndex);
  const marker = text.indexOf('"default": ', keyIndex);
  if (marker < 0 || marker > blockEnd) {
    throw new Error(`${key} has no default`);
  }
  const start = marker + '"default": '.length;
  const end = valueEnd(text, start);
  const replacement = Array.isArray(value)
    ? jsonArray(value, indentOf(text, marker))
    : JSON.stringify(value);
  return text.slice(0, start) + replacement + text.slice(end);
}

function patchActivationEvents(text, manifests) {
  const marker = '"activationEvents": [';
  const start = text.indexOf(marker);
  if (start < 0) {
    throw new Error(`${PACKAGE_JSON} lacks activationEvents`);
  }
  const end = text.indexOf("]", start);
  const existing = JSON.parse(text.slice(start + marker.length - 1, end + 1));
  const languages = new Set(languageIds(manifests));
  const kept = existing.filter(
    (event) =>
      event.startsWith("onLanguage:") &&
      languages.has(event.slice("onLanguage:".length)),
  );
  const added = [...languages]
    .map((language) => `onLanguage:${language}`)
    .filter((event) => !kept.includes(event));
  const others = existing.filter((event) => !event.startsWith("onLanguage:"));
  const events = [...kept, ...added, ...others];
  return (
    text.slice(0, start + marker.length - 1) +
    jsonArray(events, indentOf(text, start)) +
    text.slice(end + 1)
  );
}

export function generatePackageJson(text, manifests) {
  let result = text;
  const present = new Set(
    [...text.matchAll(/\n {8}"(versionlens\.[^"]+)": \{/gu)].map(
      (match) => match[1],
    ),
  );
  for (const [key, value] of derivedSettings(manifests)) {
    if (!present.has(key)) {
      throw new Error(
        `${PACKAGE_JSON} lacks ${key}; add its schema by hand, then rerun`,
      );
    }
    result = patchDefault(result, key, value);
  }
  return patchActivationEvents(result, manifests);
}

export function generateNeovim(text, manifests) {
  const { names, extensions, suffixes, patterns } = editorSupport(manifests);
  const replace = (source, variable, lines) => {
    const pattern = new RegExp(
      `(local ${variable} = \\{\\n)(?: {2}.*\\n)*\\}\\n`,
      "u",
    );
    if (!pattern.test(source)) {
      throw new Error(`${NEOVIM_SUPPORT} lacks ${variable}`);
    }
    return source.replace(
      pattern,
      (_, open) => `${open}${lines.join("\n")}\n}\n`,
    );
  };
  let result = replace(
    text,
    "supported_file_names",
    names.map((name) => `  [${JSON.stringify(name)}] = true,`),
  );
  result = replace(
    result,
    "supported_extensions",
    extensions.map((extension) => `  ${extension} = true,`),
  );
  result = replace(
    result,
    "supported_path_suffixes",
    suffixes.map((suffix) => `  ${JSON.stringify(suffix)},`),
  );
  return replace(
    result,
    "supported_name_patterns",
    patterns.map(
      ([contains, extension]) =>
        `  { contains = ${JSON.stringify(contains)}, extension = ${JSON.stringify(extension)} },`,
    ),
  );
}

export function generateJetBrains(text, manifests) {
  const { names, extensions, suffixes, patterns } = editorSupport(manifests);
  const replace = (source, variable, values) => {
    const pattern = new RegExp(
      `(private val ${variable} = (?:setOf|listOf<[^(]*>)\\(\\n)(?: {12}.*\\n)* {8}\\)`,
      "u",
    );
    if (!pattern.test(source)) {
      throw new Error(`${JETBRAINS_SUPPORT} lacks ${variable}`);
    }
    return source.replace(
      pattern,
      (_, open) =>
        `${open}${values.map((value) => `            ${value},`).join("\n")}\n        )`,
    );
  };
  const quoted = (values) => values.map((value) => JSON.stringify(value));
  let result = replace(text, "supportedFileNames", quoted(names));
  result = replace(result, "supportedExtensions", quoted(extensions));
  result = replace(result, "supportedPathSuffixes", quoted(suffixes));
  return replace(
    result,
    "supportedNamePatterns",
    patterns.map(
      ([contains, extension]) =>
        `${JSON.stringify(contains)} to ${JSON.stringify(extension)}`,
    ),
  );
}

export function generateZed(text, manifests) {
  const ids = new Set(languageIds(manifests));
  const rows = ZED_LANGUAGES.filter(
    ([, , ...sources]) =>
      sources.length === 0 || sources.some((source) => ids.has(source)),
  ).sort((a, b) => caseless(a[0], b[0]));
  const languages = `languages = [\n${rows.map(([name]) => `  ${JSON.stringify(name)},`).join("\n")}\n]`;
  const languageIdLines = rows
    .map(([name, id]) => `${JSON.stringify(name)} = ${JSON.stringify(id)}`)
    .join("\n");
  const listPattern = /languages = \[\n[\s\S]*?\n\]/u;
  const idsPattern =
    /(\[language_servers\.versionlens\.language_ids\]\n)[\s\S]*$/u;
  if (!(listPattern.test(text) && idsPattern.test(text))) {
    throw new Error(`${ZED_EXTENSION} lacks the language lists`);
  }
  return text
    .replace(listPattern, languages)
    .replace(idsPattern, (_, header) => `${header}${languageIdLines}\n`);
}

/** The CodeLens document selector languages of each provider. */
export function generateConfigKeys(text, manifests) {
  const start = text.indexOf("const filePatternKeys");
  const end = text.indexOf("function enabledFilePatternKeys");
  if (start < 0 || end < start) {
    throw new Error(`${CONFIG_KEYS} lacks the filePatternKeys table`);
  }
  let block = text.slice(start, end);
  for (const [provider, languages] of perProvider(manifests, "languages")) {
    if (languages.length === 0) {
      continue;
    }
    const pattern = new RegExp(
      `(\\[\\s*"${provider}",\\s*"${provider}\\.files",\\s*)\\[[^\\]]*\\]`,
      "u",
    );
    if (!pattern.test(block)) {
      throw new Error(
        `${CONFIG_KEYS} has no filePatternKeys row for ${provider}`,
      );
    }
    block = block.replace(
      pattern,
      (_, open) =>
        `${open}[${languages.map((l) => JSON.stringify(l)).join(", ")}]`,
    );
  }
  return formatWithBiome(
    CONFIG_KEYS,
    text.slice(0, start) + block + text.slice(end),
  );
}

const TEST_PATTERN_ROW =
  /\[\s*"(\w+)\.files",\s*"([^"]*)",\s*(\[[^\]]*\]),?\s*\]/gu;

/** The test fixture rows of the default `files` patterns and languages. */
export function generateTestPatterns(text, manifests) {
  const languages = perProvider(manifests, "languages");
  const globs = perProvider(manifests, "files");
  const marker = "const defaultFilePatternEntries = [";
  const start = text.indexOf(marker);
  const end = text.indexOf("] as const;", start);
  if (start < 0 || end < 0) {
    throw new Error(`${TEST_PATTERNS} lacks defaultFilePatternEntries`);
  }
  const body = text
    .slice(start + marker.length, end)
    .replace(TEST_PATTERN_ROW, (_row, provider) => {
      const files = globs.get(provider);
      if (!files?.length) {
        throw new Error(
          `${TEST_PATTERNS}: the table has no files for ${provider}`,
        );
      }
      const ids = JSON.stringify(languages.get(provider) ?? []);
      return `["${provider}.files", ${JSON.stringify(filesDefault(files))}, ${ids}]`;
    });
  return formatWithBiome(
    TEST_PATTERNS,
    text.slice(0, start + marker.length) + body + text.slice(end),
  );
}

/** The `defaultGlob` and `languages` of each provider in the support matrix. */
export function generateSupportMatrix(text, manifests) {
  const languages = perProvider(manifests, "languages");
  const globs = perProvider(manifests, "files");
  const matrix = JSON.parse(text);
  const listed = new Set(matrix.entries.map((entry) => entry.provider));
  for (const [provider, files] of globs) {
    if (files.length > 0 && !listed.has(provider)) {
      throw new Error(
        `${SUPPORT_MATRIX}: no entry for ${provider}; add it by hand, then rerun`,
      );
    }
  }
  for (const entry of matrix.entries) {
    for (const manifest of entry.manifests) {
      const provider = entry.provider;
      if (!(globs.get(provider)?.length && languages.has(provider))) {
        throw new Error(
          `${SUPPORT_MATRIX}: the table has no row for ${provider}`,
        );
      }
      manifest.defaultGlob = filesDefault(globs.get(provider));
      manifest.languages = languages.get(provider);
    }
  }
  return formatWithBiome(
    SUPPORT_MATRIX,
    `${JSON.stringify(matrix, null, 2)}\n`,
  );
}

/** Formats generated text as the repository formatter would. */
function formatWithBiome(path, text) {
  const biome = join(ROOT, "node_modules", "@biomejs", "biome", "bin", "biome");
  const result = spawnSync(
    process.execPath,
    [biome, "format", `--stdin-file-path=${path}`],
    { cwd: ROOT, encoding: "utf8", input: text },
  );
  if (result.status !== 0) {
    throw new Error(
      `biome could not format ${path}: ${result.error?.message ?? result.stderr}`,
    );
  }
  return result.stdout;
}

// The glob alternatives of a `files` default, without the leading globstar.
function globsOf(value) {
  if (typeof value !== "string") {
    return [JSON.stringify(value)];
  }
  const bare = value.replace(/^\*\*\//u, "");
  const braced = /^\{(.*)\}$/u.exec(bare);
  return braced ? braced[1].split(",") : [bare];
}

/** The named lists of each generated file, read back out of its text. */

function packageJsonEntries(text, manifests) {
  const json = JSON.parse(text);
  const properties = json.contributes.configuration.properties;
  const entries = new Map();
  for (const key of derivedSettings(manifests).keys()) {
    const value = properties[key]?.default;
    entries.set(
      `${key} default`,
      Array.isArray(value) ? value.map(String) : globsOf(value),
    );
  }
  entries.set(
    "activationEvents onLanguage:*",
    json.activationEvents.filter((event) => event.startsWith("onLanguage:")),
  );
  return entries;
}

function configKeysEntries(text, manifests) {
  const entries = new Map();
  for (const provider of perProvider(manifests, "languages").keys()) {
    const row = new RegExp(
      `\\[\\s*"${provider}",\\s*"${provider}\\.files",\\s*(\\[[^\\]]*\\])`,
      "u",
    ).exec(text);
    entries.set(`${provider} languages`, row ? JSON.parse(row[1]) : []);
  }
  return entries;
}

function testPatternsEntries(text) {
  const entries = new Map();
  for (const [, provider, glob, languages] of text.matchAll(TEST_PATTERN_ROW)) {
    entries.set(`${provider}.files default`, globsOf(glob));
    entries.set(`${provider}.files languages`, JSON.parse(languages));
  }
  return entries;
}

function supportMatrixEntries(text) {
  const entries = new Map();
  for (const { provider, manifests } of JSON.parse(text).entries) {
    for (const [index, manifest] of manifests.entries()) {
      const key = `${provider} manifests[${index}]`;
      entries.set(`${key} defaultGlob`, globsOf(manifest.defaultGlob));
      entries.set(`${key} languages`, manifest.languages);
    }
  }
  return entries;
}

function blockEntries(text, variables, pattern) {
  const entries = new Map();
  for (const variable of variables) {
    const match = pattern(variable).exec(text);
    entries.set(
      variable,
      match
        ? match[1]
            .split("\n")
            .map((line) => line.trim().replace(/,$/u, ""))
            .filter(Boolean)
        : [],
    );
  }
  return entries;
}

const neovimEntries = (text) =>
  blockEntries(
    text,
    NEOVIM_VARIABLES,
    (variable) =>
      new RegExp(`local ${variable} = \\{\\n((?: {2}.*\\n)*)\\}\\n`, "u"),
  );

const jetbrainsEntries = (text) =>
  blockEntries(
    text,
    JETBRAINS_VARIABLES,
    (variable) =>
      new RegExp(
        `private val ${variable} = (?:setOf|listOf<[^(]*>)\\(\\n((?: {12}.*\\n)*) {8}\\)`,
        "u",
      ),
  );

function zedEntries(text) {
  const languages = /languages = \[\n([\s\S]*?)\n\]/u.exec(text);
  const ids =
    /\[language_servers\.versionlens\.language_ids\]\n([\s\S]*)$/u.exec(text);
  const lines = (block) =>
    block
      ? block
          .split("\n")
          .map((line) => line.trim().replace(/,$/u, ""))
          .filter(Boolean)
      : [];
  return new Map([
    ["languages", lines(languages?.[1])],
    ["language_servers.versionlens.language_ids", lines(ids?.[1])],
  ]);
}

export function generateAll(manifests) {
  const targets = [
    [PACKAGE_JSON, generatePackageJson, packageJsonEntries],
    [NEOVIM_SUPPORT, generateNeovim, neovimEntries],
    [JETBRAINS_SUPPORT, generateJetBrains, jetbrainsEntries],
    [ZED_EXTENSION, generateZed, zedEntries],
    [CONFIG_KEYS, generateConfigKeys, configKeysEntries],
    [TEST_PATTERNS, generateTestPatterns, testPatternsEntries],
    [SUPPORT_MATRIX, generateSupportMatrix, supportMatrixEntries],
  ];
  return targets.map(([path, generate, entriesOf]) => {
    const current = readFileSync(join(ROOT, path), "utf8");
    return {
      path,
      current,
      next: generate(current, manifests),
      entriesOf: (text) => entriesOf(text, manifests),
    };
  });
}

const preview = (items) =>
  items.length > 6 ? `${items.slice(0, 6).join(", ")}, ...` : items.join(", ");

/** One message per generated entry that differs from the committed one. */
export function findDrift({ path, current, next, entriesOf }) {
  const expected = entriesOf(next);
  const actual = entriesOf(current);
  const drift = [];
  for (const [name, want] of expected) {
    const have = actual.get(name) ?? [];
    if (JSON.stringify(want) === JSON.stringify(have)) {
      continue;
    }
    const missing = want.filter((item) => !have.includes(item));
    const extra = have.filter((item) => !want.includes(item));
    const detail = [
      missing.length > 0 && `missing: ${preview(missing)}`,
      extra.length > 0 && `unexpected: ${preview(extra)}`,
      missing.length + extra.length === 0 && "order differs",
    ]
      .filter(Boolean)
      .join("; ");
    drift.push(
      `${path}: ${name}\n    ${detail}\n    expected: ${preview(want)}\n    actual:   ${preview(have)}`,
    );
  }
  if (drift.length === 0 && next !== current) {
    drift.push(`${path}: formatting differs from the generated output`);
  }
  return drift;
}

if (import.meta.main) {
  const check = process.argv.includes("--check");
  let results;
  try {
    results = generateAll(readManifestTable());
  } catch (error) {
    console.error(`manifests: ${error.message}`);
    process.exit(1);
  }
  if (check) {
    const drift = results.flatMap(findDrift);
    if (drift.length > 0) {
      console.error(
        `generated lists drifted from the manifest table:\n${drift.join("\n")}\nrun \`bun scripts/generate/manifests.mjs\` to regenerate`,
      );
      process.exit(1);
    }
    console.log(
      `manifests: ${results.length} generated files match the manifest table`,
    );
  } else {
    for (const { path, current, next } of results) {
      if (next !== current) {
        writeFileSync(join(ROOT, path), next);
        console.log(`updated ${path}`);
      }
    }
  }
}
