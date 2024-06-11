import { CachingOptions, IExpiryCache } from '#domain/caching';
import { Config } from '#domain/configuration';
import { IServiceCollectionFactory, IServiceProvider } from 'domain/di';
import { HttpOptions } from 'domain/http';
import { ILogger, ILoggerChannel, LoggingOptions } from 'domain/logging';
import { DependencyCache, IPackageFileWatcher, PackageCache } from 'domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { IStorage } from 'domain/storage';
import {
  FetchPackageSuggestions,
  FetchProjectSuggestions,
  GetDependencyChanges,
  GetSuggestionProvider,
  GetSuggestions
} from 'domain/useCases';

export interface IDomainServices {

  serviceCollectionFactory: IServiceCollectionFactory;

  serviceProvider: IServiceProvider;

  appConfig: Config;

  loggingOptions: LoggingOptions;

  httpOptions: HttpOptions;

  cachingOptions: CachingOptions;

  logger: ILogger;

  loggerChannel: ILoggerChannel;

  storage: IStorage,

  providerNames: Array<string>;

  suggestionProviders: Array<ISuggestionProvider>;

  packageFileWatcher: IPackageFileWatcher;

  fileWatcherDependencyCache: DependencyCache;

  packageCache: PackageCache;

  processesCache: IExpiryCache;

  GetSuggestionProvider: GetSuggestionProvider;

  fetchProjectSuggestions: FetchProjectSuggestions;

  fetchPackageSuggestions: FetchPackageSuggestions;

  getSuggestions: GetSuggestions;

  getDependencyChanges: GetDependencyChanges;

}