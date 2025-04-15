import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { type NugetOptions, DotNetFeatures } from '#domain/providers/dotnet';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<DotNetConfig>();

export class DotNetConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions,
    nugetOptions: NugetOptions,
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
    throwUndefinedOrNull(def.nuget, nugetOptions);

    this.nuget = nugetOptions;
  }

  nuget: NugetOptions;

  readonly fileLanguage = 'xml';

  get filePatterns(): string {
    return this.config.get(DotNetFeatures.FilePatterns);
  }

  get fileExcludePatterns(): string[] { return ['**/obj/**', '**/bin/**']; }

  get dependencyProperties(): Array<string> {
    return this.config.get(DotNetFeatures.DependencyProperties);
  }

  get fallbackNugetSource(): string {
    return 'https://api.nuget.org/v3/index.json';
  }

  get onSaveChangesTask(): string | null {
    return this.config.get(DotNetFeatures.OnSaveChangesTask) ?? null;
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(DotNetFeatures.PrereleaseTagFilter);
  }

}