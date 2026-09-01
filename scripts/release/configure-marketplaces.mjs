#!/usr/bin/env bun
import { readFileSync } from "node:fs";
import { homedir } from "node:os";
import { resolve } from "node:path";
import process from "node:process";
import { createInterface } from "node:readline/promises";
import { Writable } from "node:stream";

const environment = "marketplaces";
const dryRun = process.argv.includes("--dry-run");
const repoArgumentIndex = process.argv.indexOf("--repo");
const onlyArgumentIndex = process.argv.indexOf("--only");
const marketplaceIds = ["vscode", "jetbrains", "zed", "neovim"];
const marketplaceLabels = new Map([
  ["vscode", "VS Code Marketplace"],
  ["jetbrains", "JetBrains Marketplace"],
  ["zed", "Zed extension registry"],
  ["neovim", "Neovim via LuaRocks"],
]);

function run(command, options = {}) {
  const result = Bun.spawnSync(command, {
    env: { ...Bun.env, GH_PAGER: "cat" },
    stderr: "inherit",
    stdout: options.capture ? "pipe" : options.quiet ? "ignore" : "inherit",
    stdin: options.stdin ?? "inherit",
  });
  if (result.exitCode !== 0) {
    process.exit(result.exitCode);
  }
  return result.stdout?.toString().trim();
}

function requestedMarketplaces() {
  if (onlyArgumentIndex < 0) {
    return null;
  }
  const value = process.argv[onlyArgumentIndex + 1];
  if (!value) {
    throw new Error(
      `--only requires one or more of: ${marketplaceIds.join(", ")}`,
    );
  }
  const selected = new Set(
    value
      .split(",")
      .map((entry) => entry.trim().toLowerCase())
      .filter(Boolean),
  );
  const unknown = [...selected].filter(
    (entry) => !marketplaceLabels.has(entry),
  );
  if (unknown.length > 0) {
    throw new Error(
      `unknown marketplace ${unknown.join(", ")}; expected: ${marketplaceIds.join(", ")}`,
    );
  }
  return selected;
}

async function chooseMarketplaces(ask) {
  const requested = requestedMarketplaces();
  if (requested !== null) {
    return requested;
  }
  if (dryRun) {
    return new Set(marketplaceIds);
  }

  console.log(
    "\nSelect the destinations whose provider accounts are ready. Skipped destinations can be configured by running this command again later.",
  );
  const selected = new Set();
  for (const id of marketplaceIds) {
    const label = marketplaceLabels.get(id);
    const answer = (await ask(`Configure ${label}? [y/N]: `))
      .trim()
      .toLowerCase();
    if (answer === "y" || answer === "yes") {
      selected.add(id);
    } else if (answer !== "" && answer !== "n" && answer !== "no") {
      throw new Error(`expected yes or no for ${label}`);
    }
  }
  return selected;
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

function expandUserPath(value, home = homedir()) {
  if (value === "~") {
    return home;
  }
  if (value.startsWith("~/") || value.startsWith("~\\")) {
    return resolve(home, value.slice(2));
  }
  if (value.startsWith("~")) {
    throw new Error(
      "paths for another user's home directory are not supported",
    );
  }
  return resolve(value);
}

function readSecretFile(value, home = homedir()) {
  const path = expandUserPath(value, home);
  const contents = readFileSync(path);
  if (contents.length === 0) {
    throw new Error(`${path} is empty`);
  }
  return { contents, path };
}

function existingSecretNames(repo) {
  if (dryRun) {
    return new Set();
  }
  const output = run(
    [
      "gh",
      "secret",
      "list",
      "--repo",
      repo,
      "--env",
      environment,
      "--json",
      "name",
      "--jq",
      ".[].name",
    ],
    { capture: true },
  );
  return new Set(output ? output.split("\n") : []);
}

function setSecret(repo, name, value) {
  if (dryRun) {
    console.log(`${environment}: ${name}`);
    return;
  }
  const bytes =
    typeof value === "string" ? new TextEncoder().encode(value) : value;
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

  let muteInputEcho = false;
  const promptOutput = new Writable({
    write(chunk, encoding, callback) {
      if (!muteInputEcho) {
        process.stdout.write(chunk, encoding);
      }
      callback();
    },
  });
  const input = createInterface({
    input: process.stdin,
    output: promptOutput,
    terminal: process.stdin.isTTY,
  });
  const lines = input[Symbol.asyncIterator]();
  async function ask(prompt, options = {}) {
    process.stdout.write(prompt);
    muteInputEcho = options.hidden === true;
    try {
      const { done, value } = await lines.next();
      if (done) {
        throw new Error("input closed before configuration was complete");
      }
      return value;
    } finally {
      if (options.hidden === true) {
        process.stdout.write("\n");
      }
      muteInputEcho = false;
    }
  }
  const selected = await chooseMarketplaces(ask);
  if (selected.size === 0) {
    input.close();
    console.log(
      "No marketplaces selected; no GitHub environment changes were made.",
    );
    return;
  }
  console.log(`Selected marketplaces: ${[...selected].join(", ")}`);

  if (dryRun) {
    console.log(`Repository: ${repo}`);
  } else {
    run(
      [
        "gh",
        "api",
        "--method",
        "PUT",
        `repos/${repo}/environments/${environment}`,
      ],
      { quiet: true },
    );
  }
  const existingSecrets = existingSecretNames(repo);
  const pendingSecrets = [];

  async function replaceExistingSecret(name) {
    if (!existingSecrets.has(name)) {
      return true;
    }
    const answer = (
      await ask(`${name} is already configured. Replace it? [y/N]: `)
    )
      .trim()
      .toLowerCase();
    if (answer === "y" || answer === "yes") {
      return true;
    }
    if (answer === "" || answer === "n" || answer === "no") {
      console.log(`Keeping existing ${name}.`);
      return false;
    }
    throw new Error(`expected yes or no for ${name}`);
  }

  function queueSecret(name, value) {
    pendingSecrets.push({ name, value });
  }

  async function visibleSecret(name, prompt) {
    if (dryRun) {
      queueSecret(name, "dry-run");
      return;
    }
    if (!(await replaceExistingSecret(name))) {
      return;
    }
    const value = (await ask(`${prompt}: `, { hidden: true })).trim();
    if (!value) {
      throw new Error(`${name} cannot be empty`);
    }
    queueSecret(name, value);
  }

  async function fileSecret(name, prompt) {
    if (dryRun) {
      queueSecret(name, "dry-run");
      return;
    }
    if (!(await replaceExistingSecret(name))) {
      return;
    }
    while (true) {
      const value = (await ask(`${prompt} file path: `)).trim();
      if (!value) {
        throw new Error(`${name} file path cannot be empty`);
      }
      try {
        const { contents, path } = readSecretFile(value);
        console.log(`Validated ${name}: ${path}`);
        queueSecret(name, contents);
        return;
      } catch (error) {
        console.error(
          `Could not read ${name}: ${error instanceof Error ? error.message : error}`,
        );
        console.error(
          "Enter another path, or press Ctrl-C to leave all secrets unchanged.",
        );
      }
    }
  }

  async function hiddenSecret(name, prompt) {
    if (dryRun) {
      queueSecret(name, "dry-run");
      return;
    }
    if (!(await replaceExistingSecret(name))) {
      return;
    }
    const value = await ask(`${prompt}: `, { hidden: true });
    if (value.length === 0) {
      throw new Error(`${name} cannot be empty`);
    }
    queueSecret(name, value);
  }

  try {
    if (selected.has("vscode")) {
      console.log("\nVS Code Marketplace (Microsoft Entra workload identity)");
      await visibleSecret(
        "AZURE_CLIENT_ID",
        "Azure managed identity or app client ID",
      );
      await visibleSecret("AZURE_TENANT_ID", "Azure tenant ID");
      await visibleSecret("AZURE_SUBSCRIPTION_ID", "Azure subscription ID");
    }

    if (selected.has("jetbrains")) {
      console.log("\nJetBrains Marketplace");
      await hiddenSecret(
        "JETBRAINS_MARKETPLACE_TOKEN",
        "JetBrains permanent Marketplace token",
      );
      await fileSecret(
        "JB_CERTIFICATE_CHAIN",
        "JetBrains signing certificate chain PEM",
      );
      await fileSecret("JB_PRIVATE_KEY", "JetBrains signing private key PEM");
      await hiddenSecret(
        "JB_PRIVATE_KEY_PASSWORD",
        "JetBrains signing private key password",
      );
    }

    if (selected.has("zed")) {
      console.log("\nZed extension registry");
      await visibleSecret(
        "ZED_EXTENSIONS_FORK",
        "Fork of zed-industries/extensions (owner/extensions)",
      );
      await hiddenSecret(
        "ZED_EXTENSIONS_TOKEN",
        "GitHub token that can push to the fork and open a pull request against zed-industries/extensions",
      );
    }

    if (selected.has("neovim")) {
      console.log("\nNeovim via LuaRocks");
      await hiddenSecret("LUAROCKS_API_KEY", "LuaRocks API key");
    }
  } finally {
    muteInputEcho = false;
    input.close();
  }

  if (pendingSecrets.length === 0) {
    console.log("\nNo secret updates requested; existing values were kept.");
    return;
  }
  console.log(
    `\nValidated ${pendingSecrets.length} secret value${pendingSecrets.length === 1 ? "" : "s"}. Applying updates to GitHub...`,
  );
  for (const { name, value } of pendingSecrets) {
    setSecret(repo, name, value);
  }
  console.log(
    `\nConfigured GitHub environment secrets in ${repo}/${environment}.`,
  );
}

if (import.meta.main) {
  configure().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exit(1);
  });
}

export { expandUserPath, readSecretFile };
