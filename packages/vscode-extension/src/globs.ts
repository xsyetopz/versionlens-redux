const escape = (text: string): string =>
	text.replaceAll(/[$()+.?[\\\]^{|}]/gu, String.raw`\$&`);

/**
 * A server glob as a regex over a `/` path, matched as the server does: from a path segment to the
 * end, ignoring case, with `*` inside one segment and `**` across any number of directories.
 */
export const globRegex = (glob: string): RegExp => {
	const segments = glob
		.split("/")
		.map((part) =>
			part === "**"
				? "(?:[^/]+/)*"
				: `${part
						.split("*")
						.map((piece) => escape(piece))
						.join("[^/]*")}/`
		)
		.join("");
	return new RegExp(`(?:^|/)${segments.slice(0, -1)}$`, "iu");
};

/** Whether any of the regexes matches the path, with `\` read as `/`. */
export const matchesAny = (regexes: RegExp[], path: string): boolean => {
	const normalized = path.replaceAll("\\", "/");
	return regexes.some((regex) => regex.test(normalized));
};
