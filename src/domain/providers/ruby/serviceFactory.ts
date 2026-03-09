import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { createJsonClient, GitHubJsonClient, HttpOptions } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IRubyServices,
  RubyConfig,
  RubyFeatures,
  RubyGitHubClient,
  RubyHttpClient,
  RubyService,
  RubySuggestionProvider,
  RubySuggestionResolver
} from '#domain/providers/ruby';
import { nameOf } from '#domain/utils';

/**
 * Registers Ruby caching options as a singleton.
 * @param services The service collection to add to.
 */
export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    RubyService.rubyCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        RubyFeatures.Caching,
        'caching'
      )
  );
}

/**
 * Registers Ruby HTTP options as a singleton.
 * @param services The service collection to add to.
 */
export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    RubyService.rubyHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        RubyFeatures.Http,
        'http'
      )
  );
}

/**
 * Registers the Ruby configuration as a singleton.
 * @param services The service collection to add to.
 */
export function addRubyConfig(services: IServiceCollection) {
  services.addSingleton(
    RubyService.rubyConfig,
    (container: IRubyServices & IDomainServices) =>
      new RubyConfig(
        container.appConfig,
        container.rubyCachingOpts,
        container.rubyHttpOpts
      )
  );
}

/**
 * Registers the Ruby HTTP client as a singleton.
 * @param services The service collection to add to.
 */
export function addRubyHttpClient(services: IServiceCollection) {
  const serviceName = RubyService.rubyHttpClient;
  services.addSingleton(
    serviceName,
    (container: IRubyServices & IDomainServices) =>
      new RubyHttpClient(
        container.rubyConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.rubyCachingOpts,
            http: container.rubyHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Ruby GitHub client as a singleton.
 * @param services The service collection to add to.
 */
export function addRubyGitHubClient(services: IServiceCollection) {
  services.addSingleton(
    RubyService.rubyGithubClient,
    (container: IRubyServices & IDomainServices) =>
      new RubyGitHubClient(
        new GitHubJsonClient(
          container.cachingOptions,
          createJsonClient(
            container.authorizer,
            {
              caching: container.rubyCachingOpts,
              http: container.rubyHttpOpts
            }
          ),
          container.urlRequestCache
        )
      )
  );
}

/**
 * Registers the Ruby suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addRubySuggestionResolver(services: IServiceCollection) {
  const serviceName = RubyService.rubySuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: IRubyServices & IDomainServices) =>
      new RubySuggestionResolver(
        container.rubyConfig,
        container.rubyHttpClient,
        container.rubyGithubClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the Ruby suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IRubyServices & IDomainServices) =>
      new RubySuggestionProvider(
        container.rubySuggestionResolver,
        container.rubyConfig,
        container.loggerFactory.create(RubySuggestionProvider.name)
      )
  );
}
