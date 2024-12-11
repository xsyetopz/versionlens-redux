import {
  type PackageSuggestion,
  type TParsedVersion,
  SuggestionIncrements,
  SuggestionStatusText,
  UpdateableFactory
} from '#domain/packages';
import { compare, inc, maxSatisfying } from 'semver';

export function getReleaseSuggestions(
  fixedOrRangedVersion: string,
  parsed: TParsedVersion,
  releases: string[]
) {
  const potentialSuggestions: Array<[SuggestionStatusText, string]> = [];
  const suggestions: Array<PackageSuggestion> = [];

  // suggest latest?
  const suggestLatest = !parsed.isLatest || parsed.hasRangeUpdate;
  if (suggestLatest) {
    potentialSuggestions.push([SuggestionStatusText.UpdateLatest, parsed.latestRelease]);
  }

  // suggest minor and\or patch?
  if (parsed.satisfiesVersion || parsed.isFixedVersion) {
    const testVersion = parsed.satisfiesVersion ?? fixedOrRangedVersion;
    const nextMaxMajor = inc(testVersion, SuggestionIncrements.major);
    const nextMaxMinor = inc(testVersion, SuggestionIncrements.minor);
    const nextMaxPatch = inc(testVersion, SuggestionIncrements.patch);

    potentialSuggestions.push(
      [SuggestionStatusText.UpdateMinor, `>=${nextMaxMinor} <${nextMaxMajor}`],
      [SuggestionStatusText.UpdatePatch, `>=${nextMaxPatch} <${nextMaxMinor}`],
    );
  }

  // suggest ranged?
  if (!parsed.isLatest && parsed.hasRangeUpdate) {
    potentialSuggestions.push([SuggestionStatusText.UpdateRange, parsed.satisfiesVersion]);
  }

  // reduce the potential suggestions
  for (const [name, range] of potentialSuggestions) {
    const version = maxSatisfying(releases, range);
    // Only suggest if the version is not already suggested
    if (version && !suggestions.some((s) => s.version === version)) {
      suggestions.push(
        UpdateableFactory.createNextMaxUpdateable(version, name)
      );
    }
  }

  if (!parsed.satisfiesVersion && suggestions.length === 0 && suggestLatest) {
    // No satisfying version -> suggest the latest
    suggestions.push(UpdateableFactory.createLatestUpdateable(parsed.latestRelease));
  }

  // sort the versions (latest first)
  return suggestions.sort((a, b) => compare(b.version, a.version));
}