import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addCliClient,
  addDotNetConfig,
  addHttpOptions,
  addJsonClient,
  addDotnetClient,
  addNuGetClient,
  addNugetOptions,
  addProcessClient,
  addSuggestionProvider
} from '#domain/providers/dotnet';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addNugetOptions(services);

  addHttpOptions(services);

  addDotNetConfig(services);

  addProcessClient(services);

  addCliClient(services);

  addJsonClient(services);

  addDotnetClient(services);

  addNuGetClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("dotnet", serviceProvider);
}