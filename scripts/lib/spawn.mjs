/**
 * @typedef {object} SpawnOptions
 * @property {string} [cwd]
 * @property {Record<string, string | undefined>} [env]
 * @property {Uint8Array} [stdin]
 */

/**
 * @typedef {object} Captured
 * @property {number} exitCode
 * @property {string} stdout
 * @property {string} stderr
 */

/**
 * Run a command with its output captured.
 * @param {string[]} command
 * @param {SpawnOptions} [options]
 * @returns {Promise<Captured>}
 */
const capture = async (command, options = {}) => {
	const child = Bun.spawn(command, {
		...options,
		stderr: "pipe",
		stdout: "pipe",
	});
	const [stdout, stderr, exitCode] = await Promise.all([
		new Response(child.stdout).text(),
		new Response(child.stderr).text(),
		child.exited,
	]);
	return { exitCode, stderr, stdout };
};

/**
 * Run a command with its output passed through to this process.
 * @param {string[]} command
 * @param {SpawnOptions} [options]
 * @returns {Promise<number>} the exit code
 */
const inherit = (command, options = {}) =>
	Bun.spawn(command, {
		...options,
		stderr: "inherit",
		stdout: "inherit",
	}).exited;

export { capture, inherit };
