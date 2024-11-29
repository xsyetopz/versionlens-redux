import type { IAuthorizer } from '#domain/authorization';
import type { CachingOptions, IExpiryCache } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { Config } from '#domain/configuration';
import type { IServiceCollectionFactory, IServiceProvider } from '#domain/di';
import type { ILogger, ILoggerChannel, LoggingOptions } from '#domain/logging';
import type { DependencyCache, IPackageFileWatcher, PackageCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { IStorage } from '#domain/storage';
import type {
  FetchPackage,
  FetchPackages,
  GetDependencyChanges,
  GetSuggestionProvider,
  GetSuggestions
} from '#domain/useCases';

export interface IDomainServices {
  serviceCollectionFactory: IServiceCollectionFactory;
  serviceProvider: IServiceProvider;
  authorizer: IAuthorizer;
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
  shellCache: IExpiryCache;
  GetSuggestionProvider: GetSuggestionProvider;
  fetchPackages: FetchPackages;
  fetchPackage: FetchPackage;
  getSuggestions: GetSuggestions;
  getDependencyChanges: GetDependencyChanges;
}