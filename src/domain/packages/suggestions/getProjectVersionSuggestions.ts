import {
  SuggestionIncrements,
  TPackageSuggestion,
  UpdateableFactory
} from 'domain/packages';
import { inc, valid } from 'semver';

export function getProjectVersionSuggestions(projectVersion: string): TPackageSuggestion[] {
  if (!valid(projectVersion)) projectVersion = '0.0.0';

  const releaseIncrements = [
    SuggestionIncrements.major,
    SuggestionIncrements.minor,
    SuggestionIncrements.patch
  ];

  const releaseSuggestions = [];
  releaseIncrements.forEach(x => {
    const versionInc = inc(projectVersion, x)
    releaseSuggestions.push(
      UpdateableFactory.createNextMaxUpdateable(versionInc, x)
    );
  });

  return releaseSuggestions;
}