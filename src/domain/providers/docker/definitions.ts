import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type {
  DockerConfig,
  DockerHubClient,
  DockerSuggestionResolver,
  MicrosoftDockerClient
} from '#domain/providers/docker';
import { nameOf } from '#domain/utils';

export enum DockerFeatures {
  Caching = 'docker.caching',
  Http = 'docker.http',
  FilePatterns = 'docker.files',
  OnSaveChangesTask = 'docker.onSaveChanges',
  // PrereleaseTagFilter = 'docker.prereleaseTagFilter'
}

export interface IDockerServices {
  dockerCachingOpts: CachingOptions
  dockerHttpOpts: HttpOptions
  dockerConfig: DockerConfig
  dockerHubClient: DockerHubClient
  microsoftDockerClient: MicrosoftDockerClient
  dockerSuggestionResolver: DockerSuggestionResolver
}

export const DockerService = nameOf<IDockerServices>()

export type DockerRepository = {
  name: string
  digest: string
}

export type DockerClientResponse = JsonClientResponse<DockerRepository[]>

export type DockerDigestMapper = {
  tagMap: Record<string, string>
  digestMap: Record<string, string[]>
}

export type DockerVersionMapper = {
  versionMap: Record<string, string[]>
  tagMap: Record<string, string>
  releases: string[]
  latest?: string
}

export type DockerVersion = {
  version: string
  tag: string
}