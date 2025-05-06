import {
  type PackageSuggestion,
  SuggestionIncrements,
  UpdateableFactory
} from '#domain/packages';
import { inc, prerelease, valid } from 'semver';

export function getProjectVersionSuggestions(projectVersion: string): PackageSuggestion[] {
  if (!valid(projectVersion)) projectVersion = '0.0.0';

  const isPrerelease = !!prerelease(projectVersion);

  const releaseIncrements = isPrerelease
    ? [SuggestionIncrements.patch, SuggestionIncrements.prerelease]
    : [
      SuggestionIncrements.major,
      SuggestionIncrements.minor,
      SuggestionIncrements.patch
    ];

  const releaseSuggestions: PackageSuggestion[] = [];
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