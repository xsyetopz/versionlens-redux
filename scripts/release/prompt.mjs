import process from "node:process";
import { createInterface } from "node:readline/promises";
import { Writable } from "node:stream";

/**
 * @typedef {object} Prompt
 * @property {(prompt: string, options?: { hidden?: boolean }) => Promise<string>} ask
 * @property {() => void} close
 */

/** @returns {Prompt} */
const createPrompt = () => {
	let muteInputEcho = false;
	const promptOutput = new Writable({
		/**
		 * @param {string | Uint8Array} chunk
		 * @param {BufferEncoding} encoding
		 * @param {() => void} callback
		 */
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
	return {
		ask: async (prompt, options = {}) => {
			process.stdout.write(prompt);
			muteInputEcho = options.hidden === true;
			try {
				const { done, value } = await lines.next();
				if (done === true) {
					throw new Error("input closed before configuration was complete");
				}
				return value;
			} finally {
				if (options.hidden === true) {
					process.stdout.write("\n");
				}
				muteInputEcho = false;
			}
		},
		close: () => {
			muteInputEcho = false;
			input.close();
		},
	};
};

export { createPrompt };
