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

/**
 * Defines the core services available in the domain layer.
 */
export interface IDomainServices {
  /** Factory for creating service collections. */
  serviceCollectionFactory: IServiceCollectionFactory
  /** The root service provider for the domain. */
  serviceProvider: IServiceProvider
  /** The authorizer for handling credentials. */
  authorizer: IAuthorizer
  /** The application configuration. */
  appConfig: Config
  /** The collection of logger sinks. */
  loggerSinks: ILoggerSink[]
  /** The factory for creating logger instances. */
  loggerFactory: LoggerFactory
  /** Global HTTP client options. */
  httpOptions: HttpOptions
  /** Global caching options. */
  cachingOptions: CachingOptions
  /** The storage provider for file access. */
  storage: IStorage
  /** List of available package provider names. */
  providerNames: Array<string>
  /** List of initialized suggestion providers. */
  suggestionProviders: Array<ISuggestionProvider>
  /** The scheduler for recurring events. */
  eventScheduler: EventScheduler
  /** Cache for dependencies parsed from files. */
  fileWatcherDependencyCache: DependencyCache
  /** Cache for package suggestion responses. */
  packageCache: PackageCache
  /** Cache for shell command results. */
  shellCache: IExpiryCache
  /** Cache for URL-based requests. */
  urlRequestCache: IExpiryCache
  /** Use case for matching files to providers. */
  GetSuggestionProvider: GetSuggestionProvider
  /** Use case for batch fetching package data. */
  fetchPackages: FetchPackages
  /** Use case for fetching a single package. */
  fetchPackage: FetchPackage
  /** Use case for retrieving suggestions for a file. */
  getSuggestions: GetSuggestions
  /** Use case for detecting dependency changes in a file. */
  getDependencyChanges: GetDependencyChanges
  /** Use case for generating suggestion statistics. */
  getSuggestionsStats: GetSuggestionsStats
}

/**
 * Service name constant for domain services.
 */
export const DomainServiceName = nameOf<IDomainServices>()