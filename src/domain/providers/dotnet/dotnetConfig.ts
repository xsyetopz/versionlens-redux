import type { CachingOptions } from '#domain/caching';
import type { IHttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { FileMatcher, IProviderConfig } from '#domain/providers';
import { type INugetOptions, DotNetFeatures } from '#domain/providers/dotnet';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<DotNetConfig>();

export class DotNetConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: IHttpOptions,
    nugetOptions: INugetOptions,
  ) {
    throwUndefinedOrNull(ctorParam.config, config);
    throwUndefinedOrNull(ctorParam.caching, caching);
    throwUndefinedOrNull(ctorParam.http, http);
    throwUndefinedOrNull(ctorParam.nuget, nugetOptions);

    this.nuget = nugetOptions;
  }

  nuget: INugetOptions;

  get fileMatcher(): FileMatcher {
    return {
      language: 'xml',
      scheme: 'file',
      pattern: this.filePatterns,
      exclude: ['**/obj/**', '**/bin/**']
    };
  }

  get filePatterns(): string {
    return this.config.get(DotNetFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(DotNetFeatures.DependencyProperties);
  }

  get fallbackNugetSource(): string {
    return 'https://api.nuget.org/v3/index.json';
  }

  get onSaveChangesTask(): string {
    return this.config.get(DotNetFeatures.OnSaveChangesTask);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(DotNetFeatures.PrereleaseTagFilter);
  }

}