#!/usr/bin/env bun
// Runs the extension in a real VS Code against the built server and a local
// fake npm registry, so it needs no network once VS Code is downloaded.

import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import process from "node:process";
import { runTests } from "@vscode/test-electron";

const root = resolve(import.meta.dir, "../..");
const extensionPath = join(root, "packages/vscode-extension");
const serverName =
  process.platform === "win32" ? "versionlens-lsp.exe" : "versionlens-lsp";
const serverPath =
  process.env.VERSIONLENS_LSP ?? join(root, "target/debug", serverName);

const packument = JSON.stringify({
  "dist-tags": { latest: "2.0.0" },
  versions: { "1.0.0": {}, "1.1.0": {}, "2.0.0": {} },
});
const registry = createServer((_request, response) => {
  response.setHeader("content-type", "application/json");
  response.end(packument);
});
await new Promise((done) => registry.listen(0, "127.0.0.1", done));
const { port } = registry.address();

// Resolved, because VS Code reports the real path of files and the server
// compares document and folder URIs as written.
const scratch = realpathSync(mkdtempSync(join(tmpdir(), "versionlens-e2e-")));
const workspace = join(scratch, "workspace");
cpSync(join(root, "tests/fixtures/vscode-e2e"), workspace, {
  recursive: true,
});
mkdirSync(join(workspace, ".vscode"));
writeFileSync(
  join(workspace, ".vscode/settings.json"),
  JSON.stringify({
    "versionlens.npm.apiUrl": `http://127.0.0.1:${port}`,
    "versionlens.suggestions.showOnStartup": true,
    "versionlens.suggestions.showVulnerabilities": false,
  }),
);

// Bundle the test entry; VS Code loads it as CommonJS.
const built = await Bun.build({
  entrypoints: [join(root, "tests/vscode-extension/e2e.ts")],
  external: ["vscode"],
  format: "cjs",
  outdir: join(scratch, "tests"),
  target: "node",
});
if (!built.success) {
  console.error(built.logs.join("\n"));
  process.exit(1);
}

let code = 0;
try {
  await runTests({
    cachePath: join(root, ".vscode-test"),
    extensionDevelopmentPath: extensionPath,
    extensionTestsEnv: { VERSIONLENS_LSP: serverPath },
    extensionTestsPath: join(scratch, "tests/e2e.js"),
    launchArgs: [
      workspace,
      "--disable-extensions",
      // Headless Linux CI: no GPU under Xvfb and no keyring on D-Bus.
      "--disable-gpu",
      "--disable-workspace-trust",
      "--use-inmemory-secretstorage",
      "--skip-welcome",
      "--skip-release-notes",
      "--logsPath",
      join(scratch, "logs"),
      "--user-data-dir",
      join(scratch, "user-data"),
    ],
  });
} catch (error) {
  console.error(error);
  code = 1;
} finally {
  registry.close();
  if (code === 0) {
    rmSync(scratch, { force: true, recursive: true });
  } else {
    printLogs(join(scratch, "logs"));
    console.error(`Logs are kept in ${scratch}`);
  }
}
process.exit(code);

// Print VS Code's main, renderer and extension host logs so CI failures
// show the cause without downloading artifacts.
function printLogs(directory) {
  let entries;
  try {
    entries = readdirSync(directory, { recursive: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    const name = String(entry);
    if (!/(^|\/)(main|renderer\d*|exthost\d*)\.log$/.test(name)) continue;
    console.error(`--- ${name} ---`);
    console.error(readFileSync(join(directory, name), "utf8"));
  }
}
