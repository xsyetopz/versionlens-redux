import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { GoConfig, GoHttpClient, GoSuggestionResolver } from '#domain/providers/golang';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for Go configuration.
 */
export enum GoFeatures {
  Caching = 'golang.caching',
  Http = 'golang.http',
  ApiUrl = 'golang.apiUrl',
  FilePatterns = 'golang.files',
  OnSaveChangesTask = 'golang.onSaveChanges',
  PrereleaseTagFilter = 'golang.prereleaseTagFilter',
}

/**
 * Defines the services provided by the Go provider.
 */
export interface IGoService {
  /**
   * Caching options for Go.
   */
  goCachingOpts: CachingOptions;
  /**
   * HTTP options for Go.
   */
  goHttpOpts: HttpOptions;
  /**
   * Configuration for Go.
   */
  goConfig: GoConfig;
  /**
   * HTTP client for fetching from Go proxy.
   */
  goHttpClient: GoHttpClient;
  /**
   * Resolver for Go suggestions.
   */
  goSuggestionResolver: GoSuggestionResolver;
}

/**
 * Service name constant for Go services.
 */
export const GoService = nameOf<IGoService>()

/**
 * Represents the raw version list from the Go proxy API.
 */
export type GoApiClientResult = {
  /**
   * The array of version strings.
   */
  versions: string[]
}

/**
 * Represents the JSON response for Go package versions.
 */
export type GoApiClientResponse = JsonClientResponse<GoApiClientResult>