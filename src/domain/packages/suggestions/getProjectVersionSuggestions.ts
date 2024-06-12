import {
  SuggestionIncrements,
  TPackageSuggestion,
  UpdateableFactory
} from 'domain/packages';
import { inc, valid, prerelease } from 'semver';

export function getProjectVersionSuggestions(projectVersion: string): TPackageSuggestion[] {
  if (!valid(projectVersion)) projectVersion = '0.0.0';

  const isPrerelease = !!prerelease(projectVersion);

  const releaseIncrements = isPrerelease
    ? [SuggestionIncrements.patch, SuggestionIncrements.prerelease]
    : [
      SuggestionIncrements.major,
      SuggestionIncrements.minor,
      SuggestionIncrements.patch
    ];

  const releaseSuggestions = [];
  releaseIncrements.forEach(name => {
    const versionInc = inc(projectVersion, name);
    const suggestion = UpdateableFactory.createNextMaxUpdateable(
      versionInc,
      isPrerelease && name === SuggestionIncrements.patch
        ? SuggestionIncrements.release
        : name
    );

    releaseSuggestions.push(suggestion);
  });

  return releaseSuggestions;
}