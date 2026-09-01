import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const { expect, it } = Bun.jest(import.meta.path);

it("lists every required marketplace secret without making hosted changes", () => {
  const result = Bun.spawnSync([
    "bun",
    "scripts/release/configure-marketplaces.mjs",
    "--dry-run",
    "--repo",
    "xsyetopz/versionlens-redux",
  ]);
  expect(result.exitCode).toBe(0);
  const output = result.stdout.toString();
  for (const name of [
    "AZURE_CLIENT_ID",
    "AZURE_TENANT_ID",
    "AZURE_SUBSCRIPTION_ID",
    "JETBRAINS_MARKETPLACE_TOKEN",
    "JB_CERTIFICATE_CHAIN",
    "JB_PRIVATE_KEY",
    "JB_PRIVATE_KEY_PASSWORD",
    "ZED_EXTENSIONS_FORK",
    "ZED_EXTENSIONS_TOKEN",
    "LUAROCKS_API_KEY",
  ]) {
    expect(output).toContain(`marketplaces: ${name}`);
  }
});

it("adds and updates the VersionLens Zed registry entry", () => {
  const directory = mkdtempSync(join(tmpdir(), "versionlens-zed-registry-"));
  const registry = join(directory, "extensions.toml");
  try {
    writeFileSync(
      registry,
      '[alpha]\nsubmodule = "extensions/alpha"\nversion = "1.0.0"\n',
    );
    for (const version of ["0.4.0", "0.4.1"]) {
      const result = Bun.spawnSync([
        "bun",
        "scripts/release/zed-registry.mjs",
        registry,
        version,
      ]);
      expect(result.exitCode).toBe(0);
    }
    const contents = readFileSync(registry, "utf8");
    expect(contents.match(/\[versionlens-lsp\]/gu)).toHaveLength(1);
    expect(contents).toContain('path = "packages/zed-extension"');
    expect(contents).toContain('version = "0.4.1"');
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});

it("renders a versioned LuaRocks specification", () => {
  const directory = mkdtempSync(join(tmpdir(), "versionlens-rockspec-"));
  try {
    const result = Bun.spawnSync([
      "bun",
      "scripts/release/luarocks.mjs",
      "0.4.0",
      directory,
    ]);
    expect(result.exitCode).toBe(0);
    const rockspec = readFileSync(
      join(directory, "versionlens-redux-0.4.0-1.rockspec"),
      "utf8",
    );
    expect(rockspec).toContain('version = "0.4.0-1"');
    expect(rockspec).toContain("refs/tags/v0.4.0.tar.gz");
    expect(rockspec).toContain(
      'dir = "versionlens-redux-0.4.0/packages/neovim-plugin"',
    );
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});
