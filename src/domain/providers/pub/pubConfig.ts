import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { PubFeatures } from '#domain/providers/pub';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<PubConfig>();

export class PubConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
  }

  readonly fileLanguage = 'yaml';

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