import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { PypiFeatures } from '#domain/providers/pypi';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<PypiConfig>();

export class PypiConfig implements IProviderConfig {

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

  get onSaveChangesTask(): string | null {
    return this.config.get(PypiFeatures.OnSaveChangesTask) ?? null;
  }

}