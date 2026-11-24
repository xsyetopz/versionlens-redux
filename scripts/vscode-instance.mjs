// Runs one isolated VS Code instance for a scripted check. `just vscode-live` and
// `just vscode-bench` call this inside the Linux container from scripts/vscode-container, under
// Xvfb, because VS Code on macOS brings its window in front whatever its launcher asks.
import { join } from "node:path";
import process from "node:process";

const linux = process.platform === "linux";

/** The VS Code executable; VSCODE_BIN overrides it. */
const binary =
	process.env["VSCODE_BIN"] ??
	(linux
		? "/usr/share/code/code"
		: "/Applications/Visual Studio Code.app/Contents/MacOS/Code");

/** The command-line launcher beside the executable, for extension installs. */
export const cli = linux
	? join(binary, "..", "bin", "code")
	: join(binary, "..", "..", "Resources", "app", "bin", "code");

// The container runs as root, where Chromium needs its sandbox off, and has no GPU.
const flags = linux ? ["--no-sandbox", "--disable-gpu"] : [];

/**
 * Runs VS Code with `args` until it exits.
 *
 * @param {string[]} args VS Code arguments.
 * @param {Record<string, string>} env Variables added to the instance's environment.
 * @returns {Promise<{ exitCode: number, stdout: string }>} The exit code and standard output.
 */
export const instance = async (args, env = {}) => {
	const child = Bun.spawn([binary, ...flags, ...args], {
		env: { ...process.env, ...env },
		stderr: "ignore",
		stdout: "pipe"
	});
	const [stdout, exitCode] = await Promise.all([
		new Response(child.stdout).text(),
		child.exited
	]);
	return { exitCode, stdout };
};
