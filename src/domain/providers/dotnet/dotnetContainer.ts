import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addCliClient,
  addDotNetConfig,
  addDotnetSuggestionResolver,
  addHttpOptions,
  addNuGetClient,
  addNugetOptions,
  addSuggestionProvider
} from '#domain/providers/dotnet';

/**
 * Configures the DotNet service container by registering all necessary services.
 * @param serviceProvider The root service provider.
 * @param services The service collection to configure.
 * @returns A promise that resolves to the newly built child service provider.
 */
export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addNugetOptions(services);

  addHttpOptions(services);

  addDotNetConfig(services);

  addCliClient(services);

  addDotnetSuggestionResolver(services);

  addNuGetClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("dotnet", serviceProvider);
}