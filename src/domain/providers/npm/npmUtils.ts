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

export function convertNpmErrorToResponse(
  error,
  source: ClientResponseSource
): HttpClientResponse {
  return {
    source,
    status: error.code,
    data: error.message,
  }
}

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

      return [PackageStatusFactory.createFromHttpStatus(errorNum)];
  }
}

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

export async function getDotEnv(envPath: string): Promise<KeyStringDictionary> {
  // return the parsed env object
  return dotenv.parse(await readFile(envPath));
}

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

function replaceGitVersion(suggestionUpdate: SuggestionUpdate): string {
  return suggestionUpdate.parsedVersion.replace(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  )
}

function replaceAliasVersion(suggestionUpdate: SuggestionUpdate): string {
  // preserve the leading symbol from the existing version
  const preservedLeadingVersion = VersionUtils.preserveLeadingRange(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  );

  const firstColon = suggestionUpdate.parsedVersion.indexOf(':');
  const registry = suggestionUpdate.parsedVersion.substring(0, firstColon)
  return `${registry}:${suggestionUpdate.fetchedName}@${preservedLeadingVersion}`;
}