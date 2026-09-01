#!/usr/bin/env bun
import { readFileSync } from "node:fs";
import process from "node:process";
import { createInterface } from "node:readline/promises";

const environment = "marketplaces";
const dryRun = process.argv.includes("--dry-run");
const repoArgumentIndex = process.argv.indexOf("--repo");

function run(command, options = {}) {
  const result = Bun.spawnSync(command, {
    stderr: "inherit",
    stdout: options.capture ? "pipe" : "inherit",
    stdin: options.stdin ?? "inherit",
  });
  if (result.exitCode !== 0) {
    process.exit(result.exitCode);
  }
  return result.stdout?.toString().trim();
}

function requireCommand(name) {
  const result = Bun.spawnSync([name, "--version"], {
    stderr: "ignore",
    stdout: "ignore",
  });
  if (result.exitCode !== 0) {
    throw new Error(`${name} is required`);
  }
}

function setSecret(repo, name, value) {
  if (dryRun) {
    console.log(`${environment}: ${name}`);
    return;
  }
  const bytes =
    value === undefined
      ? undefined
      : typeof value === "string"
        ? new TextEncoder().encode(value)
        : value;
  run(["gh", "secret", "set", name, "--repo", repo, "--env", environment], {
    stdin: bytes,
  });
}

async function configure() {
  requireCommand("gh");
  run(["gh", "auth", "status"]);
  const requestedRepo =
    repoArgumentIndex >= 0 ? process.argv[repoArgumentIndex + 1] : undefined;
  const repo =
    requestedRepo ??
    run(
      [
        "gh",
        "repo",
        "view",
        "--json",
        "nameWithOwner",
        "--jq",
        ".nameWithOwner",
      ],
      { capture: true },
    );
  if (!repo?.match(/^[^/]+\/[^/]+$/u)) {
    throw new Error(
      `could not determine a GitHub owner/repository, got ${repo}`,
    );
  }

  if (dryRun) {
    console.log(`Repository: ${repo}`);
  } else {
    run([
      "gh",
      "api",
      "--method",
      "PUT",
      `repos/${repo}/environments/${environment}`,
    ]);
  }

  const input = createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  async function visibleSecret(name, prompt) {
    if (dryRun) {
      setSecret(repo, name, "dry-run");
      return;
    }
    const value = (await input.question(`${prompt}: `)).trim();
    if (!value) {
      throw new Error(`${name} cannot be empty`);
    }
    setSecret(repo, name, value);
  }
  async function fileSecret(name, prompt) {
    if (dryRun) {
      setSecret(repo, name, "dry-run");
      return;
    }
    const path = (await input.question(`${prompt} file path: `)).trim();
    if (!path) {
      throw new Error(`${name} file path cannot be empty`);
    }
    setSecret(repo, name, readFileSync(path));
  }
  function hiddenSecret(name, prompt) {
    if (!dryRun) {
      console.log(
        `${prompt}. GitHub CLI will read the value without storing it locally.`,
      );
    }
    input.pause();
    setSecret(repo, name);
    input.resume();
  }

  console.log("\nVS Code Marketplace (Microsoft Entra workload identity)");
  await visibleSecret(
    "AZURE_CLIENT_ID",
    "Azure managed identity or app client ID",
  );
  await visibleSecret("AZURE_TENANT_ID", "Azure tenant ID");
  await visibleSecret("AZURE_SUBSCRIPTION_ID", "Azure subscription ID");

  console.log("\nJetBrains Marketplace");
  hiddenSecret(
    "JETBRAINS_MARKETPLACE_TOKEN",
    "JetBrains permanent Marketplace token",
  );
  await fileSecret(
    "JB_CERTIFICATE_CHAIN",
    "JetBrains signing certificate chain PEM",
  );
  await fileSecret("JB_PRIVATE_KEY", "JetBrains signing private key PEM");
  hiddenSecret(
    "JB_PRIVATE_KEY_PASSWORD",
    "JetBrains signing private key password",
  );

  console.log("\nZed extension registry");
  await visibleSecret(
    "ZED_EXTENSIONS_FORK",
    "Fork of zed-industries/extensions (owner/extensions)",
  );
  hiddenSecret(
    "ZED_EXTENSIONS_TOKEN",
    "GitHub token that can push to the fork and open a pull request against zed-industries/extensions",
  );

  console.log("\nNeovim via LuaRocks");
  hiddenSecret("LUAROCKS_API_KEY", "LuaRocks API key");
  input.close();

  if (!dryRun) {
    console.log(
      `\nConfigured GitHub environment secrets in ${repo}/${environment}.`,
    );
  }
}

configure().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
