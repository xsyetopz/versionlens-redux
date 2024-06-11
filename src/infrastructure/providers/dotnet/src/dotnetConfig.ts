import { throwUndefinedOrNull } from '@esm-test/guards';
import { ICachingOptions } from '#domain/caching';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from 'domain/providers';
import { nameOf } from 'domain/utils';
import { DotNetContributions } from './definitions/eDotNetContributions';
import { INugetOptions } from "./definitions/iNugetOptions";

const ctorParam = nameOf<DotNetConfig>();

export class DotNetConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: ICachingOptions,
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

  get fileMatcher(): TProviderFileMatcher {
    return {
      language: 'xml',
      scheme: 'file',
      pattern: this.filePatterns,
      exclude: '**/{obj,bin}/**'
    };
  }

  get filePatterns(): string {
    return this.config.get(DotNetContributions.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(DotNetContributions.DependencyProperties);
  }

  get fallbackNugetSource(): string {
    return 'https://api.nuget.org/v3/index.json';
  }

  get onSaveChangesTask(): string {
    return this.config.get(DotNetContributions.OnSaveChangesTask);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(DotNetContributions.PrereleaseTagFilter);
  }

}