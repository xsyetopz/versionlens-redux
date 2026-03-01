import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { ComposerConfig, ComposerSuggestionResolver, PackagistClient } from "#domain/providers/composer";
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for Composer configuration.
 */
export enum ComposerFeatures {
  Caching = 'composer.caching',
  Http = 'composer.http',
  DependencyProperties = 'composer.dependencyProperties',
  ApiUrl = 'composer.apiUrl',
  FilePatterns = 'composer.files',
  OnSaveChangesTask = 'composer.onSaveChanges',
  PrereleaseTagFilter = 'composer.prereleaseTagFilter',
}

/**
 * Represents a single item in the Packagist API response.
 */
export interface IPackagistApiItem {
  /**
   * The version string.
   */
  version: string;
}

/**
 * Defines the services provided by the Composer provider.
 */
export interface IComposerService {
  /**
   * Caching options for Composer.
   */
  composerCachingOpts: CachingOptions;
  /**
   * HTTP options for Composer.
   */
  composerHttpOpts: HttpOptions;
  /**
   * Configuration for Composer.
   */
  composerConfig: ComposerConfig;
  /**
   * Client for the Packagist registry.
   */
  packagistClient: PackagistClient;
  /**
   * Resolver for Composer suggestions.
   */
  composerSuggestionResolver: ComposerSuggestionResolver;
}

/**
 * Service name constant for Composer services.
 */
export const ComposerService = nameOf<IComposerService>()

/**
 * Represents a single version entry for a Packagist package.
 */
export type PackagistVersionEntry = {
  /**
   * The version string.
   */
  version: string
}

/**
 * Represents the collection of packages and their versions from Packagist.
 */
export type PackagistPackagesResult = {
  /**
   * A dictionary of package names and their version entries.
   */
  packages: {
    [key: string]: PackagistVersionEntry[]
  }
}

/**
 * Represents the JSON response for Packagist package versions.
 */
export type PackagistPackagesResponse = JsonClientResponse<PackagistPackagesResult>