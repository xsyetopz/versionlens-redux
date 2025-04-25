import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { PubClient, PubConfig, PubJsonClient } from '#domain/providers/pub';
import { nameOf } from '#domain/utils';

export enum PubFeatures {
  Caching = 'pub.caching',
  Http = 'pub.http',
  DependencyProperties = 'pub.dependencyProperties',
  ApiUrl = 'pub.apiUrl',
  FilePatterns = 'pub.files',
  OnSaveChangesTask = 'pub.onSaveChanges',
  PrereleaseTagFilter = 'pub.prereleaseTagFilter',
}

export interface IPubServices {
  pubCachingOpts: CachingOptions;
  pubHttpOpts: HttpOptions;
  pubConfig: PubConfig;
  pubJsonClient: PubJsonClient;
  pubClient: PubClient;
}

export const PubService = nameOf<IPubServices>()

export type PubApiVersionEntry = {
  version: string,
  retracted: boolean
}

export type PubApiPackageResult = {
  versions: PubApiVersionEntry[]
}

export type PubJsonClientResult = {
  versions: string[]
}

export type PubPackageResponse = JsonClientResponse<PubApiPackageResult>

export type PubJsonClientResponse = JsonClientResponse<PubJsonClientResult>