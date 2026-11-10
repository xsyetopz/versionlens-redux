import type { ExtensionState } from "../state.ts";

const toRecord = (value: unknown): Record<string, unknown> =>
	typeof value === "object" && value !== null ? { ...value } : {};

/** The toggles of the editor win over the settings file. */
const withToggles = (state: ExtensionState, value: unknown): unknown => {
	const settings = toRecord(structuredClone(value ?? {}));
	return {
		...settings,
		suggestions: {
			...toRecord(settings["suggestions"]),
			showPrereleasesOnStartup: state.flags.showPrereleases,
		},
	};
};

export { withToggles };
