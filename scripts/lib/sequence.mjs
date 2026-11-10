/**
 * Run an async step for each item one after another, stopping at the first
 * rejection.
 * @template T
 * @param {readonly T[]} items
 * @param {(item: T) => Promise<void>} step
 * @returns {Promise<void>}
 */
const sequentially = async (items, step) => {
	const [first, ...rest] = items;
	if (first === undefined) {
		return;
	}
	await step(first);
	await sequentially(rest, step);
};

export { sequentially };
