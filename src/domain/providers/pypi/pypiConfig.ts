import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { PypiFeatures } from '#domain/providers/pypi';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the PyPi package provider.
 */
export class PypiConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the PypiConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for PyPi.
   * @param http The HTTP options for PyPi.
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
   * Gets the file patterns used to identify PyPi files (e.g., pyproject.toml).
   */
  get filePatterns(): string {
    return this.config.get(PypiFeatures.FilePatterns, '');
  }

  /**
   * Gets the property names that contain dependencies in PyPi files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(PypiFeatures.DependencyProperties, []);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(PypiFeatures.PrereleaseTagFilter, []);
  }

  /**
   * Gets the base API URL for the PyPi registry.
   */
  get apiUrl(): string {
    return this.config.get(PypiFeatures.ApiUrl, '');
  }

  /**
   * Gets the task to run when a PyPi file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(PypiFeatures.OnSaveChangesTask, null);
  }

}