import NpmCliConfig from '@npmcli/config';
import { ClientResponseSource, HttpClientResponse } from 'domain/clients';
import {
  PackageSourceType,
  PackageVersionType,
  TSuggestionUpdate,
  VersionUtils
} from 'domain/packages';
import { KeyStringDictionary, fileExists, readFile } from 'domain/utils';
import dotenv from 'dotenv';
import { resolve } from 'node:path';

export function npmReplaceVersion(suggestionUpdate: TSuggestionUpdate): string {
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

function replaceGitVersion(suggestionUpdate: TSuggestionUpdate): string {
  return suggestionUpdate.parsedVersion.replace(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  )
}

function replaceAliasVersion(suggestionUpdate: TSuggestionUpdate): string {
  // preserve the leading symbol from the existing version
  const preservedLeadingVersion = VersionUtils.preserveLeadingRange(
    suggestionUpdate.fetchedVersion,
    suggestionUpdate.suggestionVersion
  );

  return `npm:${suggestionUpdate.fetchedName}@${preservedLeadingVersion}`;
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

export async function resolveDotFilePath(
  dotFileName: string,
  cwds: Array<string>
): Promise<string> {
  for (const cwd of cwds) {
    const checkPath = resolve(cwd, dotFileName);
    const dotFileExists = await fileExists(checkPath);
    if (dotFileExists) return checkPath;
  }

  return "";
}

export async function getDotEnv(envPath: string): Promise<KeyStringDictionary> {
  // return the parsed env object
  return dotenv.parse(await readFile(envPath));
}

export async function createNpmRegistryOptions(packagePath: string, options: any): Promise<any> {
  const {
    npmRcFilePath,
    envFilePath,
    userConfigPath,
    hasNpmRcFile,
    hasEnvFile
  } = options;

  // load the npm config
  const npmCliConfig = new NpmCliConfig({
    shorthands: {},
    definitions: {},
    npmPath: packagePath,
    // use the npmrc path to make npm cli parse the npmrc file
    // otherwise defaults to the package path
    cwd: hasNpmRcFile ? npmRcFilePath : packagePath,
    // ensures user npmrc is parsed by npm
    argv: ['', '', `--userconfig=${userConfigPath}`],
    // pass through .env data
    env: hasEnvFile
      ? await getDotEnv(envFilePath)
      : {}
  });

  await npmCliConfig.load();

  // flatten all the options
  return npmCliConfig.list.reduce(
    (memo, list) => ({ ...memo, ...list }),
    { cwd: packagePath }
  );
}