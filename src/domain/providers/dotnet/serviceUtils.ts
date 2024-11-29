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
  DotNetSuggestionProvider,
  NuGetPackageClient,
  NuGetResourceClient,
  NugetOptions
} from '#domain/providers/dotnet';
import { nameOf } from '#domain/utils';

export function addCachingOptions(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDotNetServices>().dotnetCachingOpts,
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
    nameOf<IDotNetServices>().dotnetHttpOpts,
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
    nameOf<IDotNetServices>().nugetOpts,
    (container: IDomainServices) =>
      new NugetOptions(
        container.appConfig,
        DotNetFeatures.Nuget
      )
  );
}

export function addDotNetConfig(services: IServiceCollection) {
  services.addSingleton(
    nameOf<IDotNetServices>().dotnetConfig,
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
  const serviceName = nameOf<IDotNetServices>().dotnetShellClient;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      createShellClient(
        container.shellCache,
        container.dotnetCachingOpts,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addCliClient(services: IServiceCollection) {
  const serviceName = nameOf<IDotNetServices>().dotnetCli;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new DotNetCli(
        container.dotnetConfig,
        container.dotnetShellClient,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addJsonClient(services: IServiceCollection) {
  const serviceName = nameOf<IDotNetServices>().dotnetJsonClient;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      createJsonClient(
        container.authorizer,
        {
          caching: container.dotnetCachingOpts,
          http: container.dotnetHttpOpts
        }
      )
  );
}

export function addNuGetPackageClient(services: IServiceCollection) {
  const serviceName = nameOf<IDotNetServices>().nugetClient;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new NuGetPackageClient(
        container.dotnetConfig,
        container.dotnetJsonClient,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addNuGetResourceClient(services: IServiceCollection) {
  const serviceName = nameOf<IDotNetServices>().nugetResClient;
  services.addSingleton(
    serviceName,
    (container: IDotNetServices & IDomainServices) =>
      new NuGetResourceClient(
        container.dotnetJsonClient,
        container.logger.child({ logGroup: serviceName })
      )
  );
}

export function addSuggestionProvider(services: IServiceCollection) {
  services.addScoped(
    nameOf<IProviderServices>().suggestionProvider,
    (container: IDotNetServices & IDomainServices) =>
      new DotNetSuggestionProvider(
        container.nugetClient,
        container.dotnetCli,
        container.nugetResClient,
        container.dotnetConfig,
        container.logger.child({ logGroup: 'dotnetSuggestionProvider' })
      )
  );
}