import { throwUndefinedOrNull } from '@esm-test/guards';
import { ICachingOptions } from '#domain/caching';
import { UrlUtils } from '#domain/clients';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from '#domain/providers';
import { nameOf } from '#domain/utils';
import { DubContributions } from './definitions/eDubContributions';

const ctorParam = nameOf<DubConfig>();

export class DubConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: ICachingOptions,
    readonly http: IHttpOptions
  ) {
    throwUndefinedOrNull(ctorParam.config, config);
    throwUndefinedOrNull(ctorParam.caching, caching);
    throwUndefinedOrNull(ctorParam.http, http);
  }

  get fileMatcher(): TProviderFileMatcher {
    return {
      language: 'json',
      scheme: 'file',
      pattern: this.filePatterns,
      exclude: ''
    };
  }

  get filePatterns(): string {
    return this.config.get(DubContributions.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(DubContributions.DependencyProperties);
  }

  get apiUrl(): string {
    return UrlUtils.ensureEndSlash(this.config.get(DubContributions.ApiUrl));
  }

  get onSaveChangesTask(): string {
    return this.config.get(DubContributions.OnSaveChangesTask);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(DubContributions.prereleaseTagFilter);
  }

}