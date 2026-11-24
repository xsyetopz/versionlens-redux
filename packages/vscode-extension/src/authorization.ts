const field = (value: object, key: string): unknown => Reflect.get(value, key);

/** Parses the URL-prefix-to-header map kept in secret storage; anything else is empty. */
export const parseStored = (text?: string): Record<string, string> => {
	if (text === undefined) {
		return {};
	}
	try {
		const parsed: unknown = JSON.parse(text);
		if (typeof parsed !== "object" || parsed === null) {
			return {};
		}
		return Object.fromEntries(
			Object.entries(parsed).filter(
				(entry): entry is [string, string] => typeof entry[1] === "string"
			)
		);
	} catch {
		return {};
	}
};

/**
 * The `versionlens` section with the stored headers merged into `authorization`; a stored header
 * replaces a settings one for the same URL prefix.
 */
export const withAuthorization = (
	section: unknown,
	stored: Record<string, string>
): unknown => {
	if (Object.keys(stored).length === 0) {
		return section;
	}
	const base = typeof section === "object" && section !== null ? section : {};
	const configured = field(base, "authorization");
	const merged = {
		...(typeof configured === "object" && configured !== null
			? configured
			: {}),
		...stored
	};
	return { ...base, authorization: merged };
};
