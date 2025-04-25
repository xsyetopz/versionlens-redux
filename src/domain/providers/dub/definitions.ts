import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
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
  dubHttpOpts: HttpOptions;
  dubConfig: DubConfig;
  dubClient: DubClient;
}