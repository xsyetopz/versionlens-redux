#!/usr/bin/env bun
import { copyFile, mkdir, mkdtemp, readdir, rm } from "node:fs/promises";
import { homedir, tmpdir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import process from "node:process";

import { capture, inherit } from "../lib/spawn.mjs";
import { readTargets } from "./targets.mjs";

const REQUIRED_JAVA_MAJOR = 21;
const HOST_PATTERN = /^host:\s*(?<triple>\S+)/mu;
const JAVA_VERSION_PATTERN = /version\s+"(?<major>\d+)/u;

/**
 * @param {string[]} command
 * @param {{ env?: Record<string, string | undefined> }} [options]
 * @returns {Promise<void>}
 */
const run = async (command, options = {}) => {
	console.log(`$ ${command.join(" ")}`);
	const exitCode = await inherit(command, options);
	if (exitCode !== 0) {
		throw new Error(
			`Command failed with exit code ${exitCode}: ${command.join(" ")}`,
		);
	}
};

/**
 * @param {string[]} command
 * @returns {Promise<string>}
 */
const commandOutput = async (command) => {
	const result = await capture(command);
	if (result.exitCode !== 0) {
		return "";
	}
	return result.stdout.trim();
};

/**
 * @param {string} path
 * @returns {Promise<boolean>}
 */
const fileExists = (path) => Bun.file(path).exists();

/**
 * @param {string} candidateHome
 * @returns {Promise<number | null>}
 */
const javaMajor = async (candidateHome) => {
	const result = await capture([
		join(candidateHome, "bin", "java"),
		"-version",
	]);
	if (result.exitCode !== 0) {
		return null;
	}
	const match = JAVA_VERSION_PATTERN.exec(`${result.stderr}${result.stdout}`);
	if (match?.groups === undefined) {
		return null;
	}
	return Number(match.groups["major"]);
};

/** @returns {Promise<string[]>} */
const sdkmanJavaDirectories = async () => {
	try {
		return await readdir(join(homedir(), ".sdkman/candidates/java"));
	} catch {
		return [];
	}
};

/** @returns {Promise<[string, string][]>} */
const candidateJavaHomes = async () => {
	/** @type {[string, string][]} */
	const candidates = [];
	const configuredJavaHome = Bun.env["JAVA_HOME"];
	if (configuredJavaHome !== undefined && configuredJavaHome !== "") {
		candidates.push([configuredJavaHome, "JAVA_HOME"]);
	}
	if (
		process.platform === "darwin" &&
		(await fileExists("/usr/libexec/java_home"))
	) {
		const javaHome = await commandOutput([
			"/usr/libexec/java_home",
			"-v",
			"21",
		]);
		if (javaHome !== "") {
			candidates.push([javaHome, "macOS java_home"]);
		}
	}
	for (const prefix of ["/opt/homebrew", "/usr/local"]) {
		candidates.push([
			join(prefix, "opt/openjdk@21/libexec/openjdk.jdk/Contents/Home"),
			`${prefix}/opt/openjdk@21`,
		]);
	}
	const sdkmanJavaHome = join(homedir(), ".sdkman/candidates/java");
	for (const name of await sdkmanJavaDirectories()) {
		if (name.startsWith("21")) {
			candidates.push([
				join(sdkmanJavaHome, name, "current"),
				"SDKMAN Java 21",
			]);
		}
	}
	const pathJava = await commandOutput(["sh", "-c", "command -v java"]);
	if (pathJava !== "") {
		candidates.push([dirname(dirname(pathJava)), "PATH java"]);
	}
	return candidates;
};

/**
 * @param {[string, string]} candidate
 * @returns {Promise<{ home: string, label: string, major: number | null } | null>}
 */
const probeCandidate = async ([home, label]) => {
	if (!(await fileExists(join(home, "bin", "java")))) {
		return null;
	}
	return { home, label, major: await javaMajor(home) };
};

/** @returns {Promise<string>} */
const resolveJava21 = async () => {
	const candidates = await candidateJavaHomes();
	const probed = await Promise.all(
		candidates.map((candidate) => probeCandidate(candidate)),
	);
	/** @type {string[]} */
	const rejected = [];
	for (const probe of probed) {
		if (probe !== null) {
			if (probe.major === REQUIRED_JAVA_MAJOR) {
				return probe.home;
			}
			let version = "unreadable";
			if (probe.major !== null) {
				version = `Java ${probe.major}`;
			}
			rejected.push(`${probe.label}: ${version}`);
		}
	}
	let diagnostic = "No Java runtime candidates were found.";
	if (rejected.length > 0) {
		diagnostic = `Rejected candidates: ${rejected.join(", ")}`;
	}
	throw new Error(
		[
			"Editor packaging requires an installed Java 21 runtime.",
			"No unpinned JDK download is attempted.",
			"Set JAVA_HOME to a Java 21 installation or install the repository-managed toolchain used by CI (actions/setup-java Java 21).",
			diagnostic,
		].join(" "),
	);
};

/**
 * @param {string} destinationRoot
 * @returns {Promise<void>}
 */
const stageHostBinary = async (destinationRoot) => {
	const targets = await readTargets();
	const hostOutput = await commandOutput(["rustc", "-vV"]);
	const host = HOST_PATTERN.exec(hostOutput)?.groups?.["triple"];
	if (
		host === undefined ||
		host === "" ||
		!targets.some(({ rust }) => rust === host)
	) {
		throw new Error(
			`Host Rust target ${host ?? "(unknown)"} is not listed in scripts/package/targets.json`,
		);
	}
	const executable = host.includes("windows")
		? "versionlens-lsp.exe"
		: "versionlens-lsp";
	const targetDir = join(destinationRoot, host);
	await mkdir(targetDir, { recursive: true });
	await copyFile(
		join(process.cwd(), "target", "release", executable),
		join(targetDir, executable),
	);
};

const javaRuntime = await resolveJava21();
console.log(`Using Java ${REQUIRED_JAVA_MAJOR} runtime: ${javaRuntime}`);
const repositoryRoot = process.cwd();
const fixturesRoot = join(repositoryRoot, "tests", "fixtures");
const projectGradleState = join(
	repositoryRoot,
	"packages",
	"jetbrains-plugin",
	".gradle",
);
const projectBuildTemp = join(
	repositoryRoot,
	"packages",
	"jetbrains-plugin",
	"build",
	"tmp",
);
const gradleHome = await mkdtemp(join(tmpdir(), "versionlens-gradle-"));
if (gradleHome.startsWith(fixturesRoot)) {
	throw new Error(`Refusing Gradle user home inside fixtures: ${gradleHome}`);
}
const binariesDir = await mkdtemp(join(tmpdir(), "versionlens-lsp-binaries-"));
const gradleEnvironment = {
	...Bun.env,
	GRADLE_USER_HOME: gradleHome,
	JAVA_HOME: javaRuntime,
	PATH: `${join(javaRuntime, "bin")}${delimiter}${Bun.env["PATH"] ?? ""}`,
};

try {
	await run(["bun", "run", "package"], { env: gradleEnvironment });
	await run(
		["cargo", "build", "-p", "versionlens-lsp", "--release", "--locked"],
		{ env: gradleEnvironment },
	);
	await stageHostBinary(binariesDir);
	await run(
		[
			"cargo",
			"build",
			"--manifest-path",
			"packages/zed-extension/Cargo.toml",
			"--release",
			"--locked",
		],
		{ env: gradleEnvironment },
	);
	await run(["bun", "scripts/package/neovim.mjs"], { env: gradleEnvironment });
	await run(
		[
			"./packages/jetbrains-plugin/gradlew",
			"-p",
			"packages/jetbrains-plugin",
			"buildPlugin",
			"--no-daemon",
			`-PversionlensBinariesDir=${binariesDir}`,
			"-PversionlensAllowPartialTargets",
		],
		{ env: gradleEnvironment },
	);
} finally {
	await Promise.all(
		[gradleHome, binariesDir, projectGradleState, projectBuildTemp].map(
			(path) => rm(path, { force: true, recursive: true }),
		),
	);
}
