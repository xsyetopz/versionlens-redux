import { ICachingOptions } from '#domain/caching';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from '#domain/providers';
import { nameOf } from '#domain/utils';
import { GoFeatures } from '#providers/golang';
import { throwUndefinedOrNull } from '@esm-test/guards';

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
    return this.config.get(GoFeatures.FilePatterns);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(GoFeatures.PrereleaseTagFilter);
  }

  get apiUrl(): string {
    return this.config.get(GoFeatures.ApiUrl);
  }

  get onSaveChangesTask(): string {
    return this.config.get(GoFeatures.OnSaveChangesTask);
  }

}