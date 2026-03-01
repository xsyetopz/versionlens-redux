import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { DockerFeatures } from '#domain/providers/docker';
import { nameOf } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

const def = nameOf<DockerConfig>();

/**
 * Configuration for the Docker package provider.
 */
export class DockerConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the DockerConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for Docker.
   * @param http The HTTP options for Docker.
   */
  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull(def.config, config);
    throwUndefinedOrNull(def.caching, caching);
    throwUndefinedOrNull(def.http, http);
  }

  /**
   * The file languages supported by this provider.
   */
  readonly fileLanguage = ['dockerfile', 'dockercompose', 'yaml'];

  /**
   * Gets the file patterns used to identify Docker files.
   */
  get filePatterns(): string {
    return this.config.get(DockerFeatures.FilePatterns)!;
  }

  /**
   * Gets the task to run when a Docker file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(DockerFeatures.OnSaveChangesTask) ?? null;
  }

}