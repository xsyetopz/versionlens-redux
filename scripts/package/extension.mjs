#!/usr/bin/env bun

import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import process from "node:process";

const checkOnly = process.argv.includes("--check");
const vscodeHostImport = /^#vscode-host$/u;
const virtualModule = /.*/u;
const vscodeRuntimeExports = [
  "CodeLens",
  "Diagnostic",
  "EventEmitter",
  "FileType",
  "Range",
  "RelativePattern",
  "TextDocumentChangeReason",
  "Uri",
  "WorkspaceEdit",
  "commands",
  "env",
  "extensions",
  "languages",
  "tasks",
  "window",
  "workspace",
];
const vscodeHostPlugin = {
  name: "vscode-host",
  setup(build) {
    build.onResolve({ filter: vscodeHostImport }, () => ({
      namespace: "vscode-host",
      path: "vscode-host",
    }));
    build.onLoad({ filter: virtualModule, namespace: "vscode-host" }, () => ({
      contents: `export { ${vscodeRuntimeExports.join(", ")} } from "vscode";`,
      loader: "js",
    }));
  },
};
let extensionDist = "packages/vscode-extension/dist";
if (checkOnly) {
  extensionDist = mkdtempSync(join(tmpdir(), "versionlens-extension-build-"));
}

if (!checkOnly) {
  rmSync(extensionDist, { force: true, recursive: true });
}

const result = await Bun.build({
  external: ["vscode"],
  entrypoints: ["packages/vscode-extension/src/extension.ts"],
  format: "cjs",
  minify: true,
  outdir: extensionDist,
  plugins: [vscodeHostPlugin],
  target: "node",
});

if (checkOnly) {
  rmSync(extensionDist, { force: true, recursive: true });
}

if (!result.success) {
  for (const log of result.logs) {
    console.error(log);
  }
  process.exit(1);
}
