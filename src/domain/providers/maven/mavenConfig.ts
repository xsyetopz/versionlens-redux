import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { MavenFeatures } from '#domain/providers/maven';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the Maven package provider.
 */
export class MavenConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the MavenConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for Maven.
   * @param http The HTTP options for Maven.
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
  readonly fileLanguage = 'xml';

  /**
   * Gets the file patterns used to identify Maven files.
   */
  get filePatterns(): string {
    return this.config.get(MavenFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in Maven files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(MavenFeatures.DependencyProperties, []);
  }

  /**
   * Gets the base API URL for the Maven Central registry.
   */
  get apiUrl(): string {
    return ensureEndSlash(this.config.get(MavenFeatures.ApiUrl, ''));
  }

  /**
   * Gets the task to run when a Maven file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(MavenFeatures.OnSaveChangesTask, null);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(MavenFeatures.prereleaseTagFilter, []);
  }

}