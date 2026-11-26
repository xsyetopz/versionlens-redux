/** What the server reports for one open manifest in `versionlens/status`. */
export interface Status {
	busy: boolean;
	dependencies: number;
	enabled: boolean;
	failed: number;
	install: string | null;
	outdated: number;
	prereleases: boolean;
	sortable: boolean;
	uri: string;
	vulnerable: number;
}

const field = (value: object, key: string): unknown => Reflect.get(value, key);

/** The status in a notification payload, or undefined when the payload has no uri. */
export const parseStatus = (value: unknown): Status | undefined => {
	if (typeof value !== "object" || value === null) {
		return undefined;
	}
	const uri = field(value, "uri");
	if (typeof uri !== "string") {
		return undefined;
	}
	const flag = (key: string): boolean => field(value, key) === true;
	const count = (key: string): number => {
		const found = field(value, key);
		return typeof found === "number" ? found : 0;
	};
	const install = field(value, "install");
	return {
		busy: flag("busy"),
		dependencies: count("dependencies"),
		enabled: flag("enabled"),
		failed: count("failed"),
		install: typeof install === "string" ? install : null,
		outdated: count("outdated"),
		prereleases: flag("prereleases"),
		sortable: flag("sortable"),
		uri,
		vulnerable: count("vulnerable")
	};
};

/**
 * The context keys the editor title menus read, from the active manifest's status; no status
 * means the active file is not a manifest.
 */
export const contextKeys = (status?: Status): Record<string, boolean> => ({
	"versionlens.providerActive": status !== undefined,
	"versionlens.providerBusy": status?.busy ?? false,
	"versionlens.providerError": (status?.failed ?? 0) > 0,
	"versionlens.show": status?.enabled ?? false,
	"versionlens.showCustomInstall": (status?.install ?? null) !== null,
	"versionlens.showPrereleases": status?.prereleases ?? false,
	"versionlens.showSortAlphabetically": status?.sortable ?? false
});
