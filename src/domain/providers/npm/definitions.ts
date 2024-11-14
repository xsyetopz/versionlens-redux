import { CachingOptions } from '#domain/caching';
import { HttpOptions, IJsonHttpClient, TClientResponse } from '#domain/clients';
import {
  GitHubClient,
  GitHubOptions,
  NpaSpec,
  NpmConfig,
  NpmPackageClient,
  NpmRegistryClient
} from '#domain/providers/npm';
import { KeyDictionary } from '#domain/utils';

export enum GitHubFeatures {
  AccessToken = 'accessToken',
}

export enum NpmFeatures {
  Caching = 'npm.caching',
  Http = 'npm.http',
  Github = 'npm.github',
  DependencyProperties = 'npm.dependencyProperties',
  FilePatterns = 'npm.files',
  OnSaveChangesTask = 'npm.onSaveChanges',
  PrereleaseTagFilter = 'npm.prereleaseTagFilter',
}

export interface INpmServices {
  npmCachingOpts: CachingOptions;
  npmHttpOpts: HttpOptions;
  npmGitHubOpts: GitHubOptions;
  npmConfig: NpmConfig;
  githubJsonClient: IJsonHttpClient;
  githubClient: GitHubClient;
  npmRegistryClient: NpmRegistryClient;
  npmClient: NpmPackageClient;
}

export interface INpmRegistry {
  pickRegistry: (spec: NpaSpec, opts: any) => string;
  json: (url: string, opts: any) => Promise<any>;
}

export type TNpmCliConfigParams = {
  npmRcFilePath: string,
  envFilePath: string,
  userConfigPath: string,
  hasNpmRcFile: boolean,
  hasEnvFile: boolean
}

export type TNpmClientData = {
  projectPath: string;
  userConfigPath: string;
  npmRcFilePath: string;
  envFilePath: string;
}

export type TNpmRegistryData = {
  name: string;
  versions: KeyDictionary<any>;
  "dist-tags": KeyDictionary<string>;
}

export type TNpmRegistryClientResponse = TClientResponse<number, TNpmRegistryData>