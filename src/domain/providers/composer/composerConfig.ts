import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { ComposerFeatures } from '#domain/providers/composer';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<ComposerConfig>();

export class ComposerConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
  }

  readonly fileLanguage = 'json';

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