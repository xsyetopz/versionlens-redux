import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { GoFeatures } from '#domain/providers/golang';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<GoConfig>();

export class GoConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
  }

  readonly fileLanguage = 'go.mod';

  get filePatterns(): string {
    return this.config.get(GoFeatures.FilePatterns);
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(GoFeatures.PrereleaseTagFilter);
  }

  get apiUrl(): string {
    return this.config.get(GoFeatures.ApiUrl);
  }

  get onSaveChangesTask(): string {
    return this.config.get(GoFeatures.OnSaveChangesTask);
  }

}