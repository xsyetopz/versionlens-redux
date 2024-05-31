import {
  PackageStatusFactory,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  TPackageSuggestion,
  createSuggestion
} from 'domain/packages';
import { Nullable } from 'domain/utils';
import semver from 'semver';

export function createFromHttpStatus(status: number | string): Nullable<TPackageSuggestion> {

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

export function createNotFoundStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.NotFound,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createInternalServerErrorStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.InternalServerError,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createConnectionRefusedStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.ConnectionRefused,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createConnectionResetStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.ConnectionReset,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createForbiddenStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.Forbidden,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createNotAuthorizedStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.NotAuthorized,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createBadRequestStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.BadRequest,
    category: SuggestionCategory.Error,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createDirectoryNotFoundStatus(path: string): TPackageSuggestion {
  return {
    name: SuggestionStatusText.DirectoryNotFound,
    category: SuggestionCategory.Error,
    version: path,
    type: SuggestionTypes.status
  };
}

export function createDirectoryStatus(path: string): TPackageSuggestion {
  return {
    name: 'file://',
    category: SuggestionCategory.Directory,
    version: path,
    type: SuggestionTypes.status
  };
}

export function createInvalidStatus(requestedVersion: string): TPackageSuggestion {
  return {
    name: SuggestionStatusText.Invalid,
    category: SuggestionCategory.Error,
    version: requestedVersion,
    type: SuggestionTypes.status
  };
}

export function createNotSupportedStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.NotSupported,
    category: SuggestionCategory.NoMatch,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createNoMatchStatus(): TPackageSuggestion {
  return {
    name: SuggestionStatusText.NoMatch,
    category: SuggestionCategory.NoMatch,
    version: '',
    type: SuggestionTypes.status
  };
}

export function createMatchesLatestStatus(latestVersion: string): TPackageSuggestion {
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

export function createSatisifiesLatestStatus(latestVersion: string): TPackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.SatisfiesLatest,
    SuggestionCategory.Match,
    latestVersion,
    SuggestionTypes.status
  )
}

export function createSatisifiesStatus(satisfiesVersion: string): TPackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.Satisfies,
    SuggestionCategory.Match,
    satisfiesVersion,
    SuggestionTypes.status
  )
}

export function createFixedStatus(version: string): TPackageSuggestion {
  return createSuggestion(
    SuggestionStatusText.Fixed,
    SuggestionCategory.Match,
    version,
    SuggestionTypes.status
  );
}