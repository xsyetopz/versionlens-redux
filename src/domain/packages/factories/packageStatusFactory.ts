import {
  type PackageSuggestion,
  PackageStatusFactory,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  createSuggestion
} from '#domain/packages';
import { Nullable } from '#domain/utils';
import semver from 'semver';

/**
 * Creates a status suggestion based on an HTTP status code.
 * @param status The HTTP status code or error code.
 * @returns A package status suggestion, or null if the status is not handled.
 */
export function createFromHttpStatus(status: number | string): Nullable<PackageSuggestion> {

  if (status == 400)
    return PackageStatusFactory.createBadRequestStatus();
  else if (status == 401)
    return PackageStatusFactory.createNotAuthorizedStatus();
  else if (status == 403)
    return PackageStatusFactory.createForbiddenStatus();
  else if (status == 404)
    return PackageStatusFactory.createNotFoundStatus();
  else if (status == 500)
    return PackageStatusFactory.createInternalServerErrorStatus();

  return null;
}

/**
 * Creates a 'not found' status suggestion.
 * @returns A package status suggestion.
 */
export function createNotFoundStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.NotFound,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates an 'internal server error' status suggestion.
 * @returns A package status suggestion.
 */
export function createInternalServerErrorStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.InternalServerError,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'connection refused' status suggestion.
 * @returns A package status suggestion.
 */
export function createConnectionRefusedStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.ConnectionRefused,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'connection reset' status suggestion.
 * @returns A package status suggestion.
 */
export function createConnectionResetStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.ConnectionReset,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'forbidden' status suggestion.
 * @returns A package status suggestion.
 */
export function createForbiddenStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.Forbidden,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'not authorized' status suggestion.
 * @returns A package status suggestion.
 */
export function createNotAuthorizedStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.NotAuthorized,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'bad request' status suggestion.
 * @returns A package status suggestion.
 */
export function createBadRequestStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.BadRequest,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'directory not found' status suggestion.
 * @param path The path that was not found.
 * @returns A package status suggestion.
 */
export function createDirectoryNotFoundStatus(path: string): PackageSuggestion {
  return {
    name: SuggestionStatusText.NotFound,
    category: SuggestionCategory.Error,
    version: path,
    type: SuggestionTypes.status
  };
}

/**
 * Creates a status suggestion for a local directory.
 * @param path The path to the directory.
 * @returns A package status suggestion.
 */
export function createDirectoryStatus(path: string): PackageSuggestion {
  return {
    name: 'file://',
    category: SuggestionCategory.Directory,
    version: path,
    type: SuggestionTypes.status
  };
}

/**
 * Creates an 'invalid version' status suggestion.
 * @param requestedVersion The version that was found to be invalid.
 * @returns A package status suggestion.
 */
export function createInvalidStatus(requestedVersion: string): PackageSuggestion {
  return {
    name: SuggestionStatusText.InvalidVersion,
    category: SuggestionCategory.Error,
    version: requestedVersion,
    type: SuggestionTypes.status
  };
}

/**
 * Creates an 'invalid range' status suggestion.
 * @returns A package status suggestion.
 */
export function createInvalidRangeStatus(): PackageSuggestion {
  return createInvalidStatus('range')
}

/**
 * Creates a 'not supported' status suggestion.
 * @returns A package status suggestion.
 */
export function createNotSupportedStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.NotSupported,
    category: SuggestionCategory.NoMatch,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'no match' status suggestion.
 * @returns A package status suggestion.
 */
export function createNoMatchStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.NoMatch,
    category: SuggestionCategory.NoMatch,
    version: '',
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'matches latest' status suggestion.
 * @param latestVersion The latest version string.
 * @returns A package status suggestion.
 */
export function createMatchesLatestStatus(latestVersion: string): PackageSuggestion {
  const isPrerelease = semver.prerelease(latestVersion);

  const name = isPrerelease
    ? SuggestionStatusText.LatestIsPrerelease
    : SuggestionStatusText.Latest;

  return {
    name,
    category: SuggestionCategory.Latest,
    version: latestVersion,
    type: SuggestionTypes.status
  };
}

/**
 * Creates a 'satisfies latest' status suggestion.
 * @param latestVersion The latest version string.
 * @returns A package status suggestion.
 */
export function createSatisifiesLatestStatus(latestVersion: string): PackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.SatisfiesLatest,
    SuggestionCategory.SatisfiesLatest,
    latestVersion,
    SuggestionTypes.status
  )
}

/**
 * Creates a 'satisfies' status suggestion.
 * @param satisfiesVersion The version that satisfies the range.
 * @returns A package status suggestion.
 */
export function createSatisifiesStatus(satisfiesVersion: string): PackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.Satisfies,
    SuggestionCategory.Match,
    satisfiesVersion,
    SuggestionTypes.status
  )
}

/**
 * Creates a 'fixed branch' status suggestion.
 * @param branch The branch name.
 * @returns A package status suggestion.
 */
export function createFixedBranchStatus(branch: string): PackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.FixedBranch,
    SuggestionCategory.Match,
    branch,
    SuggestionTypes.status
  );
}

/**
 * Creates a 'fixed' status suggestion.
 * @param version The fixed version string.
 * @returns A package status suggestion.
 */
export function createFixedStatus(version: string): PackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.Fixed,
    SuggestionCategory.Match,
    version,
    SuggestionTypes.status
  );
}