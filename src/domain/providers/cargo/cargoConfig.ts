import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { CargoFeatures } from '#domain/providers/cargo';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the Cargo package provider.
 */
export class CargoConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the CargoConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for Cargo.
   * @param http The HTTP options for Cargo.
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
  readonly fileLanguage = 'toml';

  /**
   * Gets the file patterns used to identify Cargo files.
   */
  get filePatterns(): string {
    return this.config.get(CargoFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in Cargo files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(CargoFeatures.DependencyProperties, []);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(CargoFeatures.PrereleaseTagFilter, []);
  }

  /**
   * Gets the base API URL for the Crates registry.
   */
  get apiUrl(): string {
    return ensureEndSlash(this.config.get(CargoFeatures.ApiUrl, ''));
  }

  /**
   * Gets the task to run when a Cargo file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(CargoFeatures.OnSaveChangesTask, null);
  }

}