import { ICachingOptions } from '#domain/caching';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from '#domain/providers';
import { CargoFeatures } from '#domain/providers/cargo';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<CargoConfig>();

export class CargoConfig implements IProviderConfig {

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
      language: 'toml',
      scheme: 'file',
      pattern: this.filePatterns,
      exclude: ''
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