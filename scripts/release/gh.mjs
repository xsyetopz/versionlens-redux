import process from "node:process";

/** A child process that exited non-zero; its own output already explained why. */
class CommandFailed extends Error {
	/** @param {number} exitCode */
	constructor(exitCode) {
		super(`command exited with ${exitCode}`);
		this.exitCode = exitCode;
	}
}

/**
 * @param {string[]} command
 * @param {{ capture?: boolean, quiet?: boolean, stdin?: Uint8Array }} [options]
 * @returns {Promise<string>} trimmed stdout when capturing, otherwise empty
 */
const run = async (command, options = {}) => {
	/** @type {"inherit"} */
	const inherited = "inherit";
	const base = {
		env: { ...Bun.env, GH_PAGER: "cat" },
		stderr: inherited,
		stdin: options.stdin ?? inherited,
	};
	let output = "";
	let exitCode = 0;
	if (options.capture === true) {
		const child = Bun.spawn(command, { ...base, stdout: "pipe" });
		[output, exitCode] = await Promise.all([
			new Response(child.stdout).text(),
			child.exited,
		]);
	} else {
		const child = Bun.spawn(command, {
			...base,
			stdout: options.quiet === true ? "ignore" : "inherit",
		});
		exitCode = await child.exited;
	}
	if (exitCode !== 0) {
		throw new CommandFailed(exitCode);
	}
	return output.trim();
};

/** @param {string} name */
const requireCommand = async (name) => {
	const child = Bun.spawn([name, "--version"], {
		stderr: "ignore",
		stdout: "ignore",
	});
	if ((await child.exited) !== 0) {
		throw new Error(`${name} is required`);
	}
};

/**
 * Report a failure the way the command line expects: a failed child keeps its
 * exit code, anything else prints its message and exits 1.
 * @param {unknown} error
 * @returns {void}
 */
const reportFailure = (error) => {
	if (error instanceof CommandFailed) {
		process.exitCode = error.exitCode;
		return;
	}
	console.error(error instanceof Error ? error.message : String(error));
	process.exitCode = 1;
};

export { CommandFailed, reportFailure, requireCommand, run };
