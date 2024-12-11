import {
  type PackageSuggestion,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  createSuggestion
} from '#domain/packages';
import { prerelease } from 'semver';

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

export function createNextMaxUpdateable(requestedVersion: string, name: string): PackageSuggestion {
  return {
    name,
    category: SuggestionCategory.Updateable,
    version: requestedVersion,
    type: SuggestionTypes.release
  };
}

export function createTaggedPreleaseUpdateable(name: string, version: string): PackageSuggestion {
  return createSuggestion(
    name,
    SuggestionCategory.Updateable,
    version,
    SuggestionTypes.prerelease
  );
}