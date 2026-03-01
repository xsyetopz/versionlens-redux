import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { DenoSuggestionResolver, DenoConfig, JsrClient } from "#domain/providers/deno";
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for Deno configuration.
 */
export enum DenoFeatures {
  Caching = 'deno.caching',
  Http = 'deno.http',
  DependencyProperties = 'deno.dependencyProperties',
  FilePatterns = 'deno.files',
  OnSaveChangesTask = 'deno.onSaveChanges',
  PrereleaseTagFilter = 'deno.prereleaseTagFilter',
}

/**
 * Defines the services provided by the Deno provider.
 */
export interface IDenoServices {
  /**
   * Caching options for Deno.
   */
  denoCachingOpts: CachingOptions
  /**
   * HTTP options for Deno.
   */
  denoHttpOpts: HttpOptions
  /**
   * Configuration for Deno.
   */
  denoConfig: DenoConfig
  /**
   * Client for the JSR registry.
   */
  jsrClient: JsrClient
  /**
   * Resolver for Deno suggestions.
   */
  denoClient: DenoSuggestionResolver
}

/**
 * Service name constant for Deno services.
 */
export const DenoService = nameOf<IDenoServices>()

/**
 * Represents the raw package data returned by the JSR API.
 */
export type JsrApiResult = {
  /**
   * The latest version number.
   */
  latest: string
  /**
   * A dictionary of versions and their yanked status.
   */
  versions: {
    [version: string]: {
      /**
       * Whether the version is yanked.
       */
      yanked?: boolean
    }
  }
}

/**
 * Represents the JSON response from the JSR API.
 */
export type JsrApiResponse = JsonClientResponse<JsrApiResult>

/**
 * Represents the processed version list response for the JSR client.
 */
export type JsrClientResponse = JsonClientResponse<string[]>