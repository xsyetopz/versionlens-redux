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
        ["run", "-q", "-p", "versionlens-lsp", "--", "--print-manifest-table"],
      ];
  const result = spawnSync(command, args, {
    cwd: ROOT,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.status !== 0) {
    throw new Error(
      `${command} --print-manifest-table failed: ${result.stderr || result.error}`,
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
      const match = /^\*\.([A-Za-z0-9]+)$/u.exec(glob);
      if (match) {
        extensions.push(match[1].toLowerCase());
      }
    }
  }
  return {
    names: unique(names).sort(ascii),
    extensions: unique(extensions).sort(ascii),
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
  const { names, extensions } = editorSupport(manifests);
  const replace = (source, variable, lines) => {
    const pattern = new RegExp(
      `(local ${variable} = \\{\\n)[\\s\\S]*?(\\n\\}\\n)`,
      "u",
    );
    if (!pattern.test(source)) {
      throw new Error(`${NEOVIM_SUPPORT} lacks ${variable}`);
    }
    return source.replace(
      pattern,
      (_, open, close) => `${open}${lines.join("\n")}${close}`,
    );
  };
  return replace(
    replace(
      text,
      "supported_file_names",
      names.map((name) => `  [${JSON.stringify(name)}] = true,`),
    ),
    "supported_extensions",
    extensions.map((extension) => `  ${extension} = true,`),
  );
}

export function generateJetBrains(text, manifests) {
  const { names, extensions } = editorSupport(manifests);
  const replace = (source, variable, values) => {
    const pattern = new RegExp(
      `(private val ${variable} = setOf\\(\\n)[\\s\\S]*?(\\n {8}\\))`,
      "u",
    );
    if (!pattern.test(source)) {
      throw new Error(`${JETBRAINS_SUPPORT} lacks ${variable}`);
    }
    return source.replace(
      pattern,
      (_, open, close) =>
        `${open}${values.map((value) => `            ${JSON.stringify(value)},`).join("\n")}${close}`,
    );
  };
  return replace(
    replace(text, "supportedFileNames", names),
    "supportedExtensions",
    extensions,
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

export function generateAll(manifests) {
  const targets = [
    [PACKAGE_JSON, generatePackageJson],
    [NEOVIM_SUPPORT, generateNeovim],
    [JETBRAINS_SUPPORT, generateJetBrains],
    [ZED_EXTENSION, generateZed],
  ];
  return targets.map(([path, generate]) => {
    const current = readFileSync(join(ROOT, path), "utf8");
    return { path, current, next: generate(current, manifests) };
  });
}

if (import.meta.main) {
  const manifests = readManifestTable();
  for (const { path, current, next } of generateAll(manifests)) {
    if (next !== current) {
      writeFileSync(join(ROOT, path), next);
      console.log(`updated ${path}`);
    }
  }
}
