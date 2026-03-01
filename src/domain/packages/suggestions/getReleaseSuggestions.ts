import {
  type PackageSuggestion,
  type ParsedVersion,
  SuggestionIncrements,
  SuggestionStatusText,
  UpdateableFactory,
  VersionUtils
} from '#domain/packages';
import { compare, inc, major, maxSatisfying } from 'semver';

/**
 * Generates suggestions for release versions (stable updates).
 * @param fixedOrRangedVersion The current version range in the file.
 * @param parsed The parsed version information.
 * @param releases The list of available release versions.
 * @returns An array of release suggestions (latest, major, minor, patch).
 */
export function getReleaseSuggestions(
  fixedOrRangedVersion: string,
  parsed: ParsedVersion,
  releases: string[]
) {
  const potentialSuggestions: Array<[SuggestionStatusText, string]> = [];
  let suggestions: Array<PackageSuggestion> = [];

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
    const nextMajor = VersionUtils.findNextMajor(testVersion, releases);

    if (nextMajor) potentialSuggestions.push(
      [SuggestionStatusText.UpdateMajor, `>${major(testVersion)} <=${nextMajor}`]
    )

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
      suggestions.push(UpdateableFactory.createNextMaxUpdateable(version, name));
    }
  }

  if (!parsed.satisfiesVersion && suggestions.length === 0 && suggestLatest) {
    // No satisfying version -> suggest the latest
    suggestions.push(UpdateableFactory.createLatestUpdateable(parsed.latestRelease));
  }

  // sort the versions (latest first)
  suggestions = suggestions.sort((a, b) => compare(b.version, a.version));

  // suggest build?
  if (parsed.satisfiesVersion) {
    const nextBuild = VersionUtils.findVersionBuilds(
      parsed.satisfiesVersion,
      releases,
      VersionUtils.loosePrereleases
    );
    nextBuild.length > 1 && suggestions.push(UpdateableFactory.createBuildUpdateable(nextBuild.join(',')));
  }

  return suggestions;
}