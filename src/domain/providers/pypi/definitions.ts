import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { PypiConfig, PypiHttpClient, PypiSuggestionResolver } from '#domain/providers/pypi';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for PyPi configuration.
 */
export enum PypiFeatures {
  Caching = 'pypi.caching',
  Http = 'pypi.http',
  DependencyProperties = 'pypi.dependencyProperties',
  ApiUrl = 'pypi.apiUrl',
  FilePatterns = 'pypi.files',
  OnSaveChangesTask = 'pypi.onSaveChanges',
  PrereleaseTagFilter = 'pypi.prereleaseTagFilter',
}

/**
 * Defines the services provided by the PyPi provider.
 */
export interface IPypiServices {
  /**
   * Caching options for PyPi.
   */
  pypiCachingOpts: CachingOptions;
  /**
   * HTTP options for PyPi.
   */
  pypiHttpOpts: HttpOptions;
  /**
   * Configuration for PyPi.
   */
  pypiConfig: PypiConfig;
  /**
   * HTTP client for fetching from PyPi.
   */
  pypiHttpClient: PypiHttpClient;
  /**
   * Resolver for PyPi suggestions.
   */
  pypiSuggestionResolver: PypiSuggestionResolver;
}

/**
 * Service name constant for PyPi services.
 */
export const PypiService = nameOf<IPypiServices>()

/**
 * Represents the JSON response for PyPi package versions.
 */
export type PypiHttpClientResponse = JsonClientResponse<string[]>