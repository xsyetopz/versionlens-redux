import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { analyzeSources } from "./detection/analysis.mjs";
import { applyExceptions } from "./exceptions.mjs";
import { formatExceptions, formatFindings } from "./report.mjs";

const SOURCE_ROOTS = ["crates", "packages/vscode-extension/src", "tests"];
const IGNORED_DIRS = new Set(["dist", "node_modules", "target"]);
const REPOSITORY_OPTIONS = {
  complexTypeScope: "file",
  duplicateMinTokens: 30,
  ignoreCommonComplexTypes: true,
  ignorePublicApiTypes: true,
  ignorePublicPassThroughWrappers: true,
  ignoreTestFilesForDuplicates: true,
  ignoreTestFilesForPassThrough: true,
  ignoreTestFilesForQualifiedPaths: true,
};

function appendSourceFile(filePath, extension, files) {
  if (extension !== ".rs" && extension !== ".ts") {
    return;
  }
  let language = "typescript";
  if (extension === ".rs") {
    language = "rust";
  }
  files.push({
    language,
    path: filePath,
    source: fs.readFileSync(filePath, "utf8"),
  });
}

function visitEntry(directory, entry, files) {
  if (IGNORED_DIRS.has(entry.name)) {
    return;
  }
  const filePath = path.join(directory, entry.name);
  if (entry.isDirectory()) {
    walk(filePath, files);
  } else {
    appendSourceFile(filePath, path.extname(entry.name), files);
  }
}

function walk(directory, files) {
  if (!fs.existsSync(directory)) {
    return;
  }
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    visitEntry(directory, entry, files);
  }
}

function executableExists(command) {
  return (
    spawnSync("/bin/sh", ["-lc", `command -v ${command}`], {
      encoding: "utf8",
    }).status === 0
  );
}

export function collectRepositoryFiles(roots = SOURCE_ROOTS) {
  const files = [];
  for (const root of roots) {
    walk(root, files);
  }
  return files;
}

export function runRepositoryCheck(options = {}) {
  const files = collectRepositoryFiles(options.roots);
  if (files.length === 0) {
    return { error: "no source files checked", exitCode: 1 };
  }
  let checked;
  try {
    checked = applyExceptions(analyzeSources(files, REPOSITORY_OPTIONS));
  } catch (error) {
    return { error: error.message, exitCode: 1 };
  }
  const hasFindings = Object.values(checked.remaining).some(
    (findings) => findings.length > 0,
  );
  const exceptionOutput = formatExceptions(checked.applied);
  if (!hasFindings) {
    return { exitCode: 0, output: exceptionOutput };
  }

  let diffCommand = "diff";
  if (executableExists("difft")) {
    diffCommand = "difft";
  }
  if (!executableExists(diffCommand)) {
    return { error: `missing command: ${diffCommand}`, exitCode: 1 };
  }
  return {
    exitCode: 1,
    output: [
      formatFindings(checked.remaining, { diffCommand }),
      exceptionOutput,
    ]
      .filter(Boolean)
      .join("\n"),
  };
}
