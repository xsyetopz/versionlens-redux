#!/usr/bin/env bun

import fs from "node:fs";

const offenders = [];
const cargoToml = fs.readFileSync("Cargo.toml", "utf8");
const buildExtensionScript = fs.readFileSync(
	"scripts/build-extension.mjs",
	"utf8",
);
const rustToolchain = fs.readFileSync("rust-toolchain.toml", "utf8");
const packageJson = JSON.parse(fs.readFileSync("package.json", "utf8"));
const tsconfigBase = JSON.parse(fs.readFileSync("tsconfig.base.json", "utf8"));

function requireMatch(label, source, pattern) {
	if (!pattern.test(source)) {
		offenders.push(label);
	}
}

requireMatch(
	"Cargo.toml workspace edition must be 2024",
	cargoToml,
	/edition\s*=\s*"2024"/u,
);
requireMatch(
	"Cargo.toml workspace rust-version must be 1.96",
	cargoToml,
	/rust-version\s*=\s*"1\.96"/u,
);
requireMatch(
	"rust-toolchain.toml channel must be 1.96.0",
	rustToolchain,
	/channel\s*=\s*"1\.96\.0"/u,
);
requireMatch(
	"Cargo.toml ureq must disable default gzip to keep packaged native size down",
	cargoToml,
	/ureq\s*=\s*\{[^}]*default-features\s*=\s*false[^}]*features\s*=\s*\[\s*"rustls"\s*\][^}]*\}/u,
);
for (const [label, pattern] of [
	[
		"Cargo.toml release opt-level must optimize packaged native size",
		/opt-level\s*=\s*"z"/u,
	],
	[
		"Cargo.toml release LTO must be enabled for packaged native size",
		/lto\s*=\s*"fat"/u,
	],
	["Cargo.toml release debug info must be disabled", /debug\s*=\s*0/u],
	["Cargo.toml release symbols must be stripped", /strip\s*=\s*"symbols"/u],
]) {
	requireMatch(label, cargoToml, pattern);
}

if (packageJson.devDependencies?.typescript !== "^6.0.3") {
	offenders.push("package.json must use TypeScript ^6.0.3");
}

if (tsconfigBase.compilerOptions?.erasableSyntaxOnly !== true) {
	offenders.push("tsconfig.base.json must enforce erasableSyntaxOnly");
}

if (packageJson.scripts?.build !== "bun scripts/build-extension.mjs") {
	offenders.push(
		"package.json build script must use scripts/build-extension.mjs",
	);
}

if (
	packageJson.scripts?.["check:extension-build"] !==
	"bun scripts/build-extension.mjs --check"
) {
	offenders.push(
		"package.json check:extension-build must validate the VS Code extension bundle",
	);
}

if (
	!packageJson.scripts?.check
		?.split(" && ")
		.includes("bun run check:extension-build")
) {
	offenders.push("package.json check must include check:extension-build");
}

for (const [label, snippet] of [
	["clean dist before bundling", "rmSync(extensionDist"],
	["emit CommonJS for VS Code", 'format: "cjs"'],
	["minify the packaged extension bundle", "minify: true"],
	["target Node for VS Code", 'target: "node"'],
	["externalize VS Code API", 'external: ["vscode"]'],
]) {
	if (!buildExtensionScript.includes(snippet)) {
		offenders.push(`scripts/build-extension.mjs must ${label}`);
	}
}

if (
	!(
		packageJson.scripts?.["native:build:release"]?.includes("--release") &&
		packageJson.scripts?.["native:build:release"]?.includes(
			"scripts/build-native.mjs --release",
		)
	)
) {
	offenders.push("package.json must define a release native build script");
}

if (
	!packageJson.scripts?.["vscode:prepublish"]?.includes(
		"bun run native:build:release",
	)
) {
	offenders.push("vscode:prepublish must package a release native module");
}

for (const [name, command] of Object.entries(packageJson.scripts ?? {})) {
	if (/\b(?:npm|npx|pnpm|yarn)\b/u.test(command)) {
		offenders.push(
			`package.json script ${name} must use bun/bunx, not npm/npx/pnpm/yarn`,
		);
	}
}

if (fs.existsSync("package-lock.json")) {
	offenders.push("package-lock.json must not exist; use bun.lock");
}

if (!fs.existsSync("bun.lock")) {
	offenders.push("bun.lock must exist");
}

if (offenders.length > 0) {
	console.error(offenders.join("\n"));
	process.exit(1);
}

console.log("checked toolchain; Rust 2024/1.96 and TypeScript ^6.0.3 via Bun");
