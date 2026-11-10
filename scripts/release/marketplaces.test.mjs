import { expect, it } from "bun:test";
import { chmod, mkdir, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";

import { capture } from "../lib/spawn.mjs";
import { expandUserPath, readSecretFile } from "./marketplace-secrets.mjs";

const repository = "xsyetopz/versionlens-redux";

/**
 * @param {string} directory
 * @returns {Promise<string>}
 */
const fakeGitHubCli = async (directory) => {
	const binaryDirectory = join(directory, "bin");
	const windows = process.platform === "win32";
	const binary = join(binaryDirectory, windows ? "gh.cmd" : "gh");
	await mkdir(binaryDirectory, { recursive: true });
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
	await Bun.write(binary, source);
	if (!windows) {
		await chmod(binary, 0o755);
	}
	return binaryDirectory;
};

/**
 * @param {string[]} [commandArguments]
 * @param {{ env?: Record<string, string | undefined>, stdin?: Uint8Array }} [options]
 */
const configureMarketplaces = async (commandArguments = [], options = {}) => {
	const child = Bun.spawn(
		[
			"bun",
			"scripts/release/configure-marketplaces.mjs",
			"--repo",
			repository,
			...commandArguments,
		],
		{ ...options, stderr: "pipe", stdout: "pipe" },
	);
	const [stdout, stderr, exitCode] = await Promise.all([
		new Response(child.stdout).text(),
		new Response(child.stderr).text(),
		child.exited,
	]);
	return { exitCode, stderr, stdout };
};

/**
 * @param {string} directory
 * @param {string} input
 * @param {string} [existing]
 */
const configureWithFakeGitHub = async (directory, input, existing = "") => {
	const log = join(directory, "gh.log");
	const binaryDirectory = await fakeGitHubCli(directory);
	const result = await configureMarketplaces(["--only", "jetbrains"], {
		env: {
			...Bun.env,
			GH_EXISTING: existing,
			GH_LOG: log,
			PATH: `${binaryDirectory}${delimiter}${Bun.env["PATH"] ?? ""}`,
		},
		stdin: new TextEncoder().encode(input),
	});
	return { log: await Bun.file(log).text(), result };
};

it("expands home-relative secret file paths before reading them", async () => {
	const home = await mkdtemp(join(tmpdir(), "versionlens-marketplace-home-"));
	try {
		const directory = join(home, ".config", "jetbrains-signing");
		await mkdir(directory, { recursive: true });
		await Bun.write(join(directory, "chain.crt"), "certificate-chain");
		const path = expandUserPath("~/.config/jetbrains-signing/chain.crt", home);
		const secret = await readSecretFile(
			"~/.config/jetbrains-signing/chain.crt",
			home,
		);
		expect(path).toBe(join(directory, "chain.crt"));
		expect(secret.path).toBe(path);
		expect(new TextDecoder().decode(secret.contents)).toBe("certificate-chain");
	} finally {
		await rm(home, { force: true, recursive: true });
	}
});

it("validates every selected value before writing any secret", async () => {
	const directory = await mkdtemp(
		join(tmpdir(), "versionlens-marketplace-preflight-"),
	);
	try {
		const { log, result } = await configureWithFakeGitHub(
			directory,
			"token-value\n/definitely/missing/chain.crt\n",
		);
		expect(result.exitCode).toBe(1);
		expect(log).toContain("secret list");
		expect(log).not.toContain("secret set");
	} finally {
		await rm(directory, { force: true, recursive: true });
	}
});

it("keeps existing secrets by default when resuming configuration", async () => {
	const directory = await mkdtemp(
		join(tmpdir(), "versionlens-marketplace-resume-"),
	);
	try {
		const chain = join(directory, "chain.crt");
		const key = join(directory, "private-key.pem");
		await Bun.write(chain, "certificate-chain");
		await Bun.write(key, "private-key");
		const { log, result } = await configureWithFakeGitHub(
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
		await rm(directory, { force: true, recursive: true });
	}
});

it("lists every required marketplace secret without making hosted changes", async () => {
	const result = await configureMarketplaces(["--dry-run"]);
	expect(result.exitCode).toBe(0);
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
		expect(result.stdout).toContain(`marketplaces: ${name}`);
	}
	expect(result.stdout).not.toContain("AZURE_SUBSCRIPTION_ID");
});

it("allows unavailable marketplaces to be skipped", async () => {
	const result = await configureMarketplaces(["--dry-run", "--only", "zed"]);
	expect(result.exitCode).toBe(0);
	expect(result.stdout).toContain("Selected marketplaces: zed");
	expect(result.stdout).toContain("marketplaces: ZED_EXTENSIONS_FORK");
	expect(result.stdout).toContain("marketplaces: ZED_EXTENSIONS_TOKEN");
	expect(result.stdout).not.toContain("AZURE_CLIENT_ID");
	expect(result.stdout).not.toContain("JETBRAINS_MARKETPLACE_TOKEN");
	expect(result.stdout).not.toContain("LUAROCKS_API_KEY");
});

it("rejects unknown marketplace selections", async () => {
	const result = await configureMarketplaces([
		"--dry-run",
		"--only",
		"unknown",
	]);
	expect(result.exitCode).toBe(1);
	expect(result.stderr).toContain("unknown marketplace unknown");
});

it("adds and updates the VersionLens Zed registry entry", async () => {
	const directory = await mkdtemp(join(tmpdir(), "versionlens-zed-registry-"));
	const registry = join(directory, "extensions.toml");
	try {
		await Bun.write(
			registry,
			'[alpha]\nsubmodule = "extensions/alpha"\nversion = "1.0.0"\n',
		);
		const first = await capture([
			"bun",
			"scripts/release/zed-registry.mjs",
			registry,
			"0.4.0",
		]);
		expect(first.exitCode).toBe(0);
		const second = await capture([
			"bun",
			"scripts/release/zed-registry.mjs",
			registry,
			"0.4.1",
		]);
		expect(second.exitCode).toBe(0);
		const contents = await Bun.file(registry).text();
		expect(contents.match(/\[versionlens-lsp\]/gu)).toHaveLength(1);
		expect(contents).toContain('path = "packages/zed-extension"');
		expect(contents).toContain('version = "0.4.1"');
	} finally {
		await rm(directory, { force: true, recursive: true });
	}
});

it("renders a versioned LuaRocks specification", async () => {
	const directory = await mkdtemp(join(tmpdir(), "versionlens-rockspec-"));
	try {
		const result = await capture([
			"bun",
			"scripts/release/luarocks.mjs",
			"0.4.0",
			directory,
		]);
		expect(result.exitCode).toBe(0);
		const rockspec = await Bun.file(
			join(directory, "versionlens-redux-0.4.0-1.rockspec"),
		).text();
		expect(rockspec).toContain('version = "0.4.0-1"');
		expect(rockspec).toContain("refs/tags/v0.4.0.tar.gz");
		expect(rockspec).toContain(
			'dir = "versionlens-redux-0.4.0/packages/neovim-plugin"',
		);
	} finally {
		await rm(directory, { force: true, recursive: true });
	}
});
