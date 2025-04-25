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
  DotnetClient,
  NuGetClient,
  NugetOptions
} from '#domain/providers/dotnet';
import { nameOf } from '#domain/utils';

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

export function addProcessClient(services: IServiceCollection) {
  const serviceName = DotNetService.dotnetShellClient;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      createShellClient(
        container.shellCache,
        container.dotnetCachingOpts,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addCliClient(services: IServiceCollection) {
  const serviceName = DotNetService.dotnetCli;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new DotNetCli(
        container.dotnetConfig,
        container.dotnetShellClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addNuGetClient(services: IServiceCollection) {
  const serviceName = DotNetService.nugetClient;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new NuGetClient(
        createJsonClient(
          container.authorizer,
          {
            caching: container.dotnetCachingOpts,
            http: container.dotnetHttpOpts
          }
        ),
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addDotnetClient(services: IServiceCollection) {
  const serviceName = DotNetService.dotnetClient;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new DotnetClient(
        container.dotnetConfig,
        container.nugetClient,
        container.loggerFactory.create(serviceName)
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IDotNetServices & IDomainServices) =>
      new DotNetSuggestionProvider(
        container.dotnetClient,
        container.dotnetCli,
        container.nugetClient,
        container.dotnetConfig,
        container.loggerFactory.create(DotNetSuggestionProvider.name)
      )
  );
}