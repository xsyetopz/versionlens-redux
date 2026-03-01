import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { NpmFeatures } from '#domain/providers/npm';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the NPM package provider.
 */
export class NpmConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the NpmConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for NPM.
   * @param http The HTTP options for NPM.
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
   * Gets the file patterns used to identify NPM files.
   */
  get filePatterns(): string {
    return this.config.get(NpmFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in NPM files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(NpmFeatures.DependencyProperties, []);
  }

  /**
   * Gets the task to run when an NPM file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(NpmFeatures.OnSaveChangesTask, null);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(NpmFeatures.PrereleaseTagFilter, []);
  }

}