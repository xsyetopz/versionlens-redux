import { readFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

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
  const editorCheck = source.indexOf("run(editorFreshnessCommand");

  expect(packageBuild).toBeGreaterThanOrEqual(0);
  expect(vsixCheck).toBeGreaterThan(packageBuild);
  expect(editorCheck).toBeGreaterThan(vsixCheck);
  expect(source).toContain(
    'const vsixFreshnessCommand = ["bun", "scripts/check/vsix.mjs"]',
  );
  expect(source).toContain(
    'const editorFreshnessCommand = ["bun", "scripts/check/editors.mjs"]',
  );
});
