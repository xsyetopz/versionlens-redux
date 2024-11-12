import { ClientResponseSource } from '#domain/clients';
import {
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  TPackageClientResponse,
  TPackageClientResponseStatus,
  TPackageResource,
  TPackageSuggestion,
  UpdateableFactory
} from '#domain/packages';
import { fileExists } from '#domain/utils';
import { dirname, join } from 'node:path';

export function create(
  source: PackageSourceType,
  responseStatus: TPackageClientResponseStatus,
  suggestions: Array<TPackageSuggestion>
): TPackageClientResponse {

  return {
    source,
    type: null,
    resolved: null,
    responseStatus,
    suggestions
  };

}

export function createInvalidVersion(
  responseStatus: TPackageClientResponseStatus,
  type: PackageVersionType
): TPackageClientResponse {
  const source: PackageSourceType = PackageSourceType.Registry;
  const suggestions: Array<TPackageSuggestion> = [
    PackageStatusFactory.createInvalidStatus(''),
    UpdateableFactory.createLatestUpdateable(),
  ];

  return {
    source,
    type,
    responseStatus,
    resolved: null,
    suggestions
  };
}

export function createNoMatch(
  source: PackageSourceType,
  type: PackageVersionType,
  responseStatus: TPackageClientResponseStatus,
  latestVersion?: string
): TPackageClientResponse {

  const suggestions: Array<TPackageSuggestion> = [
    PackageStatusFactory.createNoMatchStatus(),
    UpdateableFactory.createLatestUpdateable(latestVersion),
  ];

  return {
    source,
    type,
    responseStatus,
    resolved: null,
    suggestions
  };
}

export function createFixed(
  source: PackageSourceType,
  responseStatus: TPackageClientResponseStatus,
  type: PackageVersionType,
  fixedVersion: string
): TPackageClientResponse {

  const suggestions: Array<TPackageSuggestion> = [
    PackageStatusFactory.createFixedStatus(fixedVersion)
  ];

  return {
    source,
    type,
    responseStatus,
    resolved: null,
    suggestions
  };
}

export async function createDirectory(
  packageName: string,
  packageFilePath: string,
  path: string
): Promise<TPackageClientResponse> {
  const source = PackageSourceType.Directory;
  const type = PackageVersionType.Version;
  const resolvedPath = join(dirname(packageFilePath), path);
  const exists = await fileExists(resolvedPath)

  const suggestions: Array<TPackageSuggestion> = [
    exists
      ? PackageStatusFactory.createDirectoryStatus(path)
      : PackageStatusFactory.createDirectoryNotFoundStatus(path)
  ];

  const responseStatus = createResponseStatus(
    ClientResponseSource.local,
    exists ? 200 : 404
  );

  const resolved = {
    name: packageName,
    version: resolvedPath,
  };

  return {
    source,
    type,
    responseStatus,
    resolved: exists ? resolved : null,
    suggestions
  };
}

const fileDependencyRegex = /^file:(.*)$/;
export async function createDirectoryFromFileProtocol(
  requested: TPackageResource
): Promise<TPackageClientResponse> {

  const fileRegExpResult = fileDependencyRegex.exec(requested.version);
  if (!fileRegExpResult) {
    return createInvalidVersion(
      createResponseStatus(ClientResponseSource.local, 400),
      <any>PackageSourceType.Directory
    );
  }

  const path = fileRegExpResult[1];

  return await createDirectory(requested.name, requested.path, path);
}

export function createGit(): TPackageClientResponse {
  return createFixed(
    PackageSourceType.Git,
    createResponseStatus(ClientResponseSource.local, 0),
    PackageVersionType.Committish,
    'git repository'
  );
}

export function createResponseStatus(
  source: ClientResponseSource,
  status: number
): TPackageClientResponseStatus {
  return {
    source,
    status
  };
}