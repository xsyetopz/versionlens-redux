import type { IAuthorizer } from '#domain/authorization';
import type { CachingOptions, IExpiryCache } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { Config } from '#domain/configuration';
import type { IServiceCollectionFactory, IServiceProvider } from '#domain/di';
import type { ILoggerSink, LoggerFactory } from '#domain/logging';
import type { DependencyCache, PackageCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { IStorage } from '#domain/storage';
import type {
  FetchPackage,
  FetchPackages,
  GetDependencyChanges,
  GetSuggestionProvider,
  GetSuggestions,
  GetSuggestionsStats
} from '#domain/useCases';
import { type EventScheduler, nameOf } from '#domain/utils';

export interface IDomainServices {
  serviceCollectionFactory: IServiceCollectionFactory
  serviceProvider: IServiceProvider
  authorizer: IAuthorizer
  appConfig: Config
  loggerSinks: ILoggerSink[]
  loggerFactory: LoggerFactory
  httpOptions: HttpOptions
  cachingOptions: CachingOptions
  storage: IStorage
  providerNames: Array<string>
  suggestionProviders: Array<ISuggestionProvider>
  eventScheduler: EventScheduler
  fileWatcherDependencyCache: DependencyCache
  packageCache: PackageCache
  shellCache: IExpiryCache
  urlRequestCache: IExpiryCache
  GetSuggestionProvider: GetSuggestionProvider
  fetchPackages: FetchPackages
  fetchPackage: FetchPackage
  getSuggestions: GetSuggestions
  getDependencyChanges: GetDependencyChanges
  getSuggestionsStats: GetSuggestionsStats
}

export const DomainServiceName = nameOf<IDomainServices>()