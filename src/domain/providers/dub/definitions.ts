import type { CachingOptions } from '#domain/caching';
import type { IHttpOptions, IJsonHttpClient } from '#domain/clients';
import type { DubClient, DubConfig } from '#domain/providers/dub';

export enum DubFeatures {
  Caching = 'dub.caching',
  Http = 'dub.http',
  DependencyProperties = 'dub.dependencyProperties',
  ApiUrl = 'dub.apiUrl',
  FilePatterns = 'dub.files',
  OnSaveChangesTask = 'dub.onSaveChanges',
  prereleaseTagFilter = 'dub.prereleaseTagFilter',
}

export interface IDubServices {
  dubCachingOpts: CachingOptions;
  dubHttpOpts: IHttpOptions;
  dubConfig: DubConfig;
  dubJsonClient: IJsonHttpClient;
  dubClient: DubClient;
}