import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { DenoFeatures } from '#domain/providers/deno';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the Deno package provider.
 */
export class DenoConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the DenoConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for Deno.
   * @param http The HTTP options for Deno.
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
   * Gets the file patterns used to identify Deno files.
   */
  get filePatterns(): string {
    return this.config.get(DenoFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in Deno files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(DenoFeatures.DependencyProperties, []);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(DenoFeatures.PrereleaseTagFilter, []);
  }

  /**
   * Gets the task to run when a Deno file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(DenoFeatures.OnSaveChangesTask, null);
  }

}