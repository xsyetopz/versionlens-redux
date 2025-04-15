import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { type GitHubOptions, NpmFeatures } from '#domain/providers/npm';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<NpmConfig>();

export class NpmConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions,
    readonly github: GitHubOptions,
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
    throwUndefinedOrNull(def.github, github);
  }

  readonly fileLanguage = 'json';

  get filePatterns(): string {
    return this.config.get(NpmFeatures.FilePatterns);
  }

  get dependencyProperties(): Array<string> {
    return this.config.get(NpmFeatures.DependencyProperties);
  }

  get onSaveChangesTask(): string | null {
    return this.config.get(NpmFeatures.OnSaveChangesTask) ?? null;
  }

  get prereleaseTagFilter(): Array<string> {
    return this.config.get(NpmFeatures.PrereleaseTagFilter);
  }

}