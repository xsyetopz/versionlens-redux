const semverPattern =
	/^(?<major>0|[1-9]\d*)\.(?<minor>0|[1-9]\d*)\.(?<patch>0|[1-9]\d*)(?:-(?<prerelease>[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+(?<build>[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/u;
const numericIdentifierPattern = /^\d+$/u;

/** @type {(message: string) => never} */
const fail = (message) => {
	throw new Error(message);
};

/**
 * @typedef {object} Semver
 * @property {bigint} major
 * @property {bigint} minor
 * @property {bigint} patch
 * @property {string[]} prerelease
 */

/**
 * @param {unknown} value
 * @returns {Semver}
 */
const parseSemver = (value) => {
	if (
		typeof value !== "string" ||
		value.length === 0 ||
		value.trim() !== value
	) {
		fail(`invalid SemVer: ${JSON.stringify(value)}`);
	}
	const match = semverPattern.exec(value);
	const [, major, minor, patch, prereleaseText] = match ?? [];
	if (major === undefined || minor === undefined || patch === undefined) {
		return fail(`invalid SemVer: ${JSON.stringify(value)}`);
	}
	const prerelease = prereleaseText?.split(".") ?? [];
	for (const identifier of prerelease) {
		if (
			numericIdentifierPattern.test(identifier) &&
			identifier.length > 1 &&
			identifier.startsWith("0")
		) {
			fail(
				`invalid SemVer numeric prerelease identifier: ${JSON.stringify(identifier)}`,
			);
		}
	}
	return {
		major: BigInt(major),
		minor: BigInt(minor),
		patch: BigInt(patch),
		prerelease,
	};
};

/**
 * @param {string | undefined} left
 * @param {string | undefined} right
 * @returns {number}
 */
const comparePrereleaseIdentifier = (left, right) => {
	if (left === undefined || right === undefined) {
		if (left === right) {
			return 0;
		}
		if (left === undefined) {
			return -1;
		}
		return 1;
	}
	if (left === right) {
		return 0;
	}
	const leftNumeric = numericIdentifierPattern.test(left);
	const rightNumeric = numericIdentifierPattern.test(right);
	if (leftNumeric && rightNumeric) {
		if (BigInt(left) < BigInt(right)) {
			return -1;
		}
		return 1;
	}
	if (leftNumeric !== rightNumeric) {
		if (leftNumeric) {
			return -1;
		}
		return 1;
	}
	if (left < right) {
		return -1;
	}
	return 1;
};

/**
 * @param {string[]} left
 * @param {string[]} right
 * @returns {number}
 */
const comparePrerelease = (left, right) => {
	if (left.length === 0 || right.length === 0) {
		if (left.length === right.length) {
			return 0;
		}
		if (left.length === 0) {
			return 1;
		}
		return -1;
	}
	const length = Math.max(left.length, right.length);
	for (let index = 0; index < length; index += 1) {
		const comparison = comparePrereleaseIdentifier(left[index], right[index]);
		if (comparison !== 0) {
			return comparison;
		}
	}
	return 0;
};

/** @type {readonly ["major", "minor", "patch"]} */
const versionFields = ["major", "minor", "patch"];

/**
 * @param {Semver} left
 * @param {Semver} right
 * @returns {number}
 */
const compareSemver = (left, right) => {
	for (const field of versionFields) {
		if (left[field] !== right[field]) {
			if (left[field] < right[field]) {
				return -1;
			}
			return 1;
		}
	}
	return comparePrerelease(left.prerelease, right.prerelease);
};

/**
 * @typedef {object} ReplaceExactlyOptions
 * @property {string} source
 * @property {RegExp} pattern
 * @property {string | ((match: string, ...groups: string[]) => string)} replacement
 * @property {number} expectedCount
 * @property {string} path
 */

/**
 * @param {ReplaceExactlyOptions} options
 * @returns {string}
 */
const replaceExactly = ({
	source,
	pattern,
	replacement,
	expectedCount,
	path,
}) => {
	let count = 0;
	/**
	 * @param {string} match
	 * @param {...string} groups
	 * @returns {string}
	 */
	const replacer = (match, ...groups) => {
		count += 1;
		if (typeof replacement === "function") {
			return replacement(match, ...groups);
		}
		return replacement;
	};
	const output = source.replace(pattern, replacer);
	if (count !== expectedCount) {
		fail(
			`${path}: expected ${expectedCount} version occurrence(s), found ${count}`,
		);
	}
	return output;
};

export { compareSemver, fail, parseSemver, replaceExactly };
