import type { CachingOptions } from '#domain/caching';
import type { IHttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { FileMatcher, IProviderConfig } from '#domain/providers';
import { CargoFeatures } from '#domain/providers/cargo';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<CargoConfig>();

export class CargoConfig implements IProviderConfig {

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
      language: 'toml',
      scheme: 'file',
      pattern: this.filePatterns
    };
  }

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