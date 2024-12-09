import type { CachingOptions } from '#domain/caching';
import type { IHttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { FileMatcher, IProviderConfig } from '#domain/providers';
import { GoFeatures } from '#domain/providers/golang';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<GoConfig>();

export class GoConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: IHttpOptions
  ) {
    throwUndefinedOrNull(ctorParam.config, config);
    throwUndefinedOrNull(ctorParam.caching, caching);
    throwUndefinedOrNull(ctorParam.http, http);
  }

  get fileMatcher(): FileMatcher {
    return {
      language: 'go.mod',
      scheme: 'file',
      pattern: this.filePatterns
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