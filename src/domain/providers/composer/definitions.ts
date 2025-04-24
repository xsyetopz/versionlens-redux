import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, IJsonHttpClient, JsonClientResponse } from '#domain/clients';
import type { ComposerClient, ComposerConfig, PackagistClient } from "#domain/providers/composer";
import { nameOf } from '#domain/utils';

export enum ComposerFeatures {
  Caching = 'composer.caching',
  Http = 'composer.http',
  DependencyProperties = 'composer.dependencyProperties',
  ApiUrl = 'composer.apiUrl',
  FilePatterns = 'composer.files',
  OnSaveChangesTask = 'composer.onSaveChanges',
  PrereleaseTagFilter = 'composer.prereleaseTagFilter',
}

export interface IPackagistApiItem {
  version: string;
}

export interface IComposerService {
  composerCachingOpts: CachingOptions;
  composerHttpOpts: HttpOptions;
  composerConfig: ComposerConfig;
  composerJsonClient: IJsonHttpClient;
  packagistClient: PackagistClient;
  composerClient: ComposerClient;
}

export const ComposerService = nameOf<IComposerService>()

export type PackagistVersionEntry = {
  version: string
}

export type PackagistPackagesResult = {
  packages: {
    [key: string]: PackagistVersionEntry[]
  }
}

export type PackagistPackagesResponse = JsonClientResponse<PackagistPackagesResult>