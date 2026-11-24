// Launches VS Code for a scripted run without taking focus: macOS `open -g -j` starts it hidden
// in the background, so the live test and the benchmark never come in front of the user's work.
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import process from "node:process";

const binary =
	process.env["VSCODE_BIN"] ??
	"/Applications/Visual Studio Code.app/Contents/MacOS/Code";

/** The `.app` bundle that holds the binary, which `open -a` takes. */
export const app = join(binary, "..", "..", "..");

/** The command-line launcher inside the bundle, for extension installs. */
export const cli = join(app, "Contents", "Resources", "app", "bin", "code");

/**
 * Runs one hidden VS Code instance to exit and returns what it printed. `open` does not pass
 * on the app's exit code, so callers judge the run by a line it prints.
 *
 * @param {string[]} args VS Code arguments.
 * @param {Record<string, string>} env Variables added to the instance's environment.
 * @returns {Promise<string>} The instance's standard output.
 */
export const hidden = async (args, env = {}) => {
	const scratch = await mkdtemp(join(tmpdir(), "versionlens-window-"));
	const out = join(scratch, "stdout");
	const child = Bun.spawn(
		[
			"open",
			"-g",
			"-j",
			"-n",
			"-W",
			"-o",
			out,
			...Object.entries(env).flatMap(([key, value]) => [
				"--env",
				`${key}=${value}`
			]),
			"-a",
			app,
			"--args",
			...args
		],
		{ stderr: "inherit", stdout: "inherit" }
	);
	if ((await child.exited) !== 0) {
		throw new Error(`open could not launch ${app}`);
	}
	const file = Bun.file(out);
	const text = (await file.exists()) ? await file.text() : "";
	await rm(scratch, { force: true, recursive: true });
	return text;
};
