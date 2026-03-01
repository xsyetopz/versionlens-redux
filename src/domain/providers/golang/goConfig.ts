import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { GoFeatures } from '#domain/providers/golang';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the Go package provider.
 */
export class GoConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the GoConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for Go.
   * @param http The HTTP options for Go.
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
  readonly fileLanguage = 'go.mod';

  /**
   * Gets the file patterns used to identify Go files.
   */
  get filePatterns(): string {
    return this.config.get(GoFeatures.FilePatterns, '');
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(GoFeatures.PrereleaseTagFilter, []);
  }

  /**
   * Gets the base API URL for the Go proxy or registry.
   */
  get apiUrl(): string {
    return this.config.get(GoFeatures.ApiUrl, '');
  }

  /**
   * Gets the task to run when a Go file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(GoFeatures.OnSaveChangesTask, null);
  }

}