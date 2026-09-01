import { readFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

import { resolveNativeEditorTarget } from "../package/editor-targets.mjs";

const { expect, it } = Bun.jest(import.meta.path);

it("VSIX checker fails when the requested archive is missing", () => {
  const missing = join(
    process.cwd(),
    "target",
    "missing-versionlens-package.vsix",
  );
  const result = Bun.spawnSync(["bun", "scripts/check/vsix.mjs", missing]);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain(`VSIX does not exist: ${missing}`);
});

it("editor packaging runs the complete VSIX and editor freshness contracts", () => {
  const source = readFileSync("scripts/package/editors.mjs", "utf8");
  const packageBuild = source.indexOf('run(["bun", "run", "package"]');
  const vsixCheck = source.indexOf("run(vsixFreshnessCommand");
  const neovimPackage = source.indexOf(
    'run(["bun", "scripts/package/neovim.mjs"]',
  );
  const editorCheck = source.indexOf("run(editorFreshnessCommand");

  expect(packageBuild).toBeGreaterThanOrEqual(0);
  expect(vsixCheck).toBeGreaterThan(packageBuild);
  expect(neovimPackage).toBeGreaterThan(vsixCheck);
  expect(editorCheck).toBeGreaterThan(neovimPackage);
  expect(source).toContain(
    'const vsixFreshnessCommand = ["bun", "scripts/check/vsix.mjs"]',
  );
  expect(source).toContain(
    'const editorFreshnessCommand = ["bun", "scripts/check/editors.mjs"]',
  );
});

it("native editor targets require the matching Rust target", () => {
  const host = resolveNativeEditorTarget([], "Neovim");
  expect(host.editorTarget).toBe(`${process.platform}-${process.arch}`);
  expect(() =>
    resolveNativeEditorTarget(
      [process.platform, process.arch, "invalid-rust-target"],
      "Neovim",
    ),
  ).toThrow("requires Rust target");
});
