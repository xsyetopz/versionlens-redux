import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IFrozenOptions } from '#domain/configuration';
import type { IProviderConfig } from '#domain/providers';
import { type NugetOptions, DotNetFeatures } from '#domain/providers/dotnet';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Configuration for the DotNet package provider.
 */
export class DotNetConfig implements IProviderConfig {

  /**
   * Initializes a new instance of the DotNetConfig class.
   * @param config The frozen options from the configuration.
   * @param caching The caching options for DotNet.
   * @param http The HTTP options for DotNet.
   * @param nugetOptions The NuGet-specific options.
   */
  constructor(
    readonly config: IFrozenOptions,
    readonly caching: CachingOptions,
    readonly http: HttpOptions,
    readonly nugetOptions: NugetOptions,
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('caching', caching);
    throwUndefinedOrNull('http', http);
    throwUndefinedOrNull('nuget', nugetOptions);
  }

  /**
   * The file language supported by this provider.
   */
  readonly fileLanguage = 'xml';

  /**
   * Gets the file patterns used to identify DotNet files.
   */
  get filePatterns(): string {
    return this.config.get(DotNetFeatures.FilePatterns, '');
  }

  /**
   * Gets the file exclusion patterns for DotNet discovery.
   */
  get fileExcludePatterns(): string[] { return ['**/obj/**', '**/bin/**']; }

  /**
   * Gets the property names that contain dependencies in DotNet files.
   */
  get dependencyProperties(): Array<string> {
    return this.config.get(DotNetFeatures.DependencyProperties, []);
  }

  /**
   * Gets the fallback NuGet source URL.
   */
  get fallbackNugetSource(): string {
    return 'https://api.nuget.org/v3/index.json';
  }

  /**
   * Gets the task to run when a DotNet file is saved.
   */
  get onSaveChangesTask(): string | null {
    return this.config.get(DotNetFeatures.OnSaveChangesTask, null);
  }

  /**
   * Gets the filters used for prerelease tags.
   */
  get prereleaseTagFilter(): Array<string> {
    return this.config.get(DotNetFeatures.PrereleaseTagFilter, []);
  }

}