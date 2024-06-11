import { throwUndefinedOrNull } from '@esm-test/guards';
import { ICachingOptions } from '#domain/caching';
import { UrlUtils } from '#domain/clients';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from 'domain/providers';
import { nameOf } from 'domain/utils';
import { PubContributions } from './definitions/ePubContributions';

const ctorParam = nameOf<PubConfig>();

export class PubConfig implements IProviderConfig {

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
      language: 'yaml',
      scheme: 'file',
      pattern: this.filePatterns,
      exclude: ''
    };
  }

  get filePatterns(): string {
    return this.config.get(PubContributions.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(PubContributions.DependencyProperties);
  }

  get apiUrl(): string {
    return UrlUtils.ensureEndSlash(this.config.get(PubContributions.ApiUrl));
  }

  get onSaveChangesTask(): string {
    return this.config.get(PubContributions.OnSaveChangesTask);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(PubContributions.PrereleaseTagFilter);
  }

}