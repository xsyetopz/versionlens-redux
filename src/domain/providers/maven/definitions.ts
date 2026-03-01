import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { MavenConfig, MavenHttpClient, MavenSuggestionResolver, MvnCli } from '#domain/providers/maven';
import { type RegistryProtocols, nameOf } from '#domain/utils';

/**
 * Feature keys used for Maven configuration.
 */
export enum MavenFeatures {
  Caching = 'maven.caching',
  Http = 'maven.http',
  DependencyProperties = 'maven.dependencyProperties',
  ApiUrl = 'maven.apiUrl',
  FilePatterns = 'maven.files',
  OnSaveChangesTask = 'maven.onSaveChanges',
  prereleaseTagFilter = 'maven.prereleaseTagFilter',
}

/**
 * Defines the services provided by the Maven provider.
 */
export interface IMavenServices {
  /**
   * Caching options for Maven.
   */
  mavenCachingOpts: CachingOptions;
  /**
   * HTTP options for Maven.
   */
  mavenHttpOpts: HttpOptions;
  /**
   * Configuration for Maven.
   */
  mavenConfig: MavenConfig;
  /**
   * Client for interacting with the Maven CLI.
   */
  mvnCli: MvnCli;
  /**
   * HTTP client for fetching from Maven repositories.
   */
  mavenHttpClient: MavenHttpClient;
  /**
   * Resolver for Maven suggestions.
   */
  mavenSuggestionResolver: MavenSuggestionResolver;
}

/**
 * Service name constant for Maven services.
 */
export const MavenService = nameOf<IMavenServices>()

/**
 * Represents a Maven repository source.
 */
export type MavenRepository = {
  /**
   * The repository URL.
   */
  url: string,
  /**
   * The registry protocol (e.g., https).
   */
  protocol: RegistryProtocols
}

/**
 * Custom data passed to the Maven client.
 */
export type MavenClientData = {
  /**
   * The list of configured Maven repositories.
   */
  repositories: Array<MavenRepository>
}

/**
 * Represents the JSON response for Maven package versions.
 */
export type MavenApiResponse = JsonClientResponse<string[]>