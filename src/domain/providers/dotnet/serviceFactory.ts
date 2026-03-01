import type { IDomainServices } from '#domain';
import { CachingOptions } from '#domain/caching';
import { HttpOptions, createJsonClient, createShellClient } from '#domain/clients';
import type { IServiceCollection } from '#domain/di';
import type { IProviderServices } from '#domain/providers';
import {
  type IDotNetServices,
  DotNetCli,
  DotNetConfig,
  DotNetFeatures,
  DotNetService,
  DotNetSuggestionProvider,
  DotnetSuggestionResolver,
  NuGetClient,
  NugetOptions
} from '#domain/providers/dotnet';
import { nameOf } from '#domain/utils';

/**
 * Registers DotNet caching options as a singleton.
 * @param services The service collection to add to.
 */
export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    DotNetService.dotnetCachingOpts,
    (container: IDomainServices) =>
      new CachingOptions(
        container.appConfig,
        DotNetFeatures.Caching,
        'caching'
      )
  );
}

/**
 * Registers DotNet HTTP options as a singleton.
 * @param services The service collection to add to.
 */
export function addHttpOptions(services: IServiceCollection) {
  services.addSingleton(
    DotNetService.dotnetHttpOpts,
    (container: IDomainServices) =>
      new HttpOptions(
        container.appConfig,
        DotNetFeatures.Http,
        'http'
      )
  );
}

/**
 * Registers NuGet options as a singleton.
 * @param services The service collection to add to.
 */
export function addNugetOptions(services: IServiceCollection) {
  services.addSingleton(
    DotNetService.nugetOpts,
    (container: IDomainServices) =>
      new NugetOptions(
        container.appConfig,
        DotNetFeatures.Nuget
      )
  );
}

/**
 * Registers the DotNet configuration as a singleton.
 * @param services The service collection to add to.
 */
export function addDotNetConfig(services: IServiceCollection) {
  services.addSingleton(
    DotNetService.dotnetConfig,
    (container: IDotNetServices & IDomainServices) =>
      new DotNetConfig(
        container.appConfig,
        container.dotnetCachingOpts,
        container.dotnetHttpOpts,
        container.nugetOpts
      )
  );
}

/**
 * Registers the DotNet CLI client as a singleton.
 * @param services The service collection to add to.
 */
export function addCliClient(services: IServiceCollection) {
  const serviceName = DotNetService.dotnetCli;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new DotNetCli(
        container.dotnetConfig,
        createShellClient(
          container.shellCache,
          container.dotnetCachingOpts,
          container.loggerFactory.create(serviceName)
        ),
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the NuGet client as a singleton.
 * @param services The service collection to add to.
 */
export function addNuGetClient(services: IServiceCollection) {
  const serviceName = DotNetService.nugetClient;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new NuGetClient(
        container.dotnetConfig,
        createJsonClient(
          container.authorizer,
          {
            caching: container.dotnetCachingOpts,
            http: container.dotnetHttpOpts
          }
        ),
        container.urlRequestCache,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the DotNet suggestion resolver as a singleton.
 * @param services The service collection to add to.
 */
export function addDotnetSuggestionResolver(services: IServiceCollection) {
  const serviceName = DotNetService.dotnetSuggestionResolver;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new DotnetSuggestionResolver(
        container.dotnetConfig,
        container.nugetClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

/**
 * Registers the DotNet suggestion provider as a scoped service.
 * @param services The service collection to add to.
 */
export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IDotNetServices & IDomainServices) =>
      new DotNetSuggestionProvider(
        container.dotnetSuggestionResolver,
        container.dotnetCli,
        container.nugetClient,
        container.dotnetConfig,
        container.loggerFactory.create(DotNetSuggestionProvider.name)
      )
  );
}