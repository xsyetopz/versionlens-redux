import * as vscode from "vscode";
import type { NativeSuggestionIndicators } from "../native/config.ts";
import { configuredValue } from "./configured.ts";

type LegacySuggestionIndicators = {
	Build?: string;
	Directory?: string;
	Error?: string;
	Latest?: string;
	Match?: string;
	NoMatch?: string;
	SatisfiesLatest?: string;
	Updateable?: string;
	UpdateableVulnerable?: string;
};

const suggestionIndicatorKeys = [
	["build", "Build"],
	["latest", "Latest"],
	["satisfiesLatest", "SatisfiesLatest"],
	["directory", "Directory"],
	["error", "Error"],
	["matched", "Match"],
	["noMatch", "NoMatch"],
	["updateable", "Updateable"],
	["updateableVulnerable", "UpdateableVulnerable"],
] as const;

export function suggestionIndicators(): NativeSuggestionIndicators | undefined {
	const config = vscode.workspace.getConfiguration("versionlens");
	const configuredNative = configuredValue<
		NativeSuggestionIndicators | undefined
	>("suggestions.indicators", config);
	const configuredLegacy = configuredValue<
		LegacySuggestionIndicators | undefined
	>("suggestions.indicators", config);
	if (!(configuredNative || configuredLegacy)) {
		return undefined;
	}

	const indicators: NativeSuggestionIndicators = {};
	for (const [nativeKey, legacyKey] of suggestionIndicatorKeys) {
		const value =
			configuredNative?.[nativeKey] ?? configuredLegacy?.[legacyKey];
		if (value !== undefined) {
			indicators[nativeKey] = value;
		}
	}

	return indicators;
}
