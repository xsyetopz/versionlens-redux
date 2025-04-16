import { CachingOptions } from '#domain/caching';
import { HttpOptions, JsonHttpClient } from '#domain/clients';
import { DockerClient, DockerConfig, DockerHubClient } from '#domain/providers/docker';

export enum DockerFeatures {
  Caching = 'docker.caching',
  Http = 'docker.http',
  DependencyProperties = 'docker.dependencyProperties',
  ApiUrl = 'docker.apiUrl',
  FilePatterns = 'docker.files',
  OnSaveChangesTask = 'docker.onSaveChanges',
  PrereleaseTagFilter = 'docker.prereleaseTagFilter'
}

export interface IDockerServices {
  dockerCachingOpts: CachingOptions;
  dockerHttpOpts: HttpOptions;
  dockerConfig: DockerConfig;
  dockerJsonClient: JsonHttpClient;
  dockerHubClient: DockerHubClient;
  dockerClient: DockerClient;
}

export type DockerApiResponse<T> = {
  count: number,
  next: string,
  name: string,
  results: T[]
}

export type DockerApiTagResult = {
  name: string
  tag_status: 'active' | 'inactive'
  digest: string
}

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