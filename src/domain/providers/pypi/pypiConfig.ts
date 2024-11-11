import { ICachingOptions } from '#domain/caching';
import { IHttpOptions } from '#domain/clients';
import { IFrozenOptions } from '#domain/configuration';
import { IProviderConfig, TProviderFileMatcher } from '#domain/providers';
import { PypiFeatures } from '#domain/providers/pypi';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const ctorParam = nameOf<PypiConfig>();

export class PypiConfig implements IProviderConfig {

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
    return this.config.get(PypiFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(PypiFeatures.DependencyProperties);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(PypiFeatures.PrereleaseTagFilter);
  }

  get apiUrl(): string {
    return this.config.get(PypiFeatures.ApiUrl);
  }

  get onSaveChangesTask(): string {
    return this.config.get(PypiFeatures.OnSaveChangesTask);
  }

}