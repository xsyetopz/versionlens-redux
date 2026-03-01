import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { IServiceCollection, IServiceProvider } from '#domain/di';
import type { ILogger } from '#domain/logging';
import type {
  PackageClientRequest,
  PackageClientResponse,
  PackageDependency,
  SuggestionReplaceFunction
} from '#domain/packages';

/**
 * Base configuration for a package provider.
 */
export interface IProviderConfig {
  /**
   * The file languages supported by this provider (e.g., 'json', 'toml').
   */
  readonly fileLanguage: string | string[]
  /**
   * Glob patterns to identify package files for this provider.
   */
  readonly filePatterns: string
  /**
   * Caching configuration for the provider.
   */
  readonly caching: CachingOptions
  /**
   * HTTP configuration for the provider.
   */
  readonly http: HttpOptions
  /**
   * Optional glob patterns to exclude from package file discovery.
   */
  readonly fileExcludePatterns?: string[]
  /**
   * Optional task to run when a package file is saved.
   */
  readonly onSaveChangesTask?: string | null
}

/**
 * Represents a module that can configure a provider's service container.
 */
export interface IProviderModule {
  /**
   * Configures the service container for the provider.
   * @param serviceProvider The root service provider.
   * @param services The service collection to configure.
   * @returns A promise resolving to the configured service provider.
   */
  configureContainer(
    serviceProvider: IServiceProvider,
    services: IServiceCollection
  ): Promise<IServiceProvider>
}

/**
 * Interface for a suggestion provider that handles dependency parsing and fetching.
 */
export interface ISuggestionProvider {
  /**
   * The unique name of the provider.
   */
  readonly name: string;
  /**
   * The provider's configuration.
   */
  readonly config: IProviderConfig;
  /**
   * The logger for the provider.
   */
  readonly logger: ILogger;
  /**
   * Optional function to customize how suggestions are inserted into the file.
   */
  suggestionReplaceFn?: SuggestionReplaceFunction;
  /**
   * Parses a package file and returns an array of identified dependencies.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency>;
  /**
   * Optional function called before queueing all suggestion fetch requests.
   * Providers can return custom client data that will be sent with each suggestion fetch request.
   * @param projectPath The path to the project.
   * @param packagePath The path to the package file.
   * @returns A promise resolving to custom client data or undefined.
   */
  preFetchSuggestions?(projectPath: string, packagePath: string): Promise<any | undefined>;

  /**
   * Fetches suggestions for a specific dependency.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  fetchSuggestions(request: PackageClientRequest<any>): Promise<PackageClientResponse>;
}

/**
 * Service mapping for provider-specific services.
 */
export interface IProviderServices {
  /**
   * The suggestion provider service.
   */
  suggestionProvider: ISuggestionProvider;
}