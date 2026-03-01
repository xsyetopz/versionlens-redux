import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { PubFeatures } from '#domain/providers/pub';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the Pub package provider.
 */
export class PubConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the PubConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for Pub.
   * @param http The HTTP options for Pub.
   */
  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('caching', caching);
    throwUndefinedOrNull('http', http);
  }

  /**
   * The file language supported by this provider.
   */
  readonly fileLanguage = 'yaml';

  /**
   * Gets the file patterns used to identify Pub files.
   */
  get filePatterns(): string {
    return this.config.get(PubFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in Pub files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(PubFeatures.DependencyProperties, []);
  }

  /**
   * Gets the base API URL for the Pub registry.
   */
  get apiUrl(): string {
    return ensureEndSlash(this.config.get(PubFeatures.ApiUrl, ''));
  }

  /**
   * Gets the task to run when a Pub file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(PubFeatures.OnSaveChangesTask, null);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(PubFeatures.PrereleaseTagFilter, []);
  }

}