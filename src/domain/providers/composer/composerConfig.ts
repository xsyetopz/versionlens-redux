import type { CachingOptions } from '#domain/caching';
import type { IHttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { FileMatcher, IProviderConfig } from '#domain/providers';
import { ComposerFeatures } from '#domain/providers/composer';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<ComposerConfig>();

export class ComposerConfig implements IProviderConfig {

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
      language: 'json',
      scheme: 'file',
      pattern: this.filePatterns
    };
  }

  get filePatterns(): string {
    return this.config.get(ComposerFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(ComposerFeatures.DependencyProperties);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(ComposerFeatures.PrereleaseTagFilter);
  }

  get apiUrl(): string {
    return ensureEndSlash(this.config.get(ComposerFeatures.ApiUrl));
  }

  get onSaveChangesTask(): string {
    return this.config.get(ComposerFeatures.OnSaveChangesTask);
  }

}