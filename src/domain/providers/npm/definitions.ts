import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, TClientResponse } from '#domain/clients';
import type {
  NpaSpec,
  NpmConfig,
  NpmGitHubClient,
  NpmPackageClient,
  NpmRegistryClient
} from '#domain/providers/npm';
import { type KeyDictionary, nameOf } from '#domain/utils';

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
  npmConfig: NpmConfig;
  npmGithubClient: NpmGitHubClient;
  npmRegistryClient: NpmRegistryClient;
  npmClient: NpmPackageClient;
}

export const NpmService = nameOf<INpmServices>()

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
  [url: string]: any,
  ca?: string | Array<string>
  cert?: string
  proxy?: string | null
  httpsProxy?: string | null
  registry: string
  strictSSL: boolean
}

export type TNpmRegistryData = {
  name: string;
  versions: KeyDictionary<any>;
  "dist-tags": KeyDictionary<string>;
}

export type TNpmRegistryClientResponse = TClientResponse<number, TNpmRegistryData>