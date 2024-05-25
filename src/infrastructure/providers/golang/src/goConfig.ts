import { throwUndefinedOrNull } from '@esm-test/guards';
import { ICachingOptions } from 'domain/caching';
import { IFrozenOptions } from 'domain/configuration';
import { IHttpOptions } from 'domain/http';
import { IProviderConfig, TProviderFileMatcher } from 'domain/providers';
import { nameOf } from 'domain/utils';
import { GoContributions } from './definitions/eGoContributions';

const ctorParam = nameOf<GoConfig>();

export class GoConfig implements IProviderConfig {

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
      language: 'go.mod',
      scheme: 'file',
      pattern: this.filePatterns,
      exclude: ''
    };
  }

  get filePatterns(): string {
    return this.config.get(GoContributions.FilePatterns);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(GoContributions.PrereleaseTagFilter);
  }

  get apiUrl(): string {
    return this.config.get(GoContributions.ApiUrl);
  }

  get onSaveChangesTask(): string {
    return this.config.get(GoContributions.OnSaveChangesTask);
  }

}