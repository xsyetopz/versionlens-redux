import { type Uri, workspace } from "#vscode-host";
import type { NativeSuggestionIndicators } from "../native/config.ts";
import { configuredValue } from "./configured.ts";

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

type LegacySuggestionIndicators = Readonly<Record<string, string | undefined>>;

export function suggestionIndicators(
  resource?: Uri,
): NativeSuggestionIndicators | undefined {
  const config = workspace.getConfiguration("versionlens", resource);
  const configuredNative = configuredValue<
    NativeSuggestionIndicators | undefined
  >("suggestions.indicators", config);
  const configuredLegacy = configuredValue<
    LegacySuggestionIndicators | undefined
  >("suggestions.indicators", config);
  if (!(configuredNative || configuredLegacy)) {
    return;
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
