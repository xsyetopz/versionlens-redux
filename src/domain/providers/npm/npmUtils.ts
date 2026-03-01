import type { ClientResponseSource, HttpClientResponse } from '#domain/clients';
import {
  type PackageSuggestion,
  type SuggestionUpdate,
  PackageSourceType,
  PackageStatusFactory,
  PackageVersionType,
  SuggestionCategory,
  SuggestionTypes,
  UpdateableFactory,
  VersionUtils
} from '#domain/packages';
import type { NpmClientData, TNpmCliConfigParams, } from '#domain/providers/npm';
import { type KeyStringDictionary, fileExists, readFile } from '#domain/utils';
import NpmCliConfig from '@npmcli/config';
import { definitions, flatten, shorthands } from '@npmcli/config/lib/definitions';
import dotenv from 'dotenv';
import { resolve } from 'node:path';

/**
 * Creates flattened NPM client configuration data using @npmcli/config.
 * @param packagePath The path to the package directory.
 * @param options The configuration parameters.
 * @returns A promise resolving to the flattened NPM client data.
 */
export async function createNpmRegistryClientData(
  packagePath: string,
  options: TNpmCliConfigParams
): Promise<NpmClientData> {
  const {
    npmRcFilePath,
    envFilePath,
    userConfigPath,
    hasNpmRcFile,
    hasEnvFile
  } = options;

  const env = {
    ...process.env,
    ...hasEnvFile ? await getDotEnv(envFilePath) : {}
  };

  // load the npm config
  const npmCliConfig = new NpmCliConfig({
    shorthands,
    definitions,
    flatten,
    npmPath: packagePath,
    // use the npmrc path to make npm cli parse the npmrc file
    // otherwise defaults to the package path
    cwd: hasNpmRcFile ? npmRcFilePath : packagePath,
    // ensures user npmrc is parsed by npm
    argv: ['', '', `--userconfig=${userConfigPath}`],
    // pass through .env data
    env
  });

  await npmCliConfig.load();

  return npmCliConfig.flat;
}

/**
 * Converts an NPM error object into an HttpClientResponse.
 * @param error The error object.
 * @param source The client response source.
 * @returns An HttpClientResponse representation of the error.
 */
export function convertNpmErrorToResponse(
  error: any,
  source: ClientResponseSource
): HttpClientResponse {
  return {
    source,
    status: error.code,
    data: error.message,
  }
}

/**
 * Creates package suggestions based on an NPM error code.
 * @param npmErrorCode The NPM error code string.
 * @returns An array of package suggestions.
 */
export function createNpmSuggestionFromErrorCode(npmErrorCode: string): PackageSuggestion[] {
  switch (npmErrorCode) {
    case 'ECONNREFUSED':
      return [PackageStatusFactory.createConnectionRefusedStatus()];
    case 'ECONNRESET':
      return [PackageStatusFactory.createConnectionResetStatus()];
    case 'EUNSUPPORTEDPROTOCOL':
      return [PackageStatusFactory.createNotSupportedStatus()];
    case 'EINVALIDTAGNAME':
      return [
        PackageStatusFactory.createInvalidStatus(''),
        UpdateableFactory.createLatestUpdateable('latest')
      ];
    case 'EINVALIDPACKAGENAME':
      return [PackageStatusFactory.createInvalidStatus('')];
    default:
      const errorNum = Number.parseInt(npmErrorCode.substring(1));
      if (Number.isNaN(errorNum)) {
        return [{
          name: npmErrorCode,
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }];
      }

      const suggestion = PackageStatusFactory.createFromHttpStatus(errorNum);
      return [
        suggestion || {
          name: npmErrorCode,
          category: SuggestionCategory.Error,
          version: '',
          type: SuggestionTypes.status
        }
      ];
  }
}

/**
 * Resolves the full path to a dotfile (like .npmrc) by checking multiple directories.
 * @param dotFileName The name of the dotfile.
 * @param cwds The directories to search in.
 * @returns A promise resolving to the full path of the dotfile, or an empty string.
 */
export async function resolveDotFilePath(
  dotFileName: string,
  cwds: Array<string>
): Promise<string> {
  for (const cwd of cwds) {
    const checkPath = resolve(cwd, dotFileName);
    const dotFileExists = await fileExists(checkPath);
    if (dotFileExists) return checkPath;
  }

  return '';
}

/**
 * Parses a .env file and returns its content as a dictionary.
 * @param envPath The path to the .env file.
 * @returns A promise resolving to a dictionary of environment variables.
 */
export async function getDotEnv(envPath: string): Promise<KeyStringDictionary> {
  // return the parsed env object
  return dotenv.parse(await readFile(envPath));
}

/**
 * Custom function to replace versions in NPM package files, handling GitHub and Alias versions.
 * @param suggestionUpdate The suggestion update information.
 * @returns The new version string.
 */
export function npmReplaceVersion(suggestionUpdate: SuggestionUpdate): string {
  if (suggestionUpdate.packageSource === PackageSourceType.Github) {
    return replaceGitVersion(suggestionUpdate);
  }

  if (suggestionUpdate.packageVersionType === PackageVersionType.Alias) {
    return replaceAliasVersion(suggestionUpdate);
  }

  // fallback to default
  return VersionUtils.preserveLeadingRange(
    suggestionUpdate.parsedVersion,
    suggestionUpdate.suggestionVersion
  );
}

/**
 * Replaces a version in a GitHub dependency string.
 * @param suggestionUpdate The suggestion update information.
 * @returns The updated dependency string.
 */
function replaceGitVersion(suggestionUpdate: SuggestionUpdate): string {
  if (!suggestionUpdate.fetchedVersion) return suggestionUpdate.parsedVersion;

  return suggestionUpdate.parsedVersion.replace(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  )
}

/**
 * Replaces a version in an NPM alias dependency string.
 * @param suggestionUpdate The suggestion update information.
 * @returns The updated dependency string.
 */
function replaceAliasVersion(suggestionUpdate: SuggestionUpdate): string {
  if (!suggestionUpdate.fetchedVersion || !suggestionUpdate.fetchedName) {
    return suggestionUpdate.parsedVersion;
  }

  // preserve the leading symbol from the existing version
  const preservedLeadingVersion = VersionUtils.preserveLeadingRange(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  );

  const firstColon = suggestionUpdate.parsedVersion.indexOf(':');
  const registry = suggestionUpdate.parsedVersion.substring(0, firstColon)
  return `${registry}:${suggestionUpdate.fetchedName}@${preservedLeadingVersion}`;
}