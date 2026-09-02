import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";

import { expandUserPath, readSecretFile } from "./configure-marketplaces.mjs";

const { expect, it } = Bun.jest(import.meta.path);

function fakeGitHubCli(directory) {
  const binaryDirectory = join(directory, "bin");
  const windows = process.platform === "win32";
  const binary = join(binaryDirectory, windows ? "gh.cmd" : "gh");
  mkdirSync(binaryDirectory, { recursive: true });
  const source = windows
    ? `@echo off
echo %*>>"%GH_LOG%"
if "%1"=="--version" echo gh version test
if "%1"=="secret" if "%2"=="list" echo %GH_EXISTING%
if "%1"=="secret" if "%2"=="set" more > nul
exit /b 0
`
    : `#!/bin/sh
printf '%s\\n' "$*" >> "$GH_LOG"
if [ "$1" = "--version" ]; then
  echo "gh version test"
elif [ "$1" = "secret" ] && [ "$2" = "list" ]; then
  printf '%s\\n' "$GH_EXISTING"
elif [ "$1" = "secret" ] && [ "$2" = "set" ]; then
  cat >/dev/null
fi
`;
  writeFileSync(binary, source);
  if (!windows) {
    chmodSync(binary, 0o755);
  }
  return binaryDirectory;
}

function configureWithFakeGitHub(directory, input, existing = "") {
  const log = join(directory, "gh.log");
  const binaryDirectory = fakeGitHubCli(directory);
  const result = Bun.spawnSync(
    [
      "bun",
      "scripts/release/configure-marketplaces.mjs",
      "--repo",
      "xsyetopz/versionlens-redux",
      "--only",
      "jetbrains",
    ],
    {
      env: {
        ...Bun.env,
        GH_EXISTING: existing,
        GH_LOG: log,
        PATH: `${binaryDirectory}${delimiter}${Bun.env.PATH ?? ""}`,
      },
      stdin: new TextEncoder().encode(input),
    },
  );
  return { log: readFileSync(log, "utf8"), result };
}

it("expands home-relative secret file paths before reading them", () => {
  const home = mkdtempSync(join(tmpdir(), "versionlens-marketplace-home-"));
  try {
    const directory = join(home, ".config", "jetbrains-signing");
    mkdirSync(directory, { recursive: true });
    writeFileSync(join(directory, "chain.crt"), "certificate-chain");
    const path = expandUserPath("~/.config/jetbrains-signing/chain.crt", home);
    const secret = readSecretFile(
      "~/.config/jetbrains-signing/chain.crt",
      home,
    );
    expect(path).toBe(join(directory, "chain.crt"));
    expect(secret.path).toBe(path);
    expect(secret.contents.toString()).toBe("certificate-chain");
  } finally {
    rmSync(home, { force: true, recursive: true });
  }
});

it("validates every selected value before writing any secret", () => {
  const directory = mkdtempSync(
    join(tmpdir(), "versionlens-marketplace-preflight-"),
  );
  try {
    const { log, result } = configureWithFakeGitHub(
      directory,
      "token-value\n/definitely/missing/chain.crt\n",
    );
    expect(result.exitCode).toBe(1);
    expect(log).toContain("secret list");
    expect(log).not.toContain("secret set");
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});

it("keeps existing secrets by default when resuming configuration", () => {
  const directory = mkdtempSync(
    join(tmpdir(), "versionlens-marketplace-resume-"),
  );
  try {
    const chain = join(directory, "chain.crt");
    const key = join(directory, "private-key.pem");
    writeFileSync(chain, "certificate-chain");
    writeFileSync(key, "private-key");
    const { log, result } = configureWithFakeGitHub(
      directory,
      `\n${chain}\n${key}\npassword\n`,
      "JETBRAINS_MARKETPLACE_TOKEN",
    );
    expect(result.exitCode).toBe(0);
    expect(log).not.toContain("secret set JETBRAINS_MARKETPLACE_TOKEN");
    expect(log).toContain("secret set JB_CERTIFICATE_CHAIN");
    expect(log).toContain("secret set JB_PRIVATE_KEY");
    expect(log).toContain("secret set JB_PRIVATE_KEY_PASSWORD");
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
});

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
  expect(output).not.toContain("AZURE_SUBSCRIPTION_ID");
});

it("allows unavailable marketplaces to be skipped", () => {
  const result = Bun.spawnSync([
    "bun",
    "scripts/release/configure-marketplaces.mjs",
    "--dry-run",
    "--repo",
    "xsyetopz/versionlens-redux",
    "--only",
    "zed",
  ]);
  expect(result.exitCode).toBe(0);
  const output = result.stdout.toString();
  expect(output).toContain("Selected marketplaces: zed");
  expect(output).toContain("marketplaces: ZED_EXTENSIONS_FORK");
  expect(output).toContain("marketplaces: ZED_EXTENSIONS_TOKEN");
  expect(output).not.toContain("AZURE_CLIENT_ID");
  expect(output).not.toContain("JETBRAINS_MARKETPLACE_TOKEN");
  expect(output).not.toContain("LUAROCKS_API_KEY");
});

it("rejects unknown marketplace selections", () => {
  const result = Bun.spawnSync([
    "bun",
    "scripts/release/configure-marketplaces.mjs",
    "--dry-run",
    "--repo",
    "xsyetopz/versionlens-redux",
    "--only",
    "unknown",
  ]);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toContain("unknown marketplace unknown");
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

it("provisions and validates LuaRocks upload requirements", () => {
  const workflow = readFileSync(
    ".github/workflows/publish-marketplaces.yml",
    "utf8",
  );
  const neovimJob = workflow.slice(workflow.indexOf("\n  neovim:"));
  const install = neovimJob.indexOf(
    "sudo apt-get install --no-install-recommends --yes lua5.4 lua-dkjson luarocks",
  );
  const guard = neovimJob.indexOf('if [[ -z "$LUAROCKS_API_KEY" ]]');
  const failClosed = neovimJob.indexOf("exit 1", guard);
  const upload = neovimJob.indexOf("luarocks upload", guard);

  expect(neovimJob).not.toBe(workflow);
  expect(install).toBeGreaterThan(-1);
  expect(guard).toBeGreaterThan(install);
  expect(failClosed).toBeGreaterThan(guard);
  expect(upload).toBeGreaterThan(failClosed);
});

it("uses tenant-only Azure authentication for VS Code publishing", () => {
  const workflow = readFileSync(
    ".github/workflows/publish-marketplaces.yml",
    "utf8",
  );
  const vscodeStart = workflow.indexOf("\n  vscode:");
  const jetbrainsStart = workflow.indexOf("\n  jetbrains:", vscodeStart);
  const vscodeJob = workflow.slice(vscodeStart, jetbrainsStart);

  expect(vscodeStart).toBeGreaterThan(-1);
  expect(jetbrainsStart).toBeGreaterThan(vscodeStart);
  expect(vscodeJob).toContain("allow-no-subscriptions: true");
  expect(vscodeJob).not.toContain("subscription-id:");
});

it("resolves the Marketplace identity before publishing VSIX packages", () => {
  const workflow = readFileSync(
    ".github/workflows/publish-marketplaces.yml",
    "utf8",
  );
  const vscodeStart = workflow.indexOf("\n  vscode:");
  const jetbrainsStart = workflow.indexOf("\n  jetbrains:", vscodeStart);
  const vscodeJob = workflow.slice(vscodeStart, jetbrainsStart);
  const login = vscodeJob.indexOf("name: Sign in to Microsoft Entra");
  const resolveIdentity = vscodeJob.indexOf(
    "name: Resolve Visual Studio Marketplace identity",
  );
  const profileRequest = vscodeJob.indexOf(
    "https://app.vssps.visualstudio.com/_apis/profile/profiles/me",
    resolveIdentity,
  );
  const marketplaceResource = vscodeJob.indexOf(
    "499b84ac-1321-427f-aa17-267ca6975798",
    resolveIdentity,
  );
  const failClosed = vscodeJob.indexOf("exit 1", resolveIdentity);
  const publish = vscodeJob.indexOf("name: Publish released VSIX packages");

  expect(login).toBeGreaterThan(-1);
  expect(resolveIdentity).toBeGreaterThan(login);
  expect(profileRequest).toBeGreaterThan(resolveIdentity);
  expect(marketplaceResource).toBeGreaterThan(profileRequest);
  expect(failClosed).toBeGreaterThan(marketplaceResource);
  expect(publish).toBeGreaterThan(failClosed);
});

it("makes VS Code Marketplace publishing resumable and retries transient failures", () => {
  const workflow = readFileSync(
    ".github/workflows/publish-marketplaces.yml",
    "utf8",
  );
  const vscodeStart = workflow.indexOf("\n  vscode:");
  const jetbrainsStart = workflow.indexOf("\n  jetbrains:", vscodeStart);
  const vscodeJob = workflow.slice(vscodeStart, jetbrainsStart);
  const publish = vscodeJob.indexOf("name: Publish released VSIX packages");
  const skipDuplicate = vscodeJob.indexOf("--skip-duplicate", publish);
  const retryLoop = vscodeJob.indexOf("for attempt in 1 2 3", publish);
  const retryDelay = vscodeJob.indexOf('sleep "$delay"', retryLoop);
  const failClosed = vscodeJob.indexOf("exit 1", retryLoop);

  expect(publish).toBeGreaterThan(-1);
  expect(retryLoop).toBeGreaterThan(publish);
  expect(skipDuplicate).toBeGreaterThan(retryLoop);
  expect(failClosed).toBeGreaterThan(skipDuplicate);
  expect(retryDelay).toBeGreaterThan(failClosed);
});
