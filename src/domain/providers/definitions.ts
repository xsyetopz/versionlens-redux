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

export interface IProviderConfig {
  readonly fileLanguage: string | string[]
  readonly filePatterns: string
  readonly caching: CachingOptions
  readonly http: HttpOptions
  readonly fileExcludePatterns?: string[]
  readonly onSaveChangesTask?: string | null
}

export interface IProviderModule {
  configureContainer(
    serviceProvider: IServiceProvider,
    services: IServiceCollection
  ): Promise<IServiceProvider>
}

export interface ISuggestionProvider {
  readonly name: string;
  readonly config: IProviderConfig;
  readonly logger: ILogger;
  suggestionReplaceFn?: SuggestionReplaceFunction;
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency>;
  /**
   * Optional function called before queueing all suggestion fetch requests.
   * Providers can return custom client data that will be sent with each suggestion fetch request
   * @param packagePath 
   */
  preFetchSuggestions?(projectPath: string, packagePath: string): Promise<any | undefined>;

  fetchSuggestions(request: PackageClientRequest<any>): Promise<PackageClientResponse>;
}

export interface IProviderServices {
  suggestionProvider: ISuggestionProvider;
}