import { ICachingOptions } from '#domain/caching';
import { IFrozenOptions } from '#domain/configuration';
import { IHttpOptions } from '#domain/http';
import { IProviderConfig, TProviderFileMatcher } from '#domain/providers';
import { nameOf } from '#domain/utils';
import { PypiFeatures } from '#providers/pypi';
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