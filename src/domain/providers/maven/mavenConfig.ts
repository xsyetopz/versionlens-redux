import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { MavenFeatures } from '#domain/providers/maven';
import { ensureEndSlash, nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<MavenConfig>();

export class MavenConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
  }

  readonly fileLanguage = 'xml';

  get filePatterns(): string {
    return this.config.get(MavenFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(MavenFeatures.DependencyProperties);
  }

  get apiUrl(): string {
    return ensureEndSlash(this.config.get(MavenFeatures.ApiUrl));
  }

  get onSaveChangesTask(): string | null {
    return this.config.get(MavenFeatures.OnSaveChangesTask) ?? null;
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(MavenFeatures.prereleaseTagFilter);
  }

}