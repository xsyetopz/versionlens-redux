import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { DockerFeatures } from '#domain/providers/docker';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<DockerConfig>();

export class DockerConfig implements IProviderConfig {

  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
  }

  readonly fileLanguage = ['dockerfile', 'dockercompose', 'yaml'];

  get filePatterns(): string {
    return this.config.get(DockerFeatures.FilePatterns)!;
  }

  get onSaveChangesTask(): string | null {
    return this.config.get(DockerFeatures.OnSaveChangesTask) ?? null;
  }

}