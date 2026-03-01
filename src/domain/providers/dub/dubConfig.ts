import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { DubFeatures } from '#domain/providers/dub';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the Dub package provider.
 */
export class DubConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the DubConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for Dub.
   * @param http The HTTP options for Dub.
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
   * The file languages supported by this provider.
   */
  readonly fileLanguage = ['json', 'jsonc'];

  /**
   * Gets the file patterns used to identify Dub files.
   */
  get filePatterns(): string {
    return this.config.get(DubFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in Dub files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(DubFeatures.DependencyProperties, []);
  }

  /**
   * Gets the base API URL for the Dub registry.
   */
  get apiUrl(): string {
    return ensureEndSlash(this.config.get(DubFeatures.ApiUrl, ''));
  }

  /**
   * Gets the task to run when a Dub file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(DubFeatures.OnSaveChangesTask, null);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(DubFeatures.prereleaseTagFilter, []);
  }

}