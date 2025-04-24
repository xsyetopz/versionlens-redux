import {
  type HttpClientResponse,
  type JsonClientResponse,
  ClientResponseSource
} from '#domain/clients';
import {
  type PackageClientResponse,
  type PackageClientResponseStatus,
  type PackageResource,
  type PackageSuggestion,
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  UpdateableFactory
} from '#domain/packages';
import { fileExists } from '#domain/utils';
import { dirname, join } from 'node:path';

export function create(
  source: PackageSourceType,
  responseStatus: PackageClientResponseStatus,
  suggestions: Array<PackageSuggestion>
): PackageClientResponse {

  return {
    source,
    type: null,
    resolved: null,
    responseStatus,
    suggestions
  };

}

export function createInvalidVersion(
  responseStatus: PackageClientResponseStatus,
  type: PackageVersionType
): PackageClientResponse {
  const source: PackageSourceType = PackageSourceType.Registry;
  const suggestions: Array<PackageSuggestion> = [
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
  responseStatus: PackageClientResponseStatus,
  latestVersion?: string
): PackageClientResponse {

  const suggestions: Array<PackageSuggestion> = [
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

export function createNotSupported(): PackageClientResponse {
  return create(
    PackageSourceType.Registry,
    { source: ClientResponseSource.remote, status: 200 },
    [PackageStatusFactory.createNotSupportedStatus()]
  )
}

export function createFixed(
  source: PackageSourceType,
  responseStatus: PackageClientResponseStatus,
  type: PackageVersionType,
  fixedVersion: string
): PackageClientResponse {

  const suggestions: Array<PackageSuggestion> = [
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
  path: string,
  source = PackageSourceType.Directory
): Promise<PackageClientResponse> {
  const type = PackageVersionType.Version;
  const resolvedPath = join(dirname(packageFilePath), path);
  const exists = await fileExists(resolvedPath)

  const suggestions: Array<PackageSuggestion> = [
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
  requested: PackageResource
): Promise<PackageClientResponse> {

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

export function createGit(): PackageClientResponse {
  return createFixed(
    PackageSourceType.Git,
    createResponseStatus(ClientResponseSource.local, 0),
    PackageVersionType.Committish,
    'git repository'
  );
}

export function createNoSuggestions(): PackageClientResponse {
  return create(
    PackageSourceType.Registry,
    { source: ClientResponseSource.remote, status: 200 },
    []
  )
}

export function mapStatusFromHttpResponse(
  response: HttpClientResponse
): PackageClientResponseStatus {
  return {
    source: response.source,
    status: response.status
  };
}

export function mapStatusFromJsonResponse(
  response: JsonClientResponse<any>
): PackageClientResponseStatus {
  return {
    source: response.source,
    status: response.status
  };
}

export function createResponseStatus(
  source: ClientResponseSource,
  status: number
): PackageClientResponseStatus {
  return { source, status };
}