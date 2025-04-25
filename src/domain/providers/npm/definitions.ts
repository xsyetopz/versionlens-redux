import type { CachingOptions } from '#domain/caching';
import type { ClientResponse, HttpOptions } from '#domain/clients';
import type {
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

export enum NpaTypes {
  Git = 'git',
  Remote = 'remote',
  File = 'file',
  Directory = 'directory',
  Tag = 'tag',
  Version = 'version',
  Range = 'range',
  Alias = 'alias',
}

export type NpaSpec = {
  type: NpaTypes;
  registry: boolean,
  name: string,
  scope: string,
  escapedName: string,
  rawSpec: any,
  saveSpec: any,
  fetchSpec: any,
  subSpec: any,
  gitRange: any,
  gitCommittish: string,
  hosted: any,
  raw: string,
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

export type NpmClientData = {
  [url: string]: any,
  ca?: string | Array<string>
  cert?: string
  proxy?: string | null
  httpsProxy?: string | null
  registry: string
  strictSSL: boolean
}

export type NpmRegistryData = {
  name: string;
  versions: KeyDictionary<any>;
  "dist-tags": KeyDictionary<string>;
}

export type NpmRegistryClientResponse = ClientResponse<number, NpmRegistryData>