import type { CachingOptions } from '#domain/caching';
import type { IHttpOptions } from '#domain/clients';
import type { IServiceCollection, IServiceProvider } from '#domain/di';
import type { ILogger } from '#domain/logging';
import type {
  IPackageClient,
  PackageDependency,
  TSuggestionReplaceFunction
} from '#domain/packages';

export type FileMatcher = {
  language: string
  scheme: string
  pattern: string
  exclude?: string[]
}

export interface IProviderConfig {
  fileMatcher: FileMatcher;
  caching: CachingOptions;
  http: IHttpOptions;
  onSaveChangesTask: string;
}

export interface IProviderModule {

  configureContainer(
    serviceProvider: IServiceProvider,
    services: IServiceCollection
  ): Promise<IServiceProvider>

}

export interface ISuggestionProvider {

  name: string;

  config: IProviderConfig;

  client: IPackageClient<any>

  logger: ILogger;

  suggestionReplaceFn?: TSuggestionReplaceFunction;

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency>;

  /**
   * Optional function called before queueing all suggestion fetch requests.
   * Providers can return custom client data that will be sent with each suggestion fetch request
   * @param packagePath 
   */
  preFetchSuggestions?(
    projectPath: string,
    packagePath: string
  ): Promise<any>;

}

export interface IProviderServices {

  suggestionProvider: ISuggestionProvider;

}