import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { CargoConfig, CargoSuggestionResolver, CratesClient } from '#domain/providers/cargo';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for Cargo configuration.
 */
export enum CargoFeatures {
  Caching = 'cargo.caching',
  Http = 'cargo.http',
  DependencyProperties = 'cargo.dependencyProperties',
  ApiUrl = 'cargo.apiUrl',
  FilePatterns = 'cargo.files',
  OnSaveChangesTask = 'cargo.onSaveChanges',
  PrereleaseTagFilter = 'cargo.prereleaseTagFilter',
}

/**
 * Defines the services provided by the Cargo provider.
 */
export interface ICargoServices {
  /**
   * Caching options for Cargo.
   */
  cargoCachingOpts: CachingOptions
  /**
   * HTTP options for Cargo.
   */
  cargoHttpOpts: HttpOptions
  /**
   * Configuration for Cargo.
   */
  cargoConfig: CargoConfig
  /**
   * Client for the Crates registry.
   */
  cratesClient: CratesClient
  /**
   * Resolver for Cargo suggestions.
   */
  cargoSuggestionResolver: CargoSuggestionResolver
}

/**
 * Service name constant for Cargo services.
 */
export const CargoService = nameOf<ICargoServices>()

/**
 * Represents a single version entry for a Crates package.
 */
export type CratesPackageVersionEntry = {
  /**
   * The version number.
   */
  num: string,
  /**
   * Whether the version is yanked.
   */
  yanked: boolean
}

/**
 * Represents the collection of versions for a Crates package.
 */
export type CratesPackageVersionsResult = {
  /**
   * The array of version entries.
   */
  versions: CratesPackageVersionEntry[]
}

/**
 * Represents the JSON response for Crates package versions.
 */
export type CratesPackageVersionsResponse = JsonClientResponse<CratesPackageVersionsResult>