import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { ComposerFeatures } from '#domain/providers/composer';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the Composer package provider.
 */
export class ComposerConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the ComposerConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for Composer.
   * @param http The HTTP options for Composer.
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
   * Gets the file patterns used to identify Composer files.
   */
  get filePatterns(): string {
    return this.config.get(ComposerFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in Composer files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(ComposerFeatures.DependencyProperties, []);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(ComposerFeatures.PrereleaseTagFilter, []);
  }

  /**
   * Gets the base API URL for the Packagist registry.
   */
  get apiUrl(): string {
    return ensureEndSlash(this.config.get(ComposerFeatures.ApiUrl, ''));
  }

  /**
   * Gets the task to run when a Composer file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(ComposerFeatures.OnSaveChangesTask, null);
  }

}