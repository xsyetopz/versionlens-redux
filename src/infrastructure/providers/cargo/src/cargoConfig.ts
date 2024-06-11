import { throwUndefinedOrNull } from '@esm-test/guards';
import { ICachingOptions } from '#domain/caching';
import { UrlUtils } from '#domain/clients';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from 'domain/providers';
import { nameOf } from 'domain/utils';
import { CargoContributions } from './definitions/eCargoContributions';

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
    return this.config.get(CargoContributions.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(CargoContributions.DependencyProperties);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(CargoContributions.PrereleaseTagFilter);
  }

  get apiUrl(): string {
    return UrlUtils.ensureEndSlash(this.config.get(CargoContributions.ApiUrl));
  }

  get onSaveChangesTask(): string {
    return this.config.get(CargoContributions.OnSaveChangesTask);
  }

}