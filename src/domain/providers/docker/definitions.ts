import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type {
  DockerConfig,
  DockerHubClient,
  DockerSuggestionResolver,
  MicrosoftHubClient
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
  microsoftHubClient: MicrosoftHubClient
  dockerSuggestionResolver: DockerSuggestionResolver
}

export const DockerService = nameOf<IDockerServices>()

export type DockerHubRepository = {
  name: string
  tag_status: 'active' | 'inactive'
  digest: string
}

export type DockerHubListReposResult = {
  count: number
  next: string
  name: string
  results: DockerHubRepository[]
}

export type DockerHubListReposResponse = JsonClientResponse<DockerHubListReposResult>

export type DockerHubListClientResponse = JsonClientResponse<DockerHubRepository[]>

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