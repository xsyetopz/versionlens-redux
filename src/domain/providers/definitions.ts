import { ICachingOptions } from '#domain/caching';
import { IServiceCollection, IServiceProvider } from '#domain/di';
import { IHttpOptions } from '#domain/http';
import { ILogger } from '#domain/logging';
import {
  IPackageClient,
  PackageDependency,
  TSuggestionReplaceFunction
} from '#domain/packages';

export type TProviderFileMatcher = {
  language: string;
  scheme: string;
  pattern: string;
  exclude: string;
}

export interface IProviderConfig {

  fileMatcher: TProviderFileMatcher;

  caching: ICachingOptions;

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