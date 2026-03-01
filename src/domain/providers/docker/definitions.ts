import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type {
  DockerConfig,
  DockerHubClient,
  DockerSuggestionResolver,
  MicrosoftDockerClient
} from '#domain/providers/docker';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for Docker configuration.
 */
export enum DockerFeatures {
  Caching = 'docker.caching',
  Http = 'docker.http',
  FilePatterns = 'docker.files',
  OnSaveChangesTask = 'docker.onSaveChanges',
  // PrereleaseTagFilter = 'docker.prereleaseTagFilter'
}

/**
 * Defines the services provided by the Docker provider.
 */
export interface IDockerServices {
  /**
   * Caching options for Docker.
   */
  dockerCachingOpts: CachingOptions
  /**
   * HTTP options for Docker.
   */
  dockerHttpOpts: HttpOptions
  /**
   * Configuration for Docker.
   */
  dockerConfig: DockerConfig
  /**
   * Client for Docker Hub.
   */
  dockerHubClient: DockerHubClient
  /**
   * Client for Microsoft Container Registry.
   */
  microsoftDockerClient: MicrosoftDockerClient
  /**
   * Resolver for Docker suggestions.
   */
  dockerSuggestionResolver: DockerSuggestionResolver
}

/**
 * Service name constant for Docker services.
 */
export const DockerService = nameOf<IDockerServices>()

/**
 * Represents a Docker repository tag and its digest.
 */
export type DockerRepository = {
  /**
   * The tag name.
   */
  name: string
  /**
   * The image digest.
   */
  digest: string
}

/**
 * Represents the JSON response for Docker repository tags.
 */
export type DockerClientResponse = JsonClientResponse<DockerRepository[]>

/**
 * Maps Docker tags to digests and vice versa.
 */
export type DockerDigestMapper = {
  /**
   * Maps tag name to digest.
   */
  tagMap: Record<string, string>
  /**
   * Maps digest to an array of tag names.
   */
  digestMap: Record<string, string[]>
}

/**
 * Maps Docker tags to semver versions and releases.
 */
export type DockerVersionMapper = {
  /**
   * Maps semver version to an array of tag names.
   */
  versionMap: Record<string, string[]>
  /**
   * Maps tag name to semver version.
   */
  tagMap: Record<string, string>
  /**
   * List of identified release versions.
   */
  releases: string[]
  /**
   * The identified 'latest' version.
   */
  latest?: string
}

/**
 * Represents a parsed Docker version and its original tag.
 */
export type DockerVersion = {
  /**
   * The parsed version string.
   */
  version: string
  /**
   * The original tag string.
   */
  tag: string
}