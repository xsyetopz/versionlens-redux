import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { PubConfig, PubJsonClient, PubSuggestionResolver } from '#domain/providers/pub';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for Pub configuration.
 */
export enum PubFeatures {
  Caching = 'pub.caching',
  Http = 'pub.http',
  DependencyProperties = 'pub.dependencyProperties',
  ApiUrl = 'pub.apiUrl',
  FilePatterns = 'pub.files',
  OnSaveChangesTask = 'pub.onSaveChanges',
  PrereleaseTagFilter = 'pub.prereleaseTagFilter',
}

/**
 * Defines the services provided by the Pub provider.
 */
export interface IPubServices {
  /**
   * Caching options for Pub.
   */
  pubCachingOpts: CachingOptions;
  /**
   * HTTP options for Pub.
   */
  pubHttpOpts: HttpOptions;
  /**
   * Configuration for Pub.
   */
  pubConfig: PubConfig;
  /**
   * Client for the Pub registry.
   */
  pubJsonClient: PubJsonClient;
  /**
   * Resolver for Pub suggestions.
   */
  pubSuggestionResolver: PubSuggestionResolver;
}

/**
 * Service name constant for Pub services.
 */
export const PubService = nameOf<IPubServices>()

/**
 * Represents a single version entry from the Pub API.
 */
export type PubApiVersionEntry = {
  /**
   * The version string.
   */
  version: string,
  /**
   * Whether the version is retracted.
   */
  retracted: boolean
}

/**
 * Represents the raw package data returned by the Pub API.
 */
export type PubApiPackageResult = {
  /**
   * The list of version entries.
   */
  versions: PubApiVersionEntry[]
}

/**
 * Represents the processed version list for the Pub client.
 */
export type PubJsonClientResult = {
  /**
   * The array of version strings.
   */
  versions: string[]
}

/**
 * Represents the JSON response for Pub package data.
 */
export type PubPackageResponse = JsonClientResponse<PubApiPackageResult>

/**
 * Represents the JSON response for the processed Pub version list.
 */
export type PubJsonClientResponse = JsonClientResponse<PubJsonClientResult>