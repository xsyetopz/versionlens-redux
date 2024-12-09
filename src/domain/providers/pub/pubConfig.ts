import type { CachingOptions } from '#domain/caching';
import type { IHttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { FileMatcher, IProviderConfig } from '#domain/providers';
import { PubFeatures } from '#domain/providers/pub';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<PubConfig>();

export class PubConfig implements IProviderConfig {

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
      language: 'yaml',
      scheme: 'file',
      pattern: this.filePatterns
    };
  }

  get filePatterns(): string {
    return this.config.get(PubFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(PubFeatures.DependencyProperties);
  }

  get apiUrl(): string {
    return ensureEndSlash(this.config.get(PubFeatures.ApiUrl));
  }

  get onSaveChangesTask(): string {
    return this.config.get(PubFeatures.OnSaveChangesTask);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(PubFeatures.PrereleaseTagFilter);
  }

}