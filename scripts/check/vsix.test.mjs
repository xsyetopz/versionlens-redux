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
