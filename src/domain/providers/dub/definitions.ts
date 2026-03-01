import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { DubConfig, DubJsonClient, DubSuggestionResolver } from '#domain/providers/dub';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for Dub configuration.
 */
export enum DubFeatures {
  Caching = 'dub.caching',
  Http = 'dub.http',
  DependencyProperties = 'dub.dependencyProperties',
  ApiUrl = 'dub.apiUrl',
  FilePatterns = 'dub.files',
  OnSaveChangesTask = 'dub.onSaveChanges',
  prereleaseTagFilter = 'dub.prereleaseTagFilter',
}

/**
 * Defines the services provided by the Dub provider.
 */
export interface IDubServices {
  /**
   * Caching options for Dub.
   */
  dubCachingOpts: CachingOptions;
  /**
   * HTTP options for Dub.
   */
  dubHttpOpts: HttpOptions;
  /**
   * Configuration for Dub.
   */
  dubConfig: DubConfig;
  /**
   * Client for the Dub registry.
   */
  dubJsonClient: DubJsonClient;
  /**
   * Resolver for Dub suggestions.
   */
  dubSuggestionResolver: DubSuggestionResolver;
}

/**
 * Service name constant for Dub services.
 */
export const DubService = nameOf<IDubServices>()

/**
 * Represents the raw version entry from the Dub API.
 */
export type DubApiResult = {
  /**
   * The list of version objects.
   */
  versions: [{ version: string }]
}

/**
 * Represents the JSON response for Dub package versions.
 */
export type DubJsonClientResponse = JsonClientResponse<string[]>