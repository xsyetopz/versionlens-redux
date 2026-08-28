const { expect, it } = Bun.jest(import.meta.path);

it("release check derives and validates every root Cargo workspace package", async () => {
  const metadata = Bun.spawnSync([
    "cargo",
    "metadata",
    "--no-deps",
    "--format-version",
    "1",
  ]);
  expect(metadata.exitCode).toBe(0);
  const workspace = JSON.parse(metadata.stdout.toString());
  const packageJson = await Bun.file("package.json").json();
  const result = Bun.spawnSync([
    "bun",
    "scripts/release/version.mjs",
    packageJson.version,
    "--check",
  ]);
  expect(result.exitCode).toBe(0);
  expect(
    workspace.packages.some(({ name }) => name === "versionlens-test-support"),
  ).toBe(true);
});
