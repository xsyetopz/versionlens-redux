import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type {
  NpmConfig,
  NpmGitHubClient,
  NpmRegistryClient,
  NpmSuggestionResolver
} from '#domain/providers/npm';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for NPM configuration.
 */
export enum NpmFeatures {
  Caching = 'npm.caching',
  Http = 'npm.http',
  DependencyProperties = 'npm.dependencyProperties',
  FilePatterns = 'npm.files',
  OnSaveChangesTask = 'npm.onSaveChanges',
  PrereleaseTagFilter = 'npm.prereleaseTagFilter',
}

/**
 * Defines the services provided by the NPM provider.
 */
export interface INpmServices {
  /**
   * Caching options for NPM.
   */
  npmCachingOpts: CachingOptions
  /**
   * HTTP options for NPM.
   */
  npmHttpOpts: HttpOptions
  /**
   * Configuration for NPM.
   */
  npmConfig: NpmConfig
  /**
   * Client for GitHub NPM dependencies.
   */
  npmGithubClient: NpmGitHubClient
  /**
   * Client for NPM registries.
   */
  npmRegistryClient: NpmRegistryClient
  /**
   * Resolver for NPM suggestions.
   */
  npmSuggestionResolver: NpmSuggestionResolver
}

/**
 * Service name constant for NPM services.
 */
export const NpmService = nameOf<INpmServices>()

/**
 * Supported NPM package argument types.
 */
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

/**
 * Represents a parsed NPM package argument.
 */
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

/**
 * Represents the raw data returned by an NPM registry API.
 */
export type NpmRegistryApiResult = {
  'dist-tags': { [tag: string]: string }
  versions: { [version: string]: any }
}

/**
 * Interface for NPM registry interaction.
 */
export interface INpmRegistry {
  /**
   * Picks the appropriate registry for a given package spec.
   */
  pickRegistry: (spec: NpaSpec, opts: any) => string;
  /**
   * Fetches JSON data from an NPM registry URL.
   */
  json: (url: string, opts: any) => Promise<NpmRegistryApiResult>;
}

/**
 * Parameters for NPM CLI configuration resolution.
 */
export type TNpmCliConfigParams = {
  npmRcFilePath: string,
  envFilePath: string,
  userConfigPath: string,
  hasNpmRcFile: boolean,
  hasEnvFile: boolean
}

/**
 * Flattened NPM client configuration data.
 */
export type NpmClientData = {
  [url: string]: any,
  ca?: string | Array<string>
  cert?: string
  proxy?: string | null
  httpsProxy?: string | null
  registry: string
  strictSSL: boolean
}

/**
 * Processed NPM registry client result.
 */
export type NpmRegistryClientResult = {
  'dist-tags': { [tag: string]: string }
  versions: string[]
}

/**
 * Represents the JSON response for NPM registry queries.
 */
export type NpmRegistryClientResponse = JsonClientResponse<NpmRegistryClientResult>