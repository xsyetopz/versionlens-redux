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

export function createNotFoundStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.NotFound,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createInternalServerErrorStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.InternalServerError,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createConnectionRefusedStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.ConnectionRefused,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createConnectionResetStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.ConnectionReset,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createForbiddenStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.Forbidden,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createNotAuthorizedStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.NotAuthorized,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createBadRequestStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.BadRequest,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createDirectoryNotFoundStatus(path: string): PackageSuggestion {
  return {
    name: SuggestionStatusText.NotFound,
    category: SuggestionCategory.Error,
    version: path,
    type: SuggestionTypes.status
  };
}

export function createDirectoryStatus(path: string): PackageSuggestion {
  return {
    name: 'file://',
    category: SuggestionCategory.Directory,
    version: path,
    type: SuggestionTypes.status
  };
}

export function createInvalidStatus(requestedVersion: string): PackageSuggestion {
  return {
    name: SuggestionStatusText.InvalidVersion,
    category: SuggestionCategory.Error,
    version: requestedVersion,
    type: SuggestionTypes.status
  };
}

export function createInvalidRangeStatus(): PackageSuggestion {
  return createInvalidStatus('range')
}

export function createNotSupportedStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.NotSupported,
    category: SuggestionCategory.NoMatch,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createNoMatchStatus(): PackageSuggestion {
  return {
    name: SuggestionStatusText.NoMatch,
    category: SuggestionCategory.NoMatch,
    version: '',
    type: SuggestionTypes.status
  };
}

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

export function createSatisifiesLatestStatus(latestVersion: string): PackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.SatisfiesLatest,
    SuggestionCategory.Latest,
    latestVersion,
    SuggestionTypes.status
  )
}

export function createSatisifiesStatus(satisfiesVersion: string): PackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.Satisfies,
    SuggestionCategory.Match,
    satisfiesVersion,
    SuggestionTypes.status
  )
}

export function createFixedStatus(version: string): PackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.Fixed,
    SuggestionCategory.Match,
    version,
    SuggestionTypes.status
  );
}