import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { CargoFeatures } from '#domain/providers/cargo';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<CargoConfig>();

export class CargoConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
  }

  readonly fileLanguage = 'toml';

  get filePatterns(): string {
    return this.config.get(CargoFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(CargoFeatures.DependencyProperties);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(CargoFeatures.PrereleaseTagFilter);
  }

  get apiUrl(): string {
    return ensureEndSlash(this.config.get(CargoFeatures.ApiUrl));
  }

  get onSaveChangesTask(): string {
    return this.config.get(CargoFeatures.OnSaveChangesTask);
  }

}