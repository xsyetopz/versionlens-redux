import {
  type PackageSuggestion,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  createSuggestion
} from '#domain/packages';
import { prerelease } from 'semver';

/**
 * Creates an updateable suggestion for the latest version.
 * @param requestedVersion Optional version string to suggest.
 * @param name Optional name for the suggestion.
 * @returns A package updateable suggestion.
 */
export function createLatestUpdateable(requestedVersion?: string, name?: string): PackageSuggestion {
  const isPrerelease = prerelease(requestedVersion);

  name ??= isPrerelease
    ? SuggestionStatusText.UpdateLatestPrerelease
    : SuggestionStatusText.UpdateLatest;

  // treat requestedVersion as latest version otherwise '*'
  return {
    name,
    category: SuggestionCategory.Updateable,
    version: requestedVersion || '*',
    type: isPrerelease
      ? SuggestionTypes.prerelease
      : requestedVersion
        ? SuggestionTypes.release
        : SuggestionTypes.tag
  };
}

/**
 * Creates an updateable suggestion for the next maximum version (e.g., minor or patch).
 * @param requestedVersion The version string.
 * @param name The name of the update (e.g., 'minor', 'patch').
 * @returns A package updateable suggestion.
 */
export function createNextMaxUpdateable(requestedVersion: string, name: string): PackageSuggestion {
  return {
    name,
    category: SuggestionCategory.Updateable,
    version: requestedVersion,
    type: SuggestionTypes.release
  };
}

/**
 * Creates a suggestion to update the build component of a version.
 * @param requestedVersion The version string with the new build data.
 * @returns A package updateable suggestion.
 */
export function createBuildUpdateable(requestedVersion: string): PackageSuggestion {
  return {
    name: SuggestionStatusText.UpdateBuild,
    category: SuggestionCategory.Build,
    version: requestedVersion,
    type: SuggestionTypes.release
  };
}

/**
 * Creates an updateable suggestion for a specific tagged prerelease.
 * @param name The name of the prerelease tag (e.g., 'beta').
 * @param version The version string.
 * @returns A package updateable suggestion.
 */
export function createTaggedPreleaseUpdateable(name: string, version: string): PackageSuggestion {
  return createSuggestion(
    name,
    SuggestionCategory.Updateable,
    version,
    SuggestionTypes.prerelease
  );
}