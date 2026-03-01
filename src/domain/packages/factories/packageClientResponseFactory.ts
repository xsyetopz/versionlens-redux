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

/**
 * Creates a standard package client response.
 * @param source The source of the package.
 * @param responseStatus The status of the response.
 * @param suggestions The list of suggestions.
 * @returns A package client response.
 */
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

/**
 * Creates a package client response for an invalid version.
 * @param responseStatus The status of the response.
 * @param type The type of version requested.
 * @returns A package client response with invalid version suggestions.
 */
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

/**
 * Creates a package client response when no matching version is found.
 * @param source The source of the package.
 * @param type The type of version requested.
 * @param responseStatus The status of the response.
 * @param latestVersion Optional latest version to suggest.
 * @returns A package client response with no match suggestions.
 */
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

/**
 * Creates a package client response for unsupported protocols or providers.
 * @returns A package client response with a not supported suggestion.
 */
export function createNotSupported(): PackageClientResponse {
  return create(
    PackageSourceType.Registry,
    { source: ClientResponseSource.remote, status: 200 },
    [PackageStatusFactory.createNotSupportedStatus()]
  )
}

/**
 * Creates a package client response for a fixed version (no updates possible).
 * @param source The source of the package.
 * @param responseStatus The status of the response.
 * @param type The type of version.
 * @param fixedVersion The fixed version string.
 * @returns A package client response with a fixed status suggestion.
 */
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

/**
 * Creates a package client response for a local directory dependency.
 * @param packageName The name of the package.
 * @param packageFilePath The path to the file containing the dependency.
 * @param path The relative path to the directory.
 * @param source The source type (defaults to Directory).
 * @returns A promise resolving to a package client response.
 */
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
/**
 * Creates a package client response for a 'file:' protocol dependency.
 * @param requested The package resource information.
 * @returns A promise resolving to a package client response.
 */
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

/**
 * Creates a package client response for a Git dependency.
 * @returns A package client response for Git.
 */
export function createGit(): PackageClientResponse {
  return createFixed(
    PackageSourceType.Git,
    createResponseStatus(ClientResponseSource.local, 0),
    PackageVersionType.Committish,
    'git repository'
  );
}

/**
 * Creates a package client response with no suggestions.
 * @returns A package client response.
 */
export function createNoSuggestions(): PackageClientResponse {
  return create(
    PackageSourceType.Registry,
    { source: ClientResponseSource.remote, status: 200 },
    []
  )
}

/**
 * Maps an HttpClientResponse to a PackageClientResponseStatus.
 * @param response The HTTP client response.
 * @returns The package client response status.
 */
export function mapStatusFromHttpResponse(
  response: HttpClientResponse
): PackageClientResponseStatus {
  return {
    source: response.source,
    status: response.status
  };
}

/**
 * Maps a JsonClientResponse to a PackageClientResponseStatus.
 * @param response The JSON client response.
 * @returns The package client response status.
 */
export function mapStatusFromJsonResponse(
  response: JsonClientResponse<any>
): PackageClientResponseStatus {
  return {
    source: response.source,
    status: response.status
  };
}

/**
 * Creates a new PackageClientResponseStatus.
 * @param source The source of the response.
 * @param status The status code.
 * @returns A package client response status.
 */
export function createResponseStatus(
  source: ClientResponseSource,
  status: number
): PackageClientResponseStatus {
  return { source, status };
}